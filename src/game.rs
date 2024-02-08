#[cfg(not(feature = "js"))]
use rand::{rngs::StdRng, Rng};
#[cfg(not(feature = "js"))]
use std::{collections::{HashMap, HashSet}, f32::consts::PI};
use serde::{Serialize, Deserialize};
use strum::EnumCount;
#[cfg(not(feature = "js"))]
use three_rs::math::{
    path3::Path3,
    FORWARD, UP,
    Vector3, 
    Quaternion
};
#[cfg(feature = "js")]
use crate::js_renderer::JsRenderer;
#[cfg(not(feature = "js"))]
use crate::renderer::Renderer;
use crate::{
    env::{Env, EnvObs}, 
    event::*, 
    map::Map, 
    npc::Npc, 
    object::Object, 
    player::{Player, PlayerAction, PlayerFOV, NUM_SENSOR_FEATURES}
};
#[cfg(not(feature = "js"))]
use crate::{
    consts,
    asset::{GameAssetRef, get_game_assets}, 
    env::{Asset, AssetRef, EnvAction, EnvStep, EnvPlayer},
    entity::{Entity, EntityData, EntityState, EntityVisibility, EntityType}, 
    spawn_loc_mng::SpawnLocManager, 
    sensor::Sensor,
    object::ObjectType,
    physics::manager::PhysicsManager,
};
#[cfg(feature = "js")]
use crate::physics::null_manager::PhysicsManager;
#[cfg(not(feature = "js"))]
use rapier3d::dynamics::RigidBodyType;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum GameStatus {
    IDLE,
    RUNNING,
    TIMEDOUT,
    FINISHED,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub status: GameStatus,
    pub winner: Option<usize>,
    pub step: usize,
    pub active_players: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MapInfo {
    pub num_keys: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelOptions {
    pub num: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptions {
    pub level: LevelOptions,
    pub fov: PlayerFOV,
    pub num_players: usize,
    pub min_keys: usize,
    pub player_radius: f32,
    pub debug: bool,
    pub training: bool,
    pub max_steps: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GameEntities {
    pub players: Vec<Player>,
    pub npcs: Vec<Npc>,
    pub objects: Vec<Object>,
}

#[cfg_attr(not(feature = "js"), derive(Clone, Serialize, Deserialize))]
pub struct Game {
    pub map_info: MapInfo,
    pub events: Vec<GameEvent>,
    pub entities: GameEntities,
    pub options: GameOptions,
    pub state: GameState,
    pub map: Map,
    pub physics: PhysicsManager,
}

impl Env for Game {
    #[cfg(not(feature = "js"))]
    fn get_assets(
        &self
    ) -> HashMap::<AssetRef, Asset> {
        get_game_assets()
    }
    
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
        let level_ref = renderer.add_model(GameAssetRef::Level as usize)
            .expect("Could not load level-0");
        
        self.events.push(GameEvent::EntityCreated(EntityCreatedEvent {
            model_ref: level_ref,
            typ: EntityType::UNKNOWN,
            asset_ref: GameAssetRef::Level as _,
        }));

        // entities
        self.entities = self.spawn_entities(renderer, rng);
        
        self.map_info = MapInfo { 
            num_keys: self.options.num_players * self.options.min_keys
        };

        self.state = GameState {
            status: GameStatus::RUNNING,
            winner: None,
            step: 0,
            active_players: self.options.num_players,
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
        renderer: &mut dyn Renderer,
        rng: &mut StdRng,
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
        
            self.update_players(actions, delta_time, rng);
            
            self.update_npcs(delta_time, rng);

            self.update_objects(delta_time, rng);
            
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
                        active: p.is_moving(),
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

    #[cfg(all(not(feature = "js"), not(target_arch = "wasm32")))]
    fn render(
        &mut self,
        renderer: &mut dyn Renderer,
        events_only: bool
    ) {
        for event in &self.events {
            match event {
                GameEvent::EntityCreated(_) => {
                    // nothing to do, the object was created already
                },
                
                GameEvent::EntitySpawned(e) => {
                    renderer.set_model_position(
                        e.model_ref, &e.pos
                    );
                    renderer.set_model_rotation(
                        e.model_ref, &e.rot
                    );
                },
                
                GameEvent::EntityMoved(e) => {
                    renderer.set_model_position(
                        e.model_ref, &e.pos
                    );
                    renderer.set_model_rotation(
                        e.model_ref, &e.rot
                    );
                },

                GameEvent::EntityStateChanged(e) => {
                    renderer.change_model_state(e.model_ref, e.from, e.to);
                },

                GameEvent::EntityRemoved(e) => {
                    renderer.hide_model(e.model_ref);
                },
                
                GameEvent::EntityVisChanged(e) => {
                    if e.visible {
                        renderer.show_model(e.model_ref);
                    }
                    else {
                        renderer.hide_model(e.model_ref);
                    }
                }
            }
        }

        if !events_only {
            renderer.render();
        }

        self.events = vec![];
    }

    #[cfg(feature = "js")]
    fn render(
        &mut self,
        renderer: JsRenderer,
    ) {
        for event in &self.events {
            match event {
                GameEvent::EntityCreated(e) => {
                    renderer.add_model(e.asset_ref, e.model_ref);
                },
                
                GameEvent::EntitySpawned(e) => {
                    renderer.set_model_position(
                        e.model_ref, &e.pos.to_slice()
                    );
                    renderer.set_model_rotation(
                        e.model_ref, &e.rot.to_slice()
                    );
                },
                
                GameEvent::EntityMoved(e) => {
                    renderer.set_model_position(
                        e.model_ref, &e.pos.to_slice()
                    );
                    renderer.set_model_rotation(
                        e.model_ref, &e.rot.to_slice()
                    );
                },

                GameEvent::EntityStateChanged(e) => {
                    renderer.change_model_state(
                        e.model_ref, e.from as u32, e.to as u32);
                },

                GameEvent::EntityRemoved(e) => {
                    renderer.hide_model(e.model_ref);
                },
                
                GameEvent::EntityVisChanged(e) => {
                    if e.visible {
                        renderer.show_model(e.model_ref);
                    }
                    else {
                        renderer.hide_model(e.model_ref);
                    }
                }
            }
        }

        renderer.render();

        self.events = vec![];
    }

    #[cfg(all(not(feature = "js"), target_arch = "wasm32"))]
    fn render(
        &mut self,
        _renderer: &mut dyn Renderer,
        _events_only: bool
    ) {
        self.events = vec![];
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
            events: vec![],
            map_info: MapInfo::default(),
            state: GameState { 
                status: GameStatus::IDLE, 
                winner: None, 
                step: 0,
                active_players: 0,
            },
            options,
            physics: PhysicsManager::new(),
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

        // npcs
        let npcs = self.spawn_npcs(renderer, rng);
        
        // exit
        let objects = self.spawn_objects(
            ObjectType::EXIT,
            consts::EXIT_SPAWN_LOCATION_NAME,
            &|_| false, 
            renderer,
            rng
        );

        GameEntities {
            players,
            npcs,
            objects,
        }
    }

    #[cfg(not(feature = "js"))]
    fn spawn_players(
        &mut self,
        renderer: &mut dyn Renderer,
        rng: &mut StdRng
    ) -> Vec<Player> {
        let mut spawn_mng = SpawnLocManager::new(
            Map::get_nodes_transform(
                self.map.find_nodes_by_partial_name(consts::PLAYER_SPAWN_LOCATION_NAME)
            )
        );

        if spawn_mng.locs.len() == 0 {
            panic!("No spawn locations '{}' for players found", consts::PLAYER_SPAWN_LOCATION_NAME);
        }

        let mut players = vec![];

        for i in 0..self.options.num_players {
            let model_ref = renderer.add_model(
                (GameAssetRef::Player1 as usize) + i
            ).unwrap();
            
            self.events.push(GameEvent::EntityCreated(EntityCreatedEvent {
                model_ref,
                typ: EntityType::PLAYER,
                asset_ref: (GameAssetRef::Player1 as usize) + i,
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

            let loc = &spawn_mng.next(i, rng);

            let char_ref = self.physics.add_character(
                &loc.0, 
                &loc.1, 
                consts::PLAYER_HEIGHT, 
                consts::PLAYER_WIDTH * 0.5, 
                true,
                Some(RigidBodyType::KinematicPositionBased)
            );

            let player = Player::new(
                loc.0, 
                loc.1, 
                Vector3::new(consts::PLAYER_WIDTH, consts::PLAYER_HEIGHT, 0.15),
                EntityState::WAITING,
                EntityVisibility::VISIBLE,
                char_ref,
                model_ref,
                path_ref,
                sensor_ref,
                readings_ref,
                rng
            );

            self.events.push(GameEvent::EntitySpawned({
                EntitySpawnedEvent {
                    model_ref,
                    pos: loc.0,
                    rot: loc.1,
                }
            }));

            self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                visible: true,
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
    fn spawn_npcs(
        &mut self,
        renderer: &mut dyn Renderer,
        rng: &mut StdRng
    ) -> Vec<Npc> {
        let mut spawn_locs = SpawnLocManager::new(
            Map::get_nodes_transform(
                self.map.find_nodes_by_partial_name(consts::NPC_SPAWN_LOCATION_NAME)
            )
        );

        if spawn_locs.locs.len() == 0 {
            panic!("No spawn locations '{}' for npcs found", consts::NPC_SPAWN_LOCATION_NAME);
        }

        let mut npcs = vec![];
        let num_npcs = self.options.num_players * self.options.min_keys;

        for i in 0..num_npcs {
            let model_ref = renderer.add_model(
                GameAssetRef::Npc1 as usize // + i
            ).unwrap();
            
            self.events.push(GameEvent::EntityCreated(EntityCreatedEvent {
                model_ref,
                typ: EntityType::PLAYER,
                asset_ref: GameAssetRef::Npc1 as usize //+ i,
            }));

            let loc = &spawn_locs.next(i, rng);
            let vis = if i == 0 {EntityVisibility::VISIBLE} else {EntityVisibility::HIDDEN};

            let char_ref = self.physics.add_character(
                &loc.0, 
                &loc.1, 
                consts::NPC_HEIGHT, 
                consts::NPC_WIDTH / 2.0, 
                if i == 0 {true} else {false},
                Some(RigidBodyType::Fixed)
            );

            let npc = Npc::new(
                loc.0, 
                loc.1, 
                Vector3::new(consts::NPC_WIDTH, consts::NPC_HEIGHT, 0.15),
                EntityState::WAITING,
                vis,
                char_ref,
                model_ref,
                rng
            );

            self.events.push(GameEvent::EntitySpawned(
                EntitySpawnedEvent {
                    model_ref,
                    pos: loc.0,
                    rot: loc.1,
                }
            ));

            if vis == EntityVisibility::VISIBLE {
                self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                    visible: true,
                    model_ref,
                }));
            }
            else {
                self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                    visible: false,
                    model_ref,
                }));
            }

            npcs.push(npc);
        }

        npcs
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
        let mut spawn_mng = SpawnLocManager::new(
            Map::get_nodes_transform(
                self.map.find_nodes_by_partial_name(name)
            )
        );

        if spawn_mng.locs.len() == 0 {
            panic!("No spawn locations '{}' for objects found", name);
        }

        let mut objects = vec![];
        let num_objs = 1;

        let rad = 1.0;

        for i in 0..num_objs {
            let asset_ref = GameAssetRef::Exit as usize;
            
            let model_ref = renderer.add_model(asset_ref)
                .expect("Invalid asset");

            self.events.push(GameEvent::EntityCreated(EntityCreatedEvent {
                model_ref,
                typ: EntityType::OBJECT,
                asset_ref,
            }));
                
            let loc = &spawn_mng.next(i, rng);
            let is_active = active(i);

            let char_ref = self.physics.add_character(
                &loc.0, 
                &loc.1, 
                consts::EXIT_HEIGHT, 
                consts::EXIT_WIDTH / 2.0, 
                false,
                Some(RigidBodyType::Fixed)
            );

            let obj = Object::new(
                typ.clone(),
                loc.0, 
                loc.1, 
                Vector3::new(rad, rad, rad),
                EntityState::WAITING,
                if is_active {EntityVisibility::VISIBLE} else {EntityVisibility::HIDDEN},
                char_ref,
                model_ref,
                rng
            );

            self.events.push(GameEvent::EntitySpawned({
                EntitySpawnedEvent {
                    model_ref,
                    pos: loc.0,
                    rot: loc.1,
                }
            }));

            if is_active {
                self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                    visible: true,
                    model_ref
                }));
            }
            else {
                self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                    visible: false,
                    model_ref
                }));
            }

            objects.push(obj);
        }

        objects
    }

    #[cfg(not(feature = "js"))]
    fn show_exit(
        &mut self
    ) {
        let exit = self.entities.objects.iter_mut().find(|o|
            o.typ == ObjectType::EXIT && 
            !o.is_visible() && !o.is_dead()
        );
        if let Some(obj) = exit {
            obj.data.vis = EntityVisibility::VISIBLE;

            self.physics.toggle_character(obj.data.char_ref, true);
            
            self.events.push(GameEvent::EntityMoved(EntityMovedEvent{
                model_ref: obj.data.model_ref,
                pos: obj.data.pos,
                rot: obj.data.rot,
            }));
            
            self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                visible: true,
                model_ref: obj.data.model_ref
            }));
        }
    }

    #[cfg(not(feature = "js"))]
    fn show_next_npc(
        &mut self
    ) {
        if let Some(npc) = self.entities.npcs.iter_mut().find(|n|
            !n.is_visible() && !n.is_dead()
        ) {
            npc.data.vis = EntityVisibility::VISIBLE;
            
            self.physics.toggle_character(npc.data.char_ref, true);
            
            self.events.push(GameEvent::EntityMoved(
                EntityMovedEvent {
                    model_ref: npc.data.model_ref,
                    pos: npc.data.pos,
                    rot: npc.data.rot,
                }
            ));

            self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                visible: true,
                model_ref: npc.data.model_ref
            }));
        }
    }

    #[cfg(not(feature = "js"))]
    fn find_closest_player(
        &self,
        pos: &Vector3,
        radius: Option<f32>,
        rng: &mut StdRng,
    ) -> Option<(usize, Player, f32)> {
        let mut min_distance_sq = if let Some(radius) = radius {radius * 2.0} else {f32::MAX};
        let mut res = None;

        let player_indexes = self.shuffle_players_idx(rng);

        for i in player_indexes {
            let player = &self.entities.players[i];
            if player.is_visible() && !player.is_dead() {
                let distance_sq = pos.sub(&player.data.pos).length_sq();
                if distance_sq < min_distance_sq {
                    min_distance_sq = distance_sq;
                    res = Some((i, player.clone(), distance_sq));
                }
            }
        }

        res
    }

    #[cfg(not(feature = "js"))]
    fn shuffle_players_idx(
        &self,
        rng: &mut StdRng
    ) -> Vec<usize> {
        let mut indexes = HashSet::new();
    
        while indexes.len() < self.options.num_players {
            indexes.insert(rng.gen_range(0..self.options.num_players));
        }
    
        indexes.into_iter().collect()
    }

    #[cfg(not(feature = "js"))]
    fn find_closest_npc(
        &self,
        pos: &Vector3,
        radius: Option<f32>
    ) -> Option<(usize, EntityData, f32)> {

        let mut min_distance_sq = if let Some(radius) = radius {radius * 2.0} else {f32::MAX};
        let mut res = None;

        for (i, npc) in self.entities.npcs.iter().enumerate().filter(|(_, n)| 
            n.is_visible() && !n.is_dead()
        ) {
            let distance_sq = pos.sub(&npc.data.pos).length_sq();
            if distance_sq < min_distance_sq {
                min_distance_sq = distance_sq;
                res = Some((i, npc.data.clone(), distance_sq));
            }
        }

        res
    }

    #[cfg(not(feature = "js"))]
    fn find_closest_object(
        &self,
        pos: &Vector3,
        typ: ObjectType
    ) -> EntityData {

        let mut min_distance = f32::MAX;
        let mut res = EntityData::default();

        for obj in self.entities.objects.iter().filter(|o| 
            o.typ == typ &&
            o.is_visible() && !o.is_dead()
        ) {
            let distance = pos.sub(&obj.data.pos).length();
            if distance < min_distance {
                min_distance = distance;
                res = obj.data.clone();
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
            let target_pos = if player.num_brains == self.options.min_keys { 
                self.find_closest_object(&player.data.pos, ObjectType::EXIT).pos
            }
            else {
                if let Some((_, npc, _)) = self.find_closest_npc(&player.data.pos, None) {
                    npc.pos
                }
                else {
                    Vector3::zero()
                }
            };

            // when training, it's better to just use target's position
            let path = if self.options.training {
                Path3::new(vec![
                    player.data.pos.clone(),
                    target_pos.clone()
                ])
            }
            // when playing, move around obstacles
            else {
                self.map.find_path_to_target(
                    &player.data.pos, &target_pos)
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
                renderer.set_model_vertices(
                    player.path_ref, 
                    paths[i].nodes.clone()
                );
                let (_sensor_vtx, readings_vtx) = Player::gen_vis_sensor_readings(
                    &player.data.pos, 
                    &readings[i].1,
                    self.options.fov.max_dist
                );
                
                /*renderer.set_vertices(
                    player.sensor_ref, 
                    sensor_vtx  
                );*/
                renderer.set_model_vertices(
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
        rot: &Quaternion,
        vel: &Vector3,
        delta_time: f32,
        forced: bool
    ) {
        let player = &mut self.entities.players[num];
        
        let dir = FORWARD.apply_quaternion(&rot);
        
        let translation = dir.mul(vel)
            .mul_scalar(delta_time);

        let new_pos = player.data.pos.add(&translation);
        
        if let Some(confined_pos) = 
            self.map.move_along_surface(&player.data.pos, &new_pos) {
            player.preserve();
            if confined_pos.x != new_pos.x || confined_pos.z != new_pos.z {
                player.data.pos = confined_pos.mul(&Vector3::new(1.0, 0.0, 1.0));
                if !forced {
                    player.data.rot = player.calc_rot_away_from_hit();
                    player.reward += consts::PENALTY_WALL;
                }
            }
            else {
                player.data.pos = if !self.options.training {
                    if let Some(translation) = 
                        self.physics.move_character(
                            player.data.char_ref, &translation) {
                        dbg!(translation);
                        new_pos.add(&translation)
                    }
                    else {
                        new_pos
                    }
                }
                else {
                    new_pos
                };
                
                if !forced {
                    player.data.rot = rot.clone();
                }
            }

            let mean_rot = player.mean_rot();

            self.events.push(GameEvent::EntityMoved(
                EntityMovedEvent {
                    model_ref: player.data.model_ref,
                    pos: player.data.pos,
                    rot: mean_rot,
                }
            ));
        }
    }

    #[cfg(not(feature = "js"))]
    fn check_exit(
        player: &Player,
        obj: &Object,
    ) -> bool {
        let player_bb = player.calc_bb();
        let item_bb = obj.calc_bb();

        player_bb.intersects_with_box(&item_bb)
    }

    #[cfg(not(feature = "js"))]
    fn update_players(
        &mut self, 
        actions: &Vec<Option<usize>>, 
        delta_time: f32,
        _rng: &mut StdRng,
    ) {
        // 1st: update state
        for player in self.entities.players.iter_mut()
            .filter(|p| p.is_visible()) {
            if let Some((from, to)) = player.update_state(self.state.step) {
                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                    model_ref: player.data.model_ref,
                    from,
                    to
                }));

                if to == EntityState::DEAD {
                    self.state.active_players -= 1;
                    if self.state.active_players == 0 {
                        self.state.status = GameStatus::FINISHED;
                        self.state.winner = None;
                    }
                    
                    player.data.vis = EntityVisibility::HIDDEN;
                    self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                        visible: false,
                        model_ref: player.data.model_ref 
                    }));
                }
            }
        }

        // 2nd: move player
        for (i, action) in actions.iter().enumerate() {
            let player = &mut self.entities.players[i];
            if player.is_visible() && !player.is_dead() {
                player.reward = 0.0;

                if player.is_moving() {
                    let rot = match action {
                        None => {
                            player.data.rot.clone()
                        },
                        Some(action) => {
                            match PlayerAction::try_from(action.clone()).unwrap() {
                                PlayerAction::MoveForward =>
                                    player.data.rot.clone(),
                                PlayerAction::TurnRight => {
                                    player.data.rot.mul(
                                        &Quaternion::from_axis_and_angle(&UP, player.turn_angle)
                                    )
                                },
                                PlayerAction::TurnLeft => {
                                    player.data.rot.mul(
                                        &Quaternion::from_axis_and_angle(&UP, -player.turn_angle)
                                    )
                                }
                            }
                        }
                    };

                    let vel = player.vel.clone();
                    self.move_player(i, &rot, &vel, delta_time, false);
                }
            }
        }

        // 3rd: check environment
        for i in 0..self.entities.players.len() {
            let player = &self.entities.players[i];
            if player.is_visible() && !player.is_dead() {
                // all brains eaten? check if exit reached
                if player.num_brains >= self.options.min_keys {
                    let door = self.entities.objects.iter().find(|o| o.typ == ObjectType::EXIT);
                    if let Some(door) = door {
                        if Self::check_exit(player, door) {
                            self.state.status = GameStatus::FINISHED;
                            self.state.winner = Some(i);
                            
                            let player = &mut self.entities.players[i];
                            player.reward += consts::REWARD_EXIT_REACHED;
                            
                            if let Some(from) = player.set_state(EntityState::CELEBRATING, self.state.step) {
                                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                    model_ref: player.data.model_ref,
                                    from,
                                    to: player.data.state
                                }));
                            }
                        }
                    }
                }
                // finished and not the winner? turn off animations
                else if self.state.status == GameStatus::FINISHED {
                    let player = &mut self.entities.players[i];
                    if let Some(from) = player.set_state(EntityState::IDLE, self.state.step) {
                        self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                            model_ref: player.data.model_ref,
                            from,
                            to: player.data.state
                        }));
                    }
                }
                // check the map..
                else {
                    // any npc close?
                    if let Some((npc_num, _, _)) = 
                        self.find_closest_npc(&player.data.pos, Some(1.0)) {

                        let player = &mut self.entities.players[i];
                        // biting and enough time elapsed?
                        if player.is_biting() &&
                            player.state_elapsed(self.state.step) >= 0.75 {
                            player.reward += consts::REWARD_BRAIN_EATEN;

                            // stop biting
                            if let Some(from) = player.set_state(EntityState::IDLE, self.state.step) {
                                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                    model_ref: player.data.model_ref,
                                    from,
                                    to: player.data.state
                                }));
                            }
                        
                            let npc = &mut self.entities.npcs[npc_num];
                            npc.health += consts::PENALTY_BITTEN;
                            // NPC has no health left? kill it and show another one
                            if npc.health <= 0.0 {
                                if let Some(from) = npc.set_state(EntityState::DYING, self.state.step) {
                                    self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                        model_ref: npc.data.model_ref,
                                        from,
                                        to: npc.data.state
                                    }));
                                }

                                player.num_brains += 1;
                                if player.num_brains == self.options.min_keys {
                                    self.show_exit();
                                } 
                                
                                self.show_next_npc();
                            }
                            // otherwise, just change its state to hurting
                            else {
                                if npc.data.state != EntityState::HIT && 
                                    npc.state_elapsed(self.state.step) >= 0.5 {
                                    if let Some(from) = npc.set_state(EntityState::HIT, self.state.step) {
                                        self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                            model_ref: npc.data.model_ref,
                                            from,
                                            to: npc.data.state
                                        }));
                                    }
                                }
                            }
                        }
                        // otherwise, if moving, start biting (and don't let the player get too close to the this NPC)
                        else if player.is_moving() {
                            if let Some(from) = player.set_state(EntityState::BITING, self.state.step) {
                                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                    model_ref: player.data.model_ref,
                                    from,
                                    to: player.data.state
                                }));
                            }
                        }

                        // make the player face the NPC
                        let player = &mut self.entities.players[i];
                        if !player.is_idle() {
                            let npc = &self.entities.npcs[npc_num];
                            let target_rot = Quaternion::from_axis_and_angle(
                                &UP,
                                npc.data.pos.sub(&player.data.pos).angle_xz()
                            );
                            let rot = player.data.rot.slerp(&target_rot, 0.1);
                            if rot != player.data.rot {
                                player.data.rot = rot;
                                self.events.push(GameEvent::EntityMoved(EntityMovedEvent{
                                    model_ref: player.data.model_ref,
                                    pos: player.data.pos,
                                    rot: player.data.rot,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "js"))]
    fn update_npcs(
        &mut self, 
        delta_time: f32,
        rng: &mut StdRng,
    ) {
        // 1st: update the state
        for npc in self.entities.npcs.iter_mut()
            .filter(|n| n.is_visible()) {
            if let Some((from, to)) = npc.update_state(self.state.step) {
                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                    model_ref: npc.data.model_ref,
                    from,
                    to
                }));

                if to == EntityState::DEAD {
                    npc.data.vis = EntityVisibility::HIDDEN;
                    self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
                        visible: false,
                        model_ref: npc.data.model_ref 
                    }));
                }
            }
        }
        
        // 2nd: check environment
        for i in 0..self.entities.npcs.len() {
            let npc = &self.entities.npcs[i];
            if npc.is_visible() && !npc.is_dead() {
                // any player close?
                if let Some((player_num, player, distance_sq)) = 
                    self.find_closest_player(&npc.data.pos, Some(3.0), rng) {
                    // close enough to be punched?
                    if distance_sq <= 1.0 * 2.0 {
                        // is the NPC already punching and enough time elapsed?
                        if npc.data.state == EntityState::PUNCHING &&
                            npc.state_elapsed(self.state.step) >= 0.75 {
                            
                            // stop punching
                            let npc = &mut self.entities.npcs[i];
                            if let Some(from) = npc.set_state(EntityState::IDLE, self.state.step) {
                                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                    model_ref: npc.data.model_ref,
                                    from,
                                    to: npc.data.state
                                }));
                            }

                            // don't hurt or move away the player if in training mode or the ANN will get confused
                            if !self.options.training {
                                let player = &mut self.entities.players[player_num];
                                player.health += consts::PENALTY_PUNCHED;
                                // player has no health left? kill it
                                if player.health <= 0.0 {
                                    if let Some(from) = player.set_state(EntityState::DYING, self.state.step) {
                                        self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                            model_ref: player.data.model_ref,
                                            from,
                                            to: player.data.state
                                        }));
                                    }
                                }
                                // otherwise, just change its state to hurting
                                else {
                                    if let Some(from) = player.set_state(EntityState::HIT, self.state.step) {
                                        self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                            model_ref: player.data.model_ref,
                                            from,
                                            to: player.data.state
                                        }));
                                    }

                                    // and move player away from NPC
                                    let inverse_rot = player.data.rot.mul(
                                        &Quaternion::from_axis_and_angle(&UP, 180.0 * PI / 180.0)
                                    );
                                    self.move_player(
                                        player_num, &inverse_rot, &Vector3::new(1.5, 1.5, 1.5), delta_time, true);
                                }
                            }
                        }
                        // otherwise, start punching if enough time elapsed
                        else if npc.data.state != EntityState::PUNCHING && 
                            npc.state_elapsed(self.state.step) >= 0.5 {
                            let npc = &mut self.entities.npcs[i];
                            if let Some(from) = npc.set_state(EntityState::PUNCHING, self.state.step) {
                                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                    model_ref: npc.data.model_ref,
                                    from,
                                    to: npc.data.state
                                }));
                            }
                        }
                    }
                    
                    // make the NPC face the closest player
                    let npc = &mut self.entities.npcs[i];
                    if !npc.is_idle() {
                        let target_rot = Quaternion::from_axis_and_angle(
                            &UP, player.data.pos.sub(&npc.data.pos).angle_xz());
                        let rot = npc.data.rot.slerp(&target_rot, 0.1);
                        if rot != npc.data.rot {
                            npc.data.rot = rot;
                            self.events.push(GameEvent::EntityMoved(EntityMovedEvent{
                                model_ref: npc.data.model_ref,
                                pos: npc.data.pos,
                                rot: npc.data.rot,
                            }));
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "js"))]
    fn update_objects(
        &mut self,
        _delta_time: f32,
        _rng: &mut StdRng,
    ) {
        let mut all_brains_collected = false;

        for player in &self.entities.players {
            if player.num_brains >= self.options.min_keys {
                all_brains_collected = true;
            }
        }

        if all_brains_collected {
            self.show_exit();
        }
    }
}