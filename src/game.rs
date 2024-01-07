use std::f32::consts::PI;
use navmesh_rs::query::NavMeshQuery;
use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use three_rs::math::{
    vector3::{Vector3, FORWARD}, 
    euler::{Euler, EulerOrder}, 
    quaternion::Quaternion, 
    path3::Path3
};
use crate::{
    player::{Player, PlayerAction, NUM_SENSOR_FEATURES}, 
    object::{Object, ObjectType, ObjectState, ObjectVisibility, ObjectDir}, 
    enemy::Enemy, 
    map::Map, 
    env::{Env, EnvAction, EnvObs, EnvStep, EnvPlayer}, 
    spawn_loc::SpawnLocations, 
    renderer::Renderer, 
    sensor::{SensorReading, SensorReadingTarget, SensorReadingHit}
};

const MAX_NAVMESH_PATH_LEN: usize = 64;

const PLAYER_SPAWN_LOCATION_NAME: &str = "spl.";
const _ENEMY_SPAWN_LOCATION_NAME: &str = "epl.";
const KEY_SPAWN_LOCATION_NAME: &str = "kpl.";
const EXIT_SPAWN_LOCATION_NAME: &str = "xpl.";

const PENALTY_WALL: f32 = -10.0;
const PENALTY_TRAP: f32 = -50.0;
const REWARD_KEY_FOUND: f32 = 100.0;
const REWARD_EXIT_FOUND: f32 = 1000.0;

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub enum GameStatus {
    IDLE,
    RUNNING,
    TIMEDOUT,
    FINISHED,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ObjectFOV {
    pub rays: usize,
    pub angle: f32,
    pub max_dist: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    pub status: GameStatus,
    pub winner: Option<usize>,
    pub step: usize,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MapInfo {
    pub num_keys: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameOptions {
    pub fov: ObjectFOV,
    pub fps: f32,
    pub num_players: usize,
    pub min_keys: usize,
    pub player_radius: f32,
    pub debug: bool,
    pub training: bool,
    pub max_steps: usize,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct GameEntities {
    pub objects: Vec<Object>,
    pub players: Vec<Player>,
    pub enemies: Vec<Enemy>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct GameStep {
    pub entities: GameEntities,
}

pub struct Game{
    pub map: Map,
    pub map_info: MapInfo,
    pub entities: GameEntities,
    pub options: GameOptions,
    pub state: GameState,
}

impl Env for Game {
    fn get_num_actions(
        &self
    ) -> usize {
        PlayerAction::COUNT
    }

    fn get_num_obs(
        &self
    ) -> usize {
        NUM_SENSOR_FEATURES * self.options.fov.rays
    }

    fn random_action(
        &mut self,
        rng: &mut StdRng
    ) -> EnvAction {
        rng.gen::<usize>() % self.get_num_actions()
    }

    fn reset(
        &mut self,
        rng: &mut StdRng
    ) -> Vec<EnvObs> {
        
        self.entities = GameEntities {
            objects: vec![],
            players: vec![],
            enemies: vec![],
        };

        self.spawn_entities(rng);

        self.map_info = MapInfo { 
            num_keys: self.options.num_players * self.options.min_keys
        };

        self.state = GameState {
            status: GameStatus::RUNNING,
            winner: None,
            step: 0
        };

        self.entities.players
            .iter()
            .map(|p| {
                p.to_obs()
            })
            .collect()
    }

    fn step(
        &mut self, 
        actions: &Vec<Option<crate::env::EnvAction>>, 
        delta_time: f32
    ) -> EnvStep {
        if self.state.status != GameStatus::RUNNING {
            return EnvStep {
                terminated: true,
                timedout: false,
                winner: self.state.winner.clone(),
                players: vec![],
            };
        }

        self.state.step += 1;
        
        if self.options.max_steps == 0 ||
            self.state.step < self.options.max_steps {
        
            self.build_players_sensors();

            for i in 0..actions.len() {
                let player = &mut self.entities.players[i];
                player.reward = 0.0;

                let dir = match actions[i] {
                    None => {
                        if player.sensor.readings.iter().any(|r| match r {
                            SensorReading::Target(_) => false,
                            SensorReading::Hit(_) => true,
                        }) {
                            player.calc_dir_away_from_hit()
                        }
                        else {
                            player.base.dir.rot.clone()
                        }
                    },
                    Some(action) => {
                        let player_action = PlayerAction::try_from(action.clone()).unwrap();
                        match player_action {
                            PlayerAction::MoveForward =>
                                player.base.dir.rot.clone(),
                            PlayerAction::TurnRight =>
                                Euler::from_quaternion(
                                    &player.base.dir.q.rotate_z(-player.turn_angle),
                                    EulerOrder::XYZ
                                ).v,
                            PlayerAction::TurnLeft => 
                                Euler::from_quaternion(
                                    &player.base.dir.q.rotate_z(player.turn_angle),
                                    EulerOrder::XYZ
                                ).v
                        }
                    }
                };

                if dir.length_sq() != 0.0 {
                    self.move_player(i, &dir, delta_time);
                }
            }

            self.update_world(delta_time);
        }
        else {
            self.state.status = GameStatus::TIMEDOUT;
        }

        EnvStep {
            terminated: self.state.status != GameStatus::RUNNING,
            timedout: self.state.status == GameStatus::TIMEDOUT,
            winner: self.state.winner,
            players: self.entities.players
                .iter()
                .map(|p| {
                    EnvPlayer {
                        reward: p.reward,
                        obs: p.to_obs(),
                    }
                })
                .collect()
        }
    }

    fn render(
        &self, 
        _pov: &Vector3, 
        _scale: f32, 
        _renderer: &mut dyn Renderer
    ) {
        todo!()
    }

    fn serialize_entities(
        &self
    ) -> Result<Vec<u8>, String> {
        rmp_serde::to_vec(&self.entities)
            .map_err(|err| err.to_string())
    }

    fn deserialize_entities(
        &mut self,
        buf: &Vec<u8>
    ) -> Result<(), String> {
        self.entities = rmp_serde::from_slice(buf)
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

impl Game {
    pub fn new(
        map: Map,
        options: GameOptions
    ) -> Self {
        Self { 
            map,
            entities: GameEntities::default(),
            map_info: MapInfo::default(),
            state: GameState { 
                status: GameStatus::IDLE, 
                winner: None, 
                step: 0,
            },
            options
        }
    }
    
    pub fn spawn_entities(
        &mut self,
        rng: &mut StdRng
    ) -> GameEntities {
        let options = &self.options;
        let map = &self.map;
        
        // players
        let players = Self::spawn_players(
            options.num_players, 
            map, 
            rng
        );

        // keys
        let mut objects = Self::spawn_objects(
            ObjectType::KEY,
            KEY_SPAWN_LOCATION_NAME, 
            options.num_players * options.min_keys, 
            &|i| i == 0, 
            map, 
            rng
        );
        
        // door
        objects.append(&mut Self::spawn_objects(
            ObjectType::EXIT,
            EXIT_SPAWN_LOCATION_NAME,
            1,
            &|_| false, 
            map, 
            rng
        ));

        GameEntities {
            objects,
            players,
            enemies: vec![],
        }
    }

    fn spawn_players(
        num: usize,
        map: &Map,
        rng: &mut StdRng
    ) -> Vec<Player> {
        let mut locs = SpawnLocations::new(
            Map::get_nodes_transform(
                map.find_nodes_by_partial_name(PLAYER_SPAWN_LOCATION_NAME)
            )
        );

        let mut players = vec![];

        for i in 0..num {
            let loc = &locs.next(i, rng);
            let player = Player::new(
                loc.0, 
                loc.1, 
                Vector3::one().mul_scalar(0.5), //FIXME!!!
                rng
            );
            players.push(player);
        }

        players
    }

    fn spawn_objects(
        typ: ObjectType,
        name: &str,
        num: usize,
        active: &dyn Fn(usize) -> bool,
        map: &Map,
        rng: &mut StdRng
    ) -> Vec<Object> {
        let mut locs = SpawnLocations::new(
            Map::get_nodes_transform(
                map.find_nodes_by_partial_name(name)
            )
        );

        let mut objects = vec![];

        for i in 0..num {
            let loc = &locs.next(i, rng);
            let is_active = active(i);
            let obj = Object::new(
                typ.clone(),
                loc.0, 
                loc.1, 
                Vector3::one().mul_scalar(0.5), //FIXME!!!
                ObjectState::IDLE,
                if is_active {ObjectVisibility::VISIBLE} else {ObjectVisibility::HIDDEN},
                rng
            );
            objects.push(obj);
        }

        objects
    }

    fn spawn_exit(
        &mut self,
    ) {
        let exit = self.entities.objects.iter_mut().find(
            |o| o.typ == ObjectType::EXIT && 
            o.vis == ObjectVisibility::HIDDEN && 
            o.state != ObjectState::DEAD
        );
        if let Some(obj) = exit {
            obj.vis = ObjectVisibility::VISIBLE;
        }
    }

    fn spawn_key(
        &mut self,
    ) {
        let key = self.entities.objects.iter_mut().find(
            |o| o.typ == ObjectType::KEY && 
            o.vis == ObjectVisibility::HIDDEN && 
            o.state != ObjectState::DEAD
        );
        if let Some(obj) = key {
            obj.vis = ObjectVisibility::VISIBLE;
        }
    }

    fn find_closest_object(
        &self,
        pos: &Vector3,
        typ: ObjectType
    ) -> Object {

        let mut min_distance = f32::MAX;
        let mut res = Object::default();

        for obj in self.entities.objects.iter().filter(|o| 
            o.vis == ObjectVisibility::VISIBLE && 
            o.state != ObjectState::DEAD && 
            o.typ == typ
        ) {
            let distance = pos.sub(&obj.pos).length();
            if distance < min_distance {
                min_distance = distance;
                res = obj.clone();
            }
        }

        res
    }

    fn find_path_to_target(
        &self, 
        from: &Vector3,
        to: &Vector3,
        limit: usize
    ) -> Path3 {
        let query = NavMeshQuery::new(&self.map.nav_mesh, limit);
        Path3::new(query.find_path(from, to))
    }

    fn calc_distance_to_closest_wall(
        &self,
        from: &Vector3,
        to: &Vector3
    ) -> Option<f32> {
        let query = NavMeshQuery::new(&self.map.nav_mesh, MAX_NAVMESH_PATH_LEN);
        if let Some(hit) = query.raycast(from, to) {
            Some(hit.0)
        }
        else {
            None
        }
    }

    fn calc_similarity_with_target(
        &self,
        dir: &Vector3,
        target_dir: &Vector3
    ) -> f32 {
        dir.dot(&target_dir).max(0.0)
    }

    fn build_players_sensors(
        &mut self
    ) {
        let mut sensors = vec![vec![]; self.entities.players.len()];
        let mut paths = vec![];

        for (i, player) in self.entities.players.iter().enumerate() {
            let target_obj = self.find_closest_object(
                &player.base.pos, 
                if player.num_keys == self.options.min_keys {
                    ObjectType::EXIT
                }
                else {
                    ObjectType::KEY
                }
            );

            // when training, it's better to just use target's position
            let path = if self.options.training {
                Path3::new(vec![
                    player.base.pos.clone(), 
                    target_obj.pos.clone()
                ])
            }
            // when playing, move around obstacles
            else {
                self.find_path_to_target(&player.base.pos, &target_obj.pos, 2)
            };

            let target_pos = path.nodes[1].clone();
            if self.options.debug {
                paths.push(path);
            }

            //FIXME!!!
            let target_dir = Euler::from_quaternion(
                &Quaternion::from_axis_and_angle(
                    &FORWARD, 
                    player.base.pos.angle_to(&target_pos)
                ),
                EulerOrder::XYZ
            ).v;
            //target_pos.sub(&player.base.pos).normalize();

            let mut rot = player.base.dir.q.rotate_z((self.options.fov.angle / 2.0) % (PI * 2.0));
            let rot_step = self.options.fov.angle / ((self.options.fov.rays - 1) as f32);

            let mut player_sensors = vec![];

            for _ in 0..self.options.fov.rays {
                let dir = Euler::from_quaternion(&rot, EulerOrder::XYZ).v;
                let sensor_pos = dir.mul_scalar(self.options.fov.max_dist);
                
                player_sensors.push(
                    if let Some(t) = self.calc_distance_to_closest_wall(
                        &player.base.pos, &sensor_pos) {
                        SensorReading::Hit(SensorReadingHit{t})
                    } else {
                        SensorReading::Target(SensorReadingTarget { 
                            similarity: self.calc_similarity_with_target(&dir, &target_dir), 
                            angle: rot.angle(),
                            dir,
                        })
                    }
                );

                rot = rot.rotate_z(rot_step);
            }
            
            sensors[i] = player_sensors;
        }

        for i in 0..self.entities.players.len() {
            let player = &mut self.entities.players[i];
            player.sensor.readings = sensors[i].to_owned();
            if self.options.debug {
                player.path = paths[i].to_owned();
            }
        }
    }

    fn move_player(
        &mut self,
        num: usize,
        dir: &Vector3,
        delta_time: f32
    ) {
        let player = &mut self.entities.players[num];
        
        let vel = player.vel.mul(dir).mul_scalar(delta_time);
        let new_pos = player.base.pos.add(&vel);
        
        let query = NavMeshQuery::new(&self.map.nav_mesh, MAX_NAVMESH_PATH_LEN);
        if let Some(confined_pos) = query.move_along_surface(&player.base.pos, &new_pos) {
            if confined_pos != new_pos {
                player.base.pos = confined_pos;
                player.reward += PENALTY_WALL;
                player.vel = Vector3::zero();
            }
            else {
                player.base.pos = new_pos;
                player.preserve();
                player.base.dir = ObjectDir::from_vector(dir);
            }
        }
    }

    fn check_keys(
        &mut self,
        num: usize
    ) -> bool {
        let mut collected = false;
        let mut player = self.entities.players[num].clone();
        
        for obj in self.entities.objects.iter_mut().filter(|o| 
            o.vis == ObjectVisibility::VISIBLE && 
            o.state != ObjectState::DEAD &&
            o.typ == ObjectType::KEY
        ) {
            let player_bb = player.calc_bb();
            let item_bb = obj.calc_bb();

            if player_bb.intersects_with_box(&item_bb) {
                player.num_keys += 1;
                player.reward += REWARD_KEY_FOUND;
                obj.state = ObjectState::DEAD;
                obj.vis = ObjectVisibility::HIDDEN;
                collected = true;
            }
        }

        self.entities.players[num] = player;

        if collected {
            self.spawn_key();
        }
        
        collected
    }

    fn check_traps(
        &mut self,
        num: usize
    ) -> bool {
        let mut triggered = false;
        let mut player = self.entities.players[num].clone();
        
        for obj in self.entities.objects.iter_mut().filter(|o| 
            o.vis == ObjectVisibility::VISIBLE && 
            o.state != ObjectState::DEAD && 
            o.typ == ObjectType::TRAP
        ) {
            let player_bb = player.calc_bb();
            let item_bb = obj.calc_bb();

            if player_bb.intersects_with_box(&item_bb) {
                player.reward += PENALTY_TRAP;
                obj.state = ObjectState::DEAD;
                triggered = true;
            }
        }

        self.entities.players[num] = player;
        
        triggered
    }

    fn check_exit(
        player: &Object,
        obj: &Object,
    ) -> bool {
        let player_bb = player.calc_bb();
        let item_bb = obj.calc_bb();

        player_bb.intersects_with_box(&item_bb)
    }

    fn update_world(
        &mut self,
        delta_time: f32
    ) {
        let mut all_keys_collected = false;

        for i in 0..self.entities.players.len() {
            if self.check_keys(i) {
                let player = &mut self.entities.players[i];
                if player.num_keys >= self.options.min_keys {
                    all_keys_collected = true;
                }
            }

            if self.check_traps(i) {
                let player = &self.entities.players[i];
                if player.health <= 0.0 {
                    self.state.status = GameStatus::FINISHED;
                    self.state.winner = None;
                }
            }

            {
                let player = &self.entities.players[i];
                if player.num_keys >= self.options.min_keys {
                    let door = self.entities.objects.iter().find(|o| o.typ == ObjectType::EXIT);
                    if let Some(door) = door {
                        if Self::check_exit(&player.base, door) {
                            self.state.status = GameStatus::FINISHED;
                            self.state.winner = Some(i);
                            let player = &mut self.entities.players[i];
                            player.reward += REWARD_EXIT_FOUND;
                        }
                    }
                }
            }

            {
                let player = &mut self.entities.players[i];
                player.vel.add_mut(&player.accel.mul_scalar(delta_time));
                if player.vel.x > player.max_vel.x {
                    player.vel.x = player.max_vel.x;
                }
                if player.vel.y > player.max_vel.y {
                    player.vel.y = player.max_vel.y;
                }
            }
        }

        if all_keys_collected {
            self.spawn_exit();
        }
    }
}