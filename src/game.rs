#[cfg(not(feature = "js"))]
use rand::{rngs::StdRng, Rng};
#[cfg(not(feature = "js"))]
use std::collections::HashMap;
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
    npc::Npc, 
    env::{Env, EnvObs}, 
    event::*, 
    map::Map, 
    object::Object, 
    player::{Player, PlayerAction, NUM_SENSOR_FEATURES}
};
#[cfg(not(feature = "js"))]
use crate::{
    asset::{GameAssetRef, get_game_assets}, 
    env::{Asset, AssetRef, EnvAction, EnvStep, EnvPlayer},
    object::{ObjectState, ObjectVisibility, ObjectType}, 
    spawn_locs::SpawnLocations, 
    sensor::Sensor
};

#[cfg(not(feature = "js"))]
const PLAYER_SPAWN_LOCATION_NAME: &str = "psl.";
#[cfg(not(feature = "js"))]
const NPC_SPAWN_LOCATION_NAME: &str = "ksl.";
#[cfg(not(feature = "js"))]
const EXIT_SPAWN_LOCATION_NAME: &str = "xsl.";

#[cfg(not(feature = "js"))]
const PENALTY_WALL: f32 = -10.0;
#[cfg(not(feature = "js"))]
const PENALTY_HIT: f32 = -20.0;
#[cfg(not(feature = "js"))]
const REWARD_BRAIN_EATEN: f32 = 100.0;
#[cfg(not(feature = "js"))]
const REWARD_EXIT_REACHED: f32 = 1000.0;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum GameStatus {
    IDLE,
    RUNNING,
    TIMEDOUT,
    FINISHED,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ObjectFOV {
    pub rays: usize,
    pub angle: f32,
    pub max_dist: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub status: GameStatus,
    pub winner: Option<usize>,
    pub step: usize,
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
    pub fov: ObjectFOV,
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

#[cfg_attr(not(feature = "js"), derive(Clone, Debug, Serialize, Deserialize))]
pub struct Game {
    pub map_info: MapInfo,
    pub events: Vec<GameEvent>,
    pub entities: GameEntities,
    pub options: GameOptions,
    pub state: GameState,
    pub map: Map,
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
        
        self.events.push(GameEvent::ObjectCreated(ObjectCreatedEvent {
            model_ref: level_ref,
            typ: ObjectType::UNKNOWN,
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
        
            self.update_players(actions, delta_time);
            
            self.update_npcs(delta_time);

            self.update_objects(delta_time);
            
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

    #[cfg(all(not(feature = "js"), not(target_arch = "wasm32")))]
    fn render(
        &mut self,
        renderer: &mut dyn Renderer,
        events_only: bool
    ) {
        for event in &self.events {
            match event {
                GameEvent::ObjectCreated(_) => {
                    // nothing to do, the object was created already
                },
                
                GameEvent::ObjectSpawned(e) => {
                    renderer.set_position(
                        e.model_ref, &e.pos
                    );
                    renderer.set_rotation(
                        e.model_ref, &e.rot
                    );
                },
                
                GameEvent::ObjectMoved(e) => {
                    renderer.set_position(
                        e.model_ref, &e.pos
                    );
                    renderer.set_rotation(
                        e.model_ref, &e.rot
                    );
                },

                GameEvent::ObjectStateChanged(e) => {
                    renderer.change_state(e.model_ref, e.from, e.to);
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
                GameEvent::ObjectCreated(e) => {
                    renderer.add_model(e.asset_ref, e.model_ref);
                },
                
                GameEvent::ObjectSpawned(e) => {
                    renderer.set_position(
                        e.model_ref, &e.pos.to_slice()
                    );
                    renderer.set_rotation(
                        e.model_ref, &e.rot.to_slice()
                    );
                },
                
                GameEvent::ObjectMoved(e) => {
                    renderer.set_position(
                        e.model_ref, &e.pos.to_slice()
                    );
                    renderer.set_rotation(
                        e.model_ref, &e.rot.to_slice()
                    );
                },

                GameEvent::ObjectStateChanged(e) => {
                    renderer.change_state(e.model_ref, e.from as u32, e.to as u32);
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
            },
            options,
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
            EXIT_SPAWN_LOCATION_NAME,
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
            let model_ref = renderer.add_model(
                (GameAssetRef::Player1 as usize) + i
            ).unwrap();
            
            self.events.push(GameEvent::ObjectCreated(ObjectCreatedEvent {
                model_ref,
                typ: ObjectType::PLAYER,
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

            let loc = &spawn_locs.next(i, rng);
            let player = Player::new(
                loc.0, 
                loc.1, 
                Vector3::new(0.75, 1.8, 0.5), //FIXME!!!
                ObjectState::IDLE,
                ObjectVisibility::VISIBLE,
                model_ref,
                path_ref,
                sensor_ref,
                readings_ref,
                rng
            );

            self.events.push(GameEvent::ObjectSpawned({
                ObjectSpawnedEvent {
                    model_ref,
                    pos: loc.0,
                    rot: loc.1,
                }
            }));

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
    fn spawn_npcs(
        &mut self,
        renderer: &mut dyn Renderer,
        rng: &mut StdRng
    ) -> Vec<Npc> {
        let mut spawn_locs = SpawnLocations::new(
            Map::get_nodes_transform(
                self.map.find_nodes_by_partial_name(NPC_SPAWN_LOCATION_NAME)
            )
        );

        if spawn_locs.locs.len() == 0 {
            panic!("No spawn locations '{}' for npcs found", NPC_SPAWN_LOCATION_NAME);
        }

        let mut npcs = vec![];
        let num_npcs = self.options.num_players * self.options.min_keys;

        for i in 0..num_npcs {
            let model_ref = renderer.add_model(
                GameAssetRef::Npc1 as usize // + i
            ).unwrap();
            
            self.events.push(GameEvent::ObjectCreated(ObjectCreatedEvent {
                model_ref,
                typ: ObjectType::PLAYER,
                asset_ref: GameAssetRef::Npc1 as usize //+ i,
            }));

            let loc = &spawn_locs.next(i, rng);
            let vis = if i == 0 {ObjectVisibility::VISIBLE} else {ObjectVisibility::HIDDEN};
            let npc = Npc::new(
                loc.0, 
                loc.1, 
                Vector3::new(0.75, 1.8, 0.5), //FIXME!!!
                ObjectState::IDLE,
                vis,
                model_ref,
                rng
            );

            self.events.push(GameEvent::ObjectSpawned(
                ObjectSpawnedEvent {
                    model_ref,
                    pos: loc.0,
                    rot: loc.1,
                }
            ));

            if vis == ObjectVisibility::VISIBLE {
                self.events.push(GameEvent::ObjectShown(
                    ObjectShownEvent {
                        model_ref,
                    }
                ));
            }
            else {
                self.events.push(GameEvent::ObjectHidden(
                    ObjectHiddenEvent {
                        model_ref,
                    }
                ));
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
        let mut spawn_locs = SpawnLocations::new(
            Map::get_nodes_transform(
                self.map.find_nodes_by_partial_name(name)
            )
        );

        if spawn_locs.locs.len() == 0 {
            panic!("No spawn locations '{}' for objects found", name);
        }

        let mut objects = vec![];
        let num_objs = 1;

        let rad = 1.0;

        for i in 0..num_objs {
            let asset_ref = GameAssetRef::Exit as usize;
            
            let model_ref = renderer.add_model(asset_ref)
                .expect("Invalid asset");

            self.events.push(GameEvent::ObjectCreated(ObjectCreatedEvent {
                model_ref,
                typ: typ.clone(),
                asset_ref,
            }));
                
            let loc = &spawn_locs.next(i, rng);
            let is_active = active(i);
            let obj = Object::new(
                typ.clone(),
                loc.0, 
                loc.1, 
                Vector3::new(rad, rad, rad),
                ObjectState::IDLE,
                if is_active {ObjectVisibility::VISIBLE} else {ObjectVisibility::HIDDEN},
                model_ref,
                rng
            );

            self.events.push(GameEvent::ObjectSpawned({
                ObjectSpawnedEvent {
                    model_ref,
                    pos: loc.0,
                    rot: loc.1,
                }
            }));

            if is_active {
                self.events.push(GameEvent::ObjectShown(ObjectShownEvent {
                    model_ref
                }));
            }
            else {
                self.events.push(GameEvent::ObjectHidden(ObjectHiddenEvent {
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
            o.vis == ObjectVisibility::HIDDEN && 
            o.state != ObjectState::DEAD
        );
        if let Some(obj) = exit {
            obj.vis = ObjectVisibility::VISIBLE;
            
            self.events.push(GameEvent::ObjectMoved(ObjectMovedEvent{
                model_ref: obj.model_ref,
                pos: obj.pos,
                rot: obj.rot,
            }));
            
            self.events.push(GameEvent::ObjectShown(ObjectShownEvent {
                model_ref: obj.model_ref
            }));
        }
    }

    #[cfg(not(feature = "js"))]
    fn show_next_npc(
        &mut self
    ) {
        if let Some(npc) = self.entities.npcs.iter_mut().find(|n|
            n.base.vis == ObjectVisibility::HIDDEN && 
            n.base.state != ObjectState::DEAD
        ) {
            npc.base.vis = ObjectVisibility::VISIBLE;
            
            self.events.push(GameEvent::ObjectMoved(
                ObjectMovedEvent {
                    model_ref: npc.base.model_ref,
                    pos: npc.base.pos,
                    rot: npc.base.rot,
                }
            ));
            
            self.events.push(GameEvent::ObjectShown(ObjectShownEvent {
                model_ref: npc.base.model_ref
            }));
        }
    }

    #[cfg(not(feature = "js"))]
    fn find_closest_player(
        &self,
        pos: &Vector3,
        radius: f32
    ) -> Option<(usize, Player)> {

        let mut min_distance_sq = radius * 2.0;
        let mut res = None;

        for (num, player) in self.entities.players.iter().enumerate().filter(|(_, p)| 
            p.base.vis == ObjectVisibility::VISIBLE && 
            p.base.state != ObjectState::DEAD
        ) {
            let distance_sq = pos.sub(&player.base.pos).length_sq();
            if distance_sq < min_distance_sq {
                min_distance_sq = distance_sq;
                res = Some((num, player.clone()));
            }
        }

        res
    }

    #[cfg(not(feature = "js"))]
    fn find_closest_npc(
        &self,
        pos: &Vector3
    ) -> Object {

        let mut min_distance = f32::MAX;
        let mut res = Object::default();

        for npc in self.entities.npcs.iter().filter(|n| 
            n.base.vis == ObjectVisibility::VISIBLE && 
            n.base.state != ObjectState::DEAD
        ) {
            let distance = pos.sub(&npc.base.pos).length();
            if distance < min_distance {
                min_distance = distance;
                res = npc.base.clone();
            }
        }

        res
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
            let target_obj = if player.num_brains == self.options.min_keys { 
                self.find_closest_object(&player.base.pos, ObjectType::EXIT)
            }
            else {
                self.find_closest_npc(&player.base.pos)
            };

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
                let (_sensor_vtx, readings_vtx) = Player::gen_vis_sensor_readings(
                    &player.base.pos, 
                    &readings[i].1,
                    self.options.fov.max_dist
                );
                
                /*renderer.set_vertices(
                    player.sensor_ref, 
                    sensor_vtx  
                );*/
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
        rot: &Quaternion,
        vel: &Vector3,
        delta_time: f32
    ) {
        let player = &mut self.entities.players[num];
        
        let dir = FORWARD.apply_quaternion(&rot);
        
        let new_pos = player.base.pos.add(&dir
            .mul(vel)
            .mul_scalar(delta_time)
        );
        
        if let Some(confined_pos) = 
            self.map.move_along_surface(&player.base.pos, &new_pos) {
            player.preserve();
            if confined_pos.x != new_pos.x || confined_pos.z != new_pos.z {
                player.base.pos = confined_pos.mul(&Vector3::new(1.0, 0.0, 1.0));
                player.base.rot = player.calc_rot_away_from_hit();
                player.reward += PENALTY_WALL;
            }
            else {
                player.base.pos = new_pos;
                player.base.rot = rot.clone();
            }

            let mean_rot = player.mean_rot();

            self.events.push(GameEvent::ObjectMoved(
                ObjectMovedEvent {
                    model_ref: player.base.model_ref,
                    pos: player.base.pos,
                    rot: mean_rot,
                }
            ));
        }
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
    fn update_players(
        &mut self, 
        actions: &Vec<Option<usize>>, 
        delta_time: f32
    ) {
        for player in &mut self.entities.players {
            if let Some((from, to)) = player.update_state(self.state.step) {
                self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                    model_ref: player.base.model_ref,
                    from,
                    to
                }));
            }
        }

        for i in 0..actions.len() {
            let player = &mut self.entities.players[i];
            player.reward = 0.0;

            if player.is_moving() {
                let rot = match actions[i] {
                    None => {
                        player.base.rot.clone()
                    },
                    Some(action) => {
                        match PlayerAction::try_from(action.clone()).unwrap() {
                            PlayerAction::MoveForward =>
                                player.base.rot.clone(),
                            PlayerAction::TurnRight => {
                                player.base.rot.mul(
                                    &Quaternion::from_axis_and_angle(&UP, player.turn_angle)
                                )
                            },
                            PlayerAction::TurnLeft => {
                                player.base.rot.mul(
                                    &Quaternion::from_axis_and_angle(&UP, -player.turn_angle)
                                )
                            }
                        }
                    }
                };

                let vel = player.vel.clone();
                self.move_player(i, &rot, &vel, delta_time);
            }
        }

        for i in 0..self.entities.players.len() {
            let player = &self.entities.players[i];
            if player.num_brains >= self.options.min_keys {
                let door = self.entities.objects.iter().find(|o| o.typ == ObjectType::EXIT);
                if let Some(door) = door {
                    if Self::check_exit(&player.base, door) {
                        self.state.status = GameStatus::FINISHED;
                        self.state.winner = Some(i);
                        
                        let player = &mut self.entities.players[i];
                        player.reward += REWARD_EXIT_REACHED;
                        
                        player.set_state(ObjectState::CELEBRATING, self.state.step);
                        self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                            model_ref: player.base.model_ref,
                            from: player.base.state,
                            to: ObjectState::CELEBRATING
                        }));
                    }
                }
            }
            else {
                if self.state.status == GameStatus::FINISHED {
                    let player = &mut self.entities.players[i];
                    player.set_state(ObjectState::IDLE, self.state.step);
                    self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                        model_ref: player.base.model_ref,
                        from: player.base.state,
                        to: ObjectState::IDLE
                    }));
                }
            }
        }
    }

    #[cfg(not(feature = "js"))]
    fn update_npcs(
        &mut self, 
        delta_time: f32
    ) {
        for npc in &mut self.entities.npcs {
            if let Some((from, to)) = npc.update_state(self.state.step) {
                self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                    model_ref: npc.base.model_ref,
                    from,
                    to
                }));
            }
        }
        
        for i in 0..self.entities.npcs.len() {
            let npc = &self.entities.npcs[i];
            if npc.base.state != ObjectState::DEAD &&
                npc.base.vis == ObjectVisibility::VISIBLE {
                // any player around?
                if let Some((player_num, player)) = self.find_closest_player(&npc.base.pos, 5.0) {
                    let player_bb = player.calc_bb();
                    let npc_bb = npc.calc_bb();
                    if player_bb.intersects_with_box(&npc_bb) {
                        let player = &mut self.entities.players[player_num];
                        if npc.base.state == ObjectState::PUNCHING {
                            let npc = &mut self.entities.npcs[i];
                            npc.set_state(ObjectState::IDLE, self.state.step);
                            self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                                model_ref: npc.base.model_ref,
                                from: npc.base.state,
                                to: ObjectState::IDLE
                            }));

                            player.health += PENALTY_HIT;
                            if player.health <= 0.0 {
                                player.set_state(ObjectState::DEAD, self.state.step);
                                self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                                    model_ref: player.base.model_ref,
                                    from: player.base.state,
                                    to: ObjectState::DEAD
                                }));
                                
                                player.base.vis = ObjectVisibility::HIDDEN;
                                self.events.push(GameEvent::ObjectHidden(ObjectHiddenEvent { 
                                    model_ref: player.base.model_ref 
                                }));
                            }
                            else {
                                player.set_state(ObjectState::HURTING, self.state.step);
                                self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                                    model_ref: player.base.model_ref,
                                    from: player.base.state,
                                    to: ObjectState::HURTING
                                }));

                                let rot = player.base.rot.inverse();
                                let vel = player.vel.mul_scalar(10.0);
                                self.move_player(player_num, &rot, &vel, delta_time);
                            }
                        }
                        else {
                            player.reward += REWARD_BRAIN_EATEN;
                            
                            if player.base.state != ObjectState::EATING {
                                player.set_state(ObjectState::EATING, self.state.step);
                                self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                                    model_ref: player.base.model_ref,
                                    from: player.base.state,
                                    to: ObjectState::EATING
                                }));
                            }
                        
                            let npc = &mut self.entities.npcs[i];
                            
                            npc.health += PENALTY_HIT;
                            if npc.health <= 0.0 {
                                npc.set_state(ObjectState::DEAD, self.state.step);
                                self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                                    model_ref: npc.base.model_ref,
                                    from: npc.base.state,
                                    to: ObjectState::DEAD
                                }));

                                npc.base.vis = ObjectVisibility::HIDDEN;
                                self.events.push(GameEvent::ObjectHidden(ObjectHiddenEvent { 
                                    model_ref: npc.base.model_ref 
                                }));

                                player.num_brains += 1;
                                if player.num_brains == self.options.min_keys {
                                    self.show_exit();
                                } 
                                
                                self.show_next_npc();
                            }
                            else {
                                if npc.base.state != ObjectState::HURTING {
                                    npc.set_state(ObjectState::HURTING, self.state.step);
                                    self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                                        model_ref: npc.base.model_ref,
                                        from: npc.base.state,
                                        to: ObjectState::HURTING
                                    }));
                                }
                            }
                        }
                    }
                    else {
                        let distance_sq = npc.base.pos.sub(&player.base.pos).length_sq();
                        if distance_sq < 1.0 * 2.0 {
                            if npc.base.state != ObjectState::PUNCHING && 
                                !(npc.base.state == ObjectState::IDLE && self.state.step - npc.base.state_changed_at < 30) {
                                let npc = &mut self.entities.npcs[i];
                                npc.set_state(ObjectState::PUNCHING, self.state.step);
                                self.events.push(GameEvent::ObjectStateChanged(ObjectStateChangedEvent {
                                    model_ref: player.base.model_ref,
                                    from: player.base.state,
                                    to: ObjectState::PUNCHING
                                }));
                            }
                        }
                        else {

                        }
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "js"))]
    fn update_objects(
        &mut self,
        _delta_time: f32
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