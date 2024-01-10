use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use three_rs::math::{
    vector3::Vector3, 
    triangle::Triangle, quaternion::Quaternion, 
};
use crate::{
    player::{Player, PlayerAction, NUM_SENSOR_FEATURES}, 
    object::{Object, ObjectType, ObjectState, ObjectVisibility}, 
    enemy::Enemy, 
    map::Map, 
    env::{Env, EnvAction, EnvObs, EnvStep, EnvPlayer}, 
    spawn_locs::SpawnLocations, 
    renderer::Renderer, sensor::SensorReading
};

const PLAYER_SPAWN_LOCATION_NAME: &str = "psl.";
const _ENEMY_SPAWN_LOCATION_NAME: &str = "esl.";
const KEY_SPAWN_LOCATION_NAME: &str = "ksl.";
const EXIT_SPAWN_LOCATION_NAME: &str = "xsl.";

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

pub struct Game<'a> {
    pub map: Map,
    pub map_info: MapInfo,
    pub entities: GameEntities,
    pub options: GameOptions,
    pub state: GameState,
    pub renderer: &'a mut dyn Renderer,
}

impl Env for Game<'_> {
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
        
        self.renderer.reset();
        
        // map
        let level_ref = self.renderer.add_from_gltf(
            [192, 0, 0], &self.map.gltf_doc, &self.map.buffers
        ).expect("Could not load GLTF");
        self.renderer.show_model(level_ref);
        
        // entities
        self.entities = self.spawn_entities(rng);

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
                            player.base.dir.clone()
                        }
                    },
                    Some(action) => {
                        match PlayerAction::try_from(action.clone()).unwrap() {
                            PlayerAction::MoveForward =>
                                player.base.dir.clone(),
                            PlayerAction::TurnRight => {
                                player.base.dir.rotate_y(player.turn_angle)
                            },
                            PlayerAction::TurnLeft => {
                                player.base.dir.rotate_y(-player.turn_angle)
                            }
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

    fn render(
        &mut self
    ) {
        self.renderer.render();
    }
}

impl<'a> Game<'a> {
    pub fn new(
        level_bytes: &[u8], 
        navmesh_bytes: &[u8],
        options: GameOptions,
        renderer: &'a mut dyn Renderer
    ) -> Self {
        let map = Map::new(level_bytes, navmesh_bytes);

        Self { 
            map,
            entities: GameEntities::default(),
            map_info: MapInfo::default(),
            state: GameState { 
                status: GameStatus::IDLE, 
                winner: None, 
                step: 0,
            },
            options,
            renderer
        }
    }
    
    fn spawn_entities(
        &mut self,
        rng: &mut StdRng
    ) -> GameEntities {
        // players
        let players = self.spawn_players(rng);

        // keys
        let mut objects = self.spawn_objects(
            ObjectType::KEY,
            KEY_SPAWN_LOCATION_NAME, 
            &|i| i == 0, 
            rng
        );
        
        // door
        objects.append(&mut self.spawn_objects(
            ObjectType::EXIT,
            EXIT_SPAWN_LOCATION_NAME,
            &|_| false, 
            rng
        ));

        GameEntities {
            objects,
            players,
            enemies: vec![],
        }
    }

    fn spawn_players(
        &mut self,
        rng: &mut StdRng
    ) -> Vec<Player> {
        let mut spawn_locs = SpawnLocations::new(
            Map::get_nodes_transform(
                self.map.find_nodes_by_partial_name(PLAYER_SPAWN_LOCATION_NAME)
            )
        );

        if spawn_locs.locs.len() == 0 {
            panic!("No spawn locations '{}' for players found", PLAYER_SPAWN_LOCATION_NAME);
        }

        let mut players = vec![];

        for i in 0..self.options.num_players {
            let model_ref = self.renderer.add_from_triangles(
                [0, 255, 0],
                vec![
                    Triangle::new(
                        Vector3::new(-0.5, 0.0, -0.5),
                        Vector3::new(1.5, 0.0, 0.0),
                        Vector3::new(-0.5, 0.0, 0.5),
                    )
                ]
            ).unwrap();

            let (path_ref, sensor_ref) = if self.options.debug {
                (
                    self.renderer.add_from_linestrips([0, 255, 255], vec![]).unwrap(),
                    self.renderer.add_from_lines([255, 255, 255], vec![]).unwrap()
                )
            }
            else {
                (0, 0)
            };

            self.renderer.show_model(model_ref);
            if self.options.debug {
                self.renderer.show_model(path_ref);
                self.renderer.show_model(sensor_ref);
            }
                
            let loc = &spawn_locs.next(i, rng);
            let player = Player::new(
                loc.0, 
                loc.1, 
                Vector3::new(0.75, 1.8, 0.5), //FIXME!!!
                model_ref,
                path_ref,
                sensor_ref,
                rng
            );
            players.push(player);
        }

        players
    }

    fn spawn_objects(
        &mut self,
        typ: ObjectType,
        name: &str,
        active: &dyn Fn(usize) -> bool,
        rng: &mut StdRng
    ) -> Vec<Object> {
        let mut spawn_locs = SpawnLocations::new(
            Map::get_nodes_transform(
                self.map.find_nodes_by_partial_name(name)
            )
        );

        if spawn_locs.locs.len() == 0 {
            panic!("No spawn locations '{}' for objects found", name);
        }

        let mut objects = vec![];

        let num_objs = if typ == ObjectType::KEY {
            self.options.num_players * self.options.min_keys
        } 
        else {
            1
        };

        for i in 0..num_objs {
            let model_ref = self.renderer.add_from_triangles(
                [0, 0, 255],
                vec![
                    Triangle::new(
                        Vector3::new(-0.5, 0.0, -0.5),
                        Vector3::new(0.5, 0.0, -0.5),
                        Vector3::new(0.5, 0.0, 0.5),
                    ),
                    Triangle::new(
                        Vector3::new(-0.5, 0.0, -0.5),
                        Vector3::new(-0.5, 0.0, 0.5),
                        Vector3::new(0.5, 0.0, 0.5),
                    )
                ]
            ).unwrap();
                
            let loc = &spawn_locs.next(i, rng);
            let is_active = active(i);
            let obj = Object::new(
                typ.clone(),
                loc.0, 
                loc.1, 
                Vector3::one().mul_scalar(0.5), //FIXME!!!
                ObjectState::IDLE,
                if is_active {ObjectVisibility::VISIBLE} else {ObjectVisibility::HIDDEN},
                model_ref,
                rng
            );
            self.renderer.set_position(obj.model_ref, &obj.pos);
            if is_active {
                self.renderer.show_model(obj.model_ref);
            }

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
            self.renderer.set_position(obj.model_ref, &obj.pos);
            self.renderer.show_model(obj.model_ref);
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
            self.renderer.set_position(obj.model_ref, &obj.pos);
            self.renderer.show_model(obj.model_ref);
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

    fn build_players_sensors(
        &mut self
    ) {
        let mut readings = vec![vec![]; self.entities.players.len()];
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

            let path = self.map.find_path_to_target(&player.base.pos, &target_obj.pos);

            let target_pos = path.nodes[1].clone();
            if self.options.debug {
                paths.push(path);
            }

            readings[i] = player.get_sensor_readings(
                &target_pos, &self.options.fov, &self.map);
        }

        for i in 0..self.entities.players.len() {
            let player = &mut self.entities.players[i];
            player.sensor.readings = readings[i].to_owned();
            if self.options.debug {
                self.renderer.set_vertices(
                    player.path_ref, 
                    paths[i].nodes.clone()
                );
                self.renderer.set_vertices(
                    player.sensor_ref, 
                    Player::gen_vis_sensor_readings(
                        &player.base.pos, 
                        &readings[i],
                        self.options.fov.max_dist
                    )
                );
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
        
        let new_pos = player.base.pos.add(&dir.mul_scalar(delta_time).mul(&player.vel));
        
        if let Some(confined_pos) = 
            self.map.move_along_surface(&player.base.pos, &new_pos) {
            if confined_pos != new_pos {
                player.base.pos = confined_pos;
                player.reward += PENALTY_WALL;
                player.base.dir = dir.neg();
            }
            else {
                player.base.pos = new_pos;
                player.preserve();
                player.base.dir = dir.clone();
            }
            
            self.renderer.set_position(
                player.base.model_ref, 
                &player.base.pos
            );
            self.renderer.set_rotation(
                player.base.model_ref, 
                &player.base.dir, 
                &Quaternion::identity()
            );
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
                self.renderer.hide_model(obj.model_ref);
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
        _delta_time: f32
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
        }

        if all_keys_collected {
            self.spawn_exit();
        }
    }
}