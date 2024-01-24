use std::vec;
#[cfg(not(feature = "js"))]
use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use three_rs::math::{
    vector3::Vector3,
    EulerOrder, 
};
#[cfg(not(feature = "js"))]
use three_rs::math::{
    path3::Path3,
    Euler
};
#[cfg(feature = "js")]
use crate::js_renderer::JsRenderer;
#[cfg(not(feature = "js"))]
use crate::renderer::Renderer;
use crate::{
    renderer::ModelRef,
    player::{Player, PlayerAction, NUM_SENSOR_FEATURES}, 
    object::{Object, ObjectType}, 
    enemy::Enemy, 
    map::Map, 
    env::{Env, EnvObs}
};
#[cfg(not(feature = "js"))]
use crate::{
    env::{EnvAction, EnvStep, EnvPlayer},
    object::{ObjectState, ObjectVisibility}, 
    spawn_locs::SpawnLocations, 
    sensor::Sensor
};

#[cfg(not(feature = "js"))]
const PLAYER_SPAWN_LOCATION_NAME: &str = "psl.";
#[cfg(not(feature = "js"))]
const _ENEMY_SPAWN_LOCATION_NAME: &str = "esl.";
#[cfg(not(feature = "js"))]
const KEY_SPAWN_LOCATION_NAME: &str = "ksl.";
#[cfg(not(feature = "js"))]
const EXIT_SPAWN_LOCATION_NAME: &str = "xsl.";

#[cfg(not(feature = "js"))]
const PENALTY_WALL: f32 = -10.0;
#[cfg(not(feature = "js"))]
const PENALTY_TRAP: f32 = -50.0;
#[cfg(not(feature = "js"))]
const REWARD_KEY_FOUND: f32 = 100.0;
#[cfg(not(feature = "js"))]
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
pub struct LevelOptions {
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameOptions {
    pub level: LevelOptions,
    pub fov: ObjectFOV,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectCreatedEvent {
    pub model_ref: ModelRef,
    pub typ: ObjectType,
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectSpawnedEvent {
    pub model_ref: ModelRef,
    pub position: Vector3,
    pub direction: Vector3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectMovedEvent {
    pub model_ref: ModelRef,
    pub position: Vector3,
    pub direction: Vector3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectDiedEvent {
    pub model_ref: ModelRef,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectShownEvent {
    pub model_ref: ModelRef,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectHiddenEvent {
    pub model_ref: ModelRef,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum GameEvent {
    ObjectCreated(ObjectCreatedEvent),
    ObjectSpawned(ObjectSpawnedEvent),
    ObjectShown(ObjectShownEvent),
    ObjectHidden(ObjectHiddenEvent),
    ObjectMoved(ObjectMovedEvent),
    ObjectDied(ObjectDiedEvent),
}

#[cfg_attr(not(feature = "js"), derive(Clone, Serialize, Deserialize))]
pub struct Game {
    pub map_info: MapInfo,
    pub events: Vec<GameEvent>,
    pub entities: GameEntities,
    pub options: GameOptions,
    pub state: GameState,
    pub map: Map,
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

    #[cfg(not(feature = "js"))]
    fn random_action(
        &mut self,
        rng: &mut StdRng
    ) -> EnvAction {
        rng.gen::<usize>() % self.get_num_actions()
    }

    #[cfg(not(feature = "js"))]
    fn reset(
        &mut self,
        renderer: &mut dyn Renderer,
        rng: &mut StdRng,
    ) -> Vec<EnvObs> {
        renderer.reset();
        self.events = vec![];

        // map
        let level_ref = renderer.add_model(
            self.options.level.name.clone()
        ).expect("Could not load level-0");
        
        self.events.push(GameEvent::ObjectCreated(ObjectCreatedEvent {
            model_ref: level_ref,
            typ: ObjectType::UNKNOWN,
            id: self.options.level.name.clone(),
        }));

        renderer.show_model(level_ref);
        
        // entities
        self.entities = self.spawn_entities(renderer, rng);
        
        self.map_info = MapInfo { 
            num_keys: self.options.num_players * self.options.min_keys
        };

        self.state = GameState {
            status: GameStatus::RUNNING,
            winner: None,
            step: 0
        };

        self.build_players_sensors(renderer);

        self.entities.players
            .iter()
            .map(|p| {
                p.to_obs()
            })
            .collect()
    }

    #[cfg(feature = "js")]
    fn reset(
        &mut self,
        renderer: JsRenderer,
    ) -> Vec<EnvObs> {
        renderer.reset();
        vec![]
    }

    #[cfg(not(feature = "js"))]
    fn step(
        &mut self, 
        actions: &Vec<Option<crate::env::EnvAction>>, 
        delta_time: f32,
        renderer: &mut dyn Renderer
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
        
            for i in 0..actions.len() {
                let player = &mut self.entities.players[i];
                player.reward = 0.0;

                let dir = match actions[i] {
                    None => {
                        player.base.dir.clone()
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
                    self.move_player(i, &dir, delta_time, renderer);
                }
            }

            self.update_world(delta_time, renderer);
            
            self.build_players_sensors(renderer);
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

    fn serialize_events(
        &self
    ) -> Result<Vec<u8>, String> {
        rmp_serde::to_vec(&self.events)
            .map_err(|err| err.to_string())
    }

    fn deserialize_events(
        &mut self,
        buf: &Vec<u8>
    ) -> Result<(), String> {
        self.events = rmp_serde::from_slice(buf)
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    fn clear_events(
        &mut self
    ) {
        self.events = vec![];
    }

    #[cfg(not(feature = "js"))]
    fn render(
        &mut self,
        renderer: &mut dyn Renderer
    ) {
        renderer.render();
    }

    #[cfg(feature = "js")]
    async fn render(
        &mut self,
        renderer: JsRenderer,
    ) {
        for event in &self.events {
            match event {
                GameEvent::ObjectCreated(e) => {
                    renderer.add_model(&e.id, e.model_ref).await;
                },
                
                GameEvent::ObjectSpawned(e) => {
                    renderer.set_position(
                        e.model_ref, &e.position.to_slice()
                    );
                    renderer.set_rotation_from_euler(
                        e.model_ref, &e.direction.to_slice(), EulerOrder::XYZ as u32
                    );
                },
                
                GameEvent::ObjectMoved(e) => {
                    renderer.set_position(
                        e.model_ref, &e.position.to_slice()
                    );
                    renderer.set_rotation_from_euler(
                        e.model_ref, &e.direction.to_slice(), EulerOrder::XYZ as u32
                    );
                },

                GameEvent::ObjectDied(e) => {
                    renderer.hide_model(e.model_ref);
                },
                
                GameEvent::ObjectShown(e) => {
                    renderer.show_model(e.model_ref);

                },
                GameEvent::ObjectHidden(e) => {
                    renderer.hide_model(e.model_ref);
                },
            }
        }

        renderer.render();
    }
}

impl Game {
    pub fn new(
        level_bytes: &[u8], 
        navmesh_bytes: &[u8],
        options: GameOptions
    ) -> Self {
        let map = Map::new(level_bytes, navmesh_bytes);

        Self { 
            map,
            entities: GameEntities::default(),
            events: vec![],
            map_info: MapInfo::default(),
            state: GameState { 
                status: GameStatus::IDLE, 
                winner: None, 
                step: 0,
            },
            options
        }
    }

    #[cfg(not(feature = "js"))]
    fn spawn_entities(
        &mut self,
        renderer: &mut dyn Renderer,
        rng: &mut StdRng
    ) -> GameEntities {
        // players
        let players = self.spawn_players(renderer, rng);

        // keys
        let mut objects = self.spawn_objects(
            ObjectType::KEY,
            KEY_SPAWN_LOCATION_NAME, 
            &|i| i == 0, 
            renderer,
            rng
        );
        
        // door
        objects.append(&mut self.spawn_objects(
            ObjectType::EXIT,
            EXIT_SPAWN_LOCATION_NAME,
            &|_| false, 
            renderer,
            rng
        ));

        GameEntities {
            objects,
            players,
            enemies: vec![],
        }
    }

    #[cfg(not(feature = "js"))]
    fn spawn_players(
        &mut self,
        renderer: &mut dyn Renderer,
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
            let id = format!("player-{}", i);
            let model_ref = renderer.add_model(
                id.clone()
            ).unwrap();
            
            self.events.push(GameEvent::ObjectCreated(ObjectCreatedEvent {
                model_ref,
                typ: ObjectType::PLAYER,
                id,
            }));

            #[cfg(not(target_arch = "wasm32"))]
            let (path_ref, sensor_ref, readings_ref) = if self.options.debug {
                (
                    renderer
                        .add_from_linestrips([0.0, 1.0, 1.0], vec![]).unwrap(),
                    renderer
                        .add_from_lines([63.0 / 255.0, 63.0 / 255.0, 63.0 / 255.0], vec![]).unwrap(),
                    renderer
                        .add_from_lines([1.0, 1.0, 1.0], vec![]).unwrap()
                )                
            }
            else {
                (0, 0, 0)
            };
            #[cfg(target_arch = "wasm32")]
            let (path_ref, sensor_ref, readings_ref) = (0, 0, 0);

            let loc = &spawn_locs.next(i, rng);
            let player = Player::new(
                loc.0, 
                loc.1, 
                Vector3::new(0.75, 1.8, 0.5), //FIXME!!!
                model_ref,
                path_ref,
                sensor_ref,
                readings_ref,
                rng
            );

            self.events.push(GameEvent::ObjectSpawned({
                ObjectSpawnedEvent {
                    model_ref,
                    position: loc.0,
                    direction: loc.1,
                }
            }));

            renderer.show_model(model_ref);
            self.events.push(GameEvent::ObjectShown(ObjectShownEvent {
                model_ref
            }));

            #[cfg(not(target_arch = "wasm32"))]
            if self.options.debug {
                renderer.show_model(path_ref);
                renderer.show_model(sensor_ref);
                renderer.show_model(readings_ref);
            }
            
            players.push(player);
        }

        players
    }

    #[cfg(not(feature = "js"))]
    fn spawn_objects(
        &mut self,
        typ: ObjectType,
        name: &str,
        active: &dyn Fn(usize) -> bool,
        renderer: &mut dyn Renderer,
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

        let rad = if typ == ObjectType::KEY {1.0} else {2.0};

        for i in 0..num_objs {
            let id = if typ == ObjectType::KEY {format!("key-{}", i)} else {format!("exit-{}", i)};
            let model_ref = renderer.add_model(
                id.clone()
            ).unwrap();
            self.events.push(GameEvent::ObjectCreated(ObjectCreatedEvent {
                model_ref,
                typ: typ.clone(),
                id,
            }));
                
            let loc = &spawn_locs.next(i, rng);
            let is_active = active(i);
            let obj = Object::new(
                typ.clone(),
                loc.0.add(&Vector3::new(0.0, rad / 2.0, 0.0)), 
                loc.1, 
                Vector3::new(rad * 2.0, rad * 2.0, rad * 2.0),
                ObjectState::IDLE,
                if is_active {ObjectVisibility::VISIBLE} else {ObjectVisibility::HIDDEN},
                model_ref,
                rng
            );

            renderer.set_position(model_ref, &obj.pos);
            renderer.set_rotation_from_euler(
                model_ref, 
                &Euler::from_vector(obj.dir, EulerOrder::XYZ)
            );
            self.events.push(GameEvent::ObjectSpawned({
                ObjectSpawnedEvent {
                    model_ref,
                    position: loc.0.add(&Vector3::new(0.0, 2.0, 0.0)),
                    direction: loc.1,
                }
            }));

            if is_active {
                renderer.show_model(model_ref);
                self.events.push(GameEvent::ObjectShown(ObjectShownEvent {
                    model_ref
                }));
            }
            else {
                renderer.hide_model(model_ref);
                self.events.push(GameEvent::ObjectHidden(ObjectHiddenEvent {
                    model_ref
                }));
            }

            objects.push(obj);
        }

        objects
    }

    #[cfg(not(feature = "js"))]
    fn spawn_exit(
        &mut self,
        renderer: &mut dyn Renderer
    ) {
        let exit = self.entities.objects.iter_mut().find(
            |o| o.typ == ObjectType::EXIT && 
            o.vis == ObjectVisibility::HIDDEN && 
            o.state != ObjectState::DEAD
        );
        if let Some(obj) = exit {
            obj.vis = ObjectVisibility::VISIBLE;
            
            renderer.set_position(obj.model_ref, &obj.pos);
            self.events.push(GameEvent::ObjectMoved(ObjectMovedEvent{
                model_ref: obj.model_ref,
                position: obj.pos,
                direction: obj.dir,
            }));
            
            renderer.show_model(obj.model_ref);
            self.events.push(GameEvent::ObjectShown(ObjectShownEvent {
                model_ref: obj.model_ref
            }));
        }
    }

    #[cfg(not(feature = "js"))]
    fn spawn_key(
        &mut self,
        renderer: &mut dyn Renderer
    ) {
        let key = self.entities.objects.iter_mut().find(
            |o| o.typ == ObjectType::KEY && 
            o.vis == ObjectVisibility::HIDDEN && 
            o.state != ObjectState::DEAD
        );
        if let Some(obj) = key {
            obj.vis = ObjectVisibility::VISIBLE;
            
            renderer.set_position(obj.model_ref, &obj.pos);
            self.events.push(GameEvent::ObjectMoved(
                ObjectMovedEvent {
                    model_ref: obj.model_ref,
                    position: obj.pos,
                    direction: obj.dir,
                }
            ));
            
            renderer.show_model(obj.model_ref);
            self.events.push(GameEvent::ObjectShown(ObjectShownEvent {
                model_ref: obj.model_ref
            }));
        }
    }

    #[cfg(not(feature = "js"))]
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

    #[cfg(not(feature = "js"))]
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    fn build_players_sensors(
        &mut self,
        renderer: &mut dyn Renderer
    ) {
        let mut readings = vec![(false, vec![]); self.entities.players.len()];
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
                
                self.map.find_path_to_target(
                    &player.base.pos, &target_obj.pos)
            };

            let target_pos = path.nodes[1].clone();
            if self.options.debug {
                paths.push(path);
            }

            readings[i] = player.get_sensor_readings(
                &target_pos, &self.options.fov, &mut self.map);
        }

        for i in 0..self.entities.players.len() {
            let player = &mut self.entities.players[i];
            player.sensor = Sensor {
                has_hits: readings[i].0,
                readings: readings[i].1.to_owned(),
            };
            
            #[cfg(not(target_arch = "wasm32"))]
            if self.options.debug {
                renderer.set_vertices(
                    player.path_ref, 
                    paths[i].nodes.clone()
                );
                let (sensor_vtx, readings_vtx) = Player::gen_vis_sensor_readings(
                    &player.base.pos, 
                    &readings[i].1,
                    self.options.fov.max_dist
                );
                
                renderer.set_vertices(
                    player.sensor_ref, 
                    sensor_vtx  
                );
                renderer.set_vertices(
                    player.readings_ref, 
                    readings_vtx
                );
            }
        }
    }

    #[cfg(not(feature = "js"))]
    fn move_player(
        &mut self,
        num: usize,
        dir: &Vector3,
        delta_time: f32,
        renderer: &mut dyn Renderer
    ) {
        let player = &mut self.entities.players[num];
        
        let new_pos = player.base.pos.add(
            &dir.mul_scalar(delta_time).mul(&player.vel)
        );
        
        if let Some(confined_pos) = 
            self.map.move_along_surface(&player.base.pos, &new_pos) {
            player.preserve();
            if confined_pos != new_pos {
                player.base.pos = confined_pos;
                player.base.dir = player.calc_dir_away_from_hit();
                player.reward += PENALTY_WALL;
            }
            else {
                player.base.pos = new_pos;
                player.base.dir = dir.clone();
            }

            let mean_dir = player.mean_dir();

            self.events.push(GameEvent::ObjectMoved(
                ObjectMovedEvent {
                    model_ref: player.base.model_ref,
                    position: player.base.pos,
                    direction: mean_dir,
                }
            ));
            
            renderer.set_position(
                player.base.model_ref, 
                &player.base.pos
            );
            renderer.set_rotation_from_euler(
                player.base.model_ref, 
                &Euler::from_vector(mean_dir, EulerOrder::XYZ)
            );
        }
    }

    #[cfg(not(feature = "js"))]
    fn check_keys(
        &mut self,
        num: usize,
        renderer: &mut dyn Renderer
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

                renderer.hide_model(obj.model_ref);
                self.events.push(GameEvent::ObjectHidden(ObjectHiddenEvent {
                    model_ref: obj.model_ref
                }));
            }
        }

        self.entities.players[num] = player;

        if collected {
            self.spawn_key(renderer);
        }
        
        collected
    }

    #[cfg(not(feature = "js"))]
    fn check_traps(
        &mut self,
        num: usize,
        renderer: &mut dyn Renderer
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
                
                renderer.hide_model(obj.model_ref);
                self.events.push(GameEvent::ObjectHidden(ObjectHiddenEvent {
                    model_ref: obj.model_ref
                }));
            }
        }

        self.entities.players[num] = player;
        
        triggered
    }

    #[cfg(not(feature = "js"))]
    fn check_exit(
        player: &Object,
        obj: &Object,
    ) -> bool {
        let player_bb = player.calc_bb();
        let item_bb = obj.calc_bb();

        player_bb.intersects_with_box(&item_bb)
    }

    #[cfg(not(feature = "js"))]
    fn update_world(
        &mut self,
        _delta_time: f32,
        renderer: &mut dyn Renderer
    ) {
        let mut all_keys_collected = false;

        for i in 0..self.entities.players.len() {
            if self.check_keys(i, renderer) {
                let player = &mut self.entities.players[i];
                if player.num_keys >= self.options.min_keys {
                    all_keys_collected = true;
                }
            }

            if self.check_traps(i, renderer) {
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
            self.spawn_exit(renderer);
        }
    }
}