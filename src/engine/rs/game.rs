use rand::{rngs::StdRng, Rng};
use std::{collections::{HashMap, HashSet}, f32::consts::PI};
use strum::EnumCount;
use three_rs::math::{
    path3::Path3,
    FORWARD, UP,
    Quaternion,
    Vector3
};
use neuralarena_cdk::models::{
    assets::{Asset, AssetRef}, 
    env::{Env, EnvAction, EnvObs, EnvPlayer, EnvPlayerOptions, EnvStep}, 
    game::{GameInfo, GameMapId}, 
    instance::{InstanceEntities, InstancePlayer}, 
    renderer::Renderer
};
use crate::{
    consts, 
    engine::{Game, GameEntities, GameOptions, GameState, GameStatus, MapInfo}, 
    entities::{
        Entity, EntityData, EntityState, EntityType, EntityVisibility, 
        Npc, 
        Object, ObjectType, 
        Player, PlayerAction, NUM_SENSOR_FEATURES
    }, 
    event::*, 
    map::Map
};
use super::{
    asset::{get_game_assets, GameAssetRef}, 
    collision_mng::CollisionAvoidanceManager, 
    spawn_loc_mng::SpawnLocManager,
    sensor::Sensor
};

impl Env<EntityState> for Game {
    fn get_assets(
        &self,
        game: Option<&GameInfo>,
        map_id: GameMapId,
        players: &Vec<Option<&InstancePlayer>>
    ) -> HashMap::<AssetRef, Asset> {
        get_game_assets(game, map_id, players)
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

    fn random_action(
        &mut self,
        rng: &mut StdRng
    ) -> EnvAction {
        rng.gen::<usize>() % self.get_num_actions()
    }

    fn reset(
        &mut self,
        players_options: &Vec<EnvPlayerOptions>,
        renderer: &mut dyn Renderer<EntityState>,
        rng: &mut StdRng,
    ) -> Vec<EnvObs> {
        renderer.reset();
        self.events = vec![];

        // map
        let level_ref = renderer.add_model(self.map_info.map_id.into())
            .expect(&format!("Could not load map {}", self.map_info.map_id));
        
        self.events.push(GameEvent::EntityCreated(EntityCreatedEvent {
            model_ref: level_ref,
            typ: EntityType::UNKNOWN,
            asset_ref: (GameAssetRef::Map0 as usize) + self.map_info.map_id,
        }));

        // entities
        self.entities = self.spawn_entities(players_options, renderer, rng);
        
        self.map_info = MapInfo { 
            map_id: self.map_info.map_id,
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

    fn step(
        &mut self, 
        actions: &Vec<(usize, Option<EnvAction>)>, 
        delta_time: f32,
        renderer: &mut dyn Renderer<EntityState>,
        rng: &mut StdRng,
    ) -> EnvStep {

        if self.state.status != GameStatus::RUNNING {
            return EnvStep {
                terminated: true,
                winner: self.state.winner.clone(),
                players: vec![],
            };
        }

        self.state.step += 1;

        self.update_players(actions, delta_time, rng);
        
        self.update_npcs(delta_time, rng);

        self.update_objects(delta_time, rng);
        
        self.build_players_sensors(renderer);

        EnvStep {
            terminated: self.state.status != GameStatus::RUNNING,
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

    fn serialize_events(
        &self
    ) -> Result<Vec<u8>, String> {
        rmp_serde::to_vec(&self.events)
            .map_err(|err| err.to_string())
    }

    fn serialize_entities(
        &self
    ) -> Result<Vec<u8>, String> {
        rmp_serde::to_vec::<InstanceEntities>(&self.entities.clone().into())
            .map_err(|err| err.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn render(
        &mut self,
        renderer: &mut dyn Renderer<EntityState>,
        delta_time: f32
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

        self.events = vec![];

        renderer.update(delta_time);

        if delta_time != 0.0 {
            renderer.render();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn render(
        &mut self,
        _renderer: &mut dyn Renderer<EntityState>,
        _delta_time: f32
    ) {
        self.events = vec![];
    }
}

impl Game {
    pub fn new(
        map: Map, 
        map_info: MapInfo,
        options: GameOptions
    ) -> Self {
        Self { 
            map,
            entities: GameEntities::default(),
            events: vec![],
            map_info,
            state: GameState { 
                status: GameStatus::IDLE, 
                winner: None, 
                step: 0,
                active_players: 0,
            },
            options,
            collision: CollisionAvoidanceManager::new(),
        }
    }

    fn spawn_entities(
        &mut self,
        players_options: &Vec<EnvPlayerOptions>,
        renderer: &mut dyn Renderer<EntityState>,
        rng: &mut StdRng
    ) -> GameEntities {
        // players
        let players = self.spawn_players(players_options, renderer, rng);

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

    fn spawn_players(
        &mut self,
        players_options: &Vec<EnvPlayerOptions>,
        renderer: &mut dyn Renderer<EntityState>,
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
                (GameAssetRef::Player0 as usize) + i
            ).unwrap();
            
            self.events.push(GameEvent::EntityCreated(EntityCreatedEvent {
                model_ref,
                typ: EntityType::PLAYER,
                asset_ref: (GameAssetRef::Player0 as usize) + i,
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

            let loc = &spawn_mng.next(rng);

            let index = players.len();

            let options = &players_options[index];

            let player = Player::new(
                index,
                loc.0, 
                loc.1, 
                options,
                EntityState::WAITING,
                EntityVisibility::VISIBLE,
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

    fn spawn_npcs(
        &mut self,
        renderer: &mut dyn Renderer<EntityState>,
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
                GameAssetRef::Npc0 as usize // + i
            ).unwrap();
            
            self.events.push(GameEvent::EntityCreated(EntityCreatedEvent {
                model_ref,
                typ: EntityType::PLAYER,
                asset_ref: GameAssetRef::Npc0 as usize //+ i,
            }));

            let loc = &spawn_locs.next(rng);
            let vis = if i == 0 {
                EntityVisibility::VISIBLE
            } else {
                EntityVisibility::HIDDEN
            };
            let index = npcs.len();

            let npc = Npc::new(
                index,
                loc.0, 
                loc.1, 
                Vector3::new(
                    consts::NPC_WIDTH, 
                    consts::NPC_HEIGHT, 
                    0.5
                ),
                consts::NPC_RADIUS,
                EntityState::WAITING,
                vis,
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

    fn spawn_objects(
        &mut self,
        typ: ObjectType,
        name: &str,
        active: &dyn Fn(usize) -> bool,
        renderer: &mut dyn Renderer<EntityState>,
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

        for i in 0..num_objs {
            let asset_ref = GameAssetRef::Exit as usize;
            
            let model_ref = renderer.add_model(asset_ref)
                .expect("Invalid asset");

            self.events.push(GameEvent::EntityCreated(EntityCreatedEvent {
                model_ref,
                typ: EntityType::OBJECT,
                asset_ref,
            }));
                
            let loc = &spawn_mng.next(rng);
            let is_active = active(i);
            let index = objects.len();

            let obj = Object::new(
                typ.clone(),
                index,
                loc.0, 
                loc.1, 
                Vector3::new(
                    consts::EXIT_RADIUS, 
                    consts::EXIT_RADIUS, 
                    0.2
                ),
                consts::EXIT_RADIUS,
                EntityState::WAITING,
                if is_active {
                    EntityVisibility::VISIBLE
                } else {
                    EntityVisibility::HIDDEN
                },
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

    fn show_exit(
        &mut self
    ) {
        let exit = self.entities.objects.iter_mut().find(|o|
            o.typ == ObjectType::EXIT && 
            !o.is_visible() && !o.is_dead_or_dying()
        );

        if let Some(obj) = exit {
            obj.data.vis = EntityVisibility::VISIBLE;

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

    fn show_next_npc(
        &mut self
    ) {
        if let Some(npc) = self.entities.npcs.iter_mut().find(|n|
            !n.is_visible() && !n.is_dead_or_dying()
        ) {
            npc.data.vis = EntityVisibility::VISIBLE;
            
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

    fn kill_player(
        &mut self, 
        num: usize
    ) {
        self.state.active_players -= 1;
        if self.state.active_players == 0 {
            self.state.status = GameStatus::FINISHED;
            self.state.winner = None;
        }

        let player = &mut self.entities.players[num];
                    
        player.data.vis = EntityVisibility::HIDDEN;
        self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
            visible: false,
            model_ref: player.data.model_ref 
        }));
    }

    fn kill_npc(
        &mut self, 
        num: usize
    ) {
        let npc = &mut self.entities.npcs[num];
        
        npc.data.vis = EntityVisibility::HIDDEN;
        self.events.push(GameEvent::EntityVisChanged(EntityVisChangedEvent {
            visible: false,
            model_ref: npc.data.model_ref 
        }));
    }

    fn find_closest_player_shuffled(
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
            if player.is_visible() && !player.is_dead_or_dying() {
                let distance_sq = pos.sub(&player.data.pos).length_sq();
                if distance_sq < min_distance_sq {
                    min_distance_sq = distance_sq;
                    res = Some((i, player.clone(), distance_sq));
                }
            }
        }

        res
    }

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

    fn find_closest_npc(
        &self,
        pos: &Vector3,
        radius: Option<f32>
    ) -> Option<(usize, f32)> {
        let mut min_distance_sq = if let Some(radius) = radius {radius * 2.0} else {f32::MAX};
        let mut res = None;

        for (i, npc) in self.entities.npcs.iter().enumerate().filter(|(_, n)| 
            n.is_visible() && !n.is_dead_or_dying()
        ) {
            let distance_sq = pos.sub(&npc.data.pos).length_sq();
            if distance_sq < min_distance_sq {
                min_distance_sq = distance_sq;
                res = Some((i, distance_sq));
            }
        }

        res
    }

    fn find_closest_object(
        &self,
        pos: &Vector3,
        typ: ObjectType
    ) -> EntityData {
        let mut min_distance = f32::MAX;
        let mut res = EntityData::default();

        for obj in self.entities.objects.iter().filter(|o| 
            o.typ == typ &&
            o.is_visible() && !o.is_dead_or_dying()
        ) {
            let distance = pos.sub(&obj.data.pos).length();
            if distance < min_distance {
                min_distance = distance;
                res = obj.data.clone();
            }
        }

        res
    }

    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    fn build_players_sensors(
        &mut self,
        renderer: &mut dyn Renderer<EntityState>
    ) {
        let mut readings = vec![(false, vec![]); self.entities.players.len()];
        let mut paths = vec![];

        for npc in &mut self.entities.npcs {
            npc.targeted_cnt = 0;
        }

        for (i, player) in self.entities.players.iter().enumerate() {
            let target_pos = if player.num_brains == self.options.min_keys { 
                self.find_closest_object(&player.data.pos, ObjectType::EXIT).pos
            }
            else {
                if let Some((npc_num, distance_sq)) = 
                    self.find_closest_npc(&player.data.pos, None) {
                    let npc = &mut self.entities.npcs[npc_num];
                    
                    if !self.options.training {
                        let rot = Quaternion::from_axis_and_angle(
                            &UP, (npc.targeted_cnt as f32 % 4.0) * 90.0 * PI / 180.0);

                        let around_pos = if distance_sq > 3.0 * 2.0 {
                            Vector3::new(0.0, 0.0, 2.0).apply_quaternion(&rot)
                        }
                        else if distance_sq > 2.0 * 2.0 {
                            Vector3::new(0.0, 0.0, 1.0).apply_quaternion(&rot)
                        }
                        else {
                            Vector3::zero()
                        };
                        
                        npc.targeted_cnt += 1;
                        npc.data.pos.add(&around_pos)
                    }
                    else {
                        npc.data.pos
                    }
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

            let target_pos = if path.nodes.len() > 0 {
                let pos = path.nodes[1].clone();

                if self.options.debug {
                    paths.push(path);
                }
    
                pos
            }
            else {
                paths.push(Path3::new(vec![
                    player.data.pos.clone(),
                    target_pos.clone()
                ]));
                
                target_pos.clone()
            };


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

    fn move_player(
        &mut self,
        num: usize,
        action: &Option<usize>,
        rot: &Quaternion,
        vel: &Vector3,
        delta_time: f32,
        forced: bool
    ) {
        let dir = FORWARD.apply_quaternion(&rot);
        
        let translation = dir.mul(vel)
            .mul_scalar(delta_time);

        let player = &self.entities.players[num];
        let next_pos = player.data.pos.add(&translation);
        
        if let Some(constrained_pos) = 
            self.map.move_along_surface(&player.data.pos, &next_pos) {            
            if constrained_pos.length_sq() != next_pos.length_sq() {
                let player = &mut self.entities.players[num];
                player.preserve();
                player.data.pos = constrained_pos
                    .add(&Vector3::new(0.0, -consts::NAVMESH_CELL_HEIGHT * 0.8, 0.0));
                if !forced {
                    player.action = action.into();
                    player.data.rot = player.calc_rot_away_from_hit();
                    player.reward += consts::PENALTY_WALL;
                }
            }
            else {
                let constrained_translation = if !self.options.training {
                    self.collision.move_player(
                        num, translation, &self.entities, delta_time)
                }
                else {
                    translation
                };

                let player = &mut self.entities.players[num];
                player.preserve();
                player.data.pos.add_mut(&constrained_translation);
                if !forced {
                    player.action = action.into();
                    player.data.rot = rot.clone();
                }
            }

            let player = &self.entities.players[num];
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

    fn check_exit(
        player: &Player,
        obj: &Object,
    ) -> bool {
        let player_bb = player.calc_bb();
        let item_bb = obj.calc_bb();

        player_bb.intersects_with_box(&item_bb)
    }

    fn update_players(
        &mut self, 
        actions: &Vec<(usize, Option<usize>)>, 
        delta_time: f32,
        _rng: &mut StdRng,
    ) {
        // 1st: update state
        let mut killed_idxs = vec![];
        for player in self.entities.players.iter_mut()
            .filter(|p| p.is_visible()) {
            if let Some((from, to)) = player.update_state(self.state.step) {
                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                    model_ref: player.data.model_ref,
                    from,
                    to
                }));

                if to == EntityState::DEAD {
                    killed_idxs.push(player.data.index);
                }
            }
        }

        for num in killed_idxs {
            self.kill_player(num);
        }

        // 2nd: move player
        for (p, action) in actions {
            let i = *p;

            let player = &mut self.entities.players[i];
            if player.is_visible() && !player.is_dead_or_dying() {
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
                    self.move_player(i, action, &rot, &vel, delta_time, false);
                }
            }
        }

        // 3rd: check environment
        for (p, _) in actions {
            let i = *p;

            let player = &self.entities.players[i];
            if player.is_visible() && !player.is_dead_or_dying() {
                // all brains eaten? check if exit reached
                if player.num_brains >= self.options.min_keys {
                    let exit = self.entities.objects.iter().find(|o| o.typ == ObjectType::EXIT);
                    if let Some(exit) = exit {
                        if Self::check_exit(player, exit) {
                            self.state.status = GameStatus::FINISHED;
                            self.state.winner = Some(i);
                            
                            let player = &mut self.entities.players[i];
                            player.reward += consts::REWARD_EXIT_REACHED;
                            player.xp += 1;
                            
                            if let Some(from) = player.set_state(EntityState::CELEBRATING, self.state.step) {
                                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                    model_ref: player.data.model_ref,
                                    from,
                                    to: player.data.state
                                }));
                            }

                            // all other players player should become idle
                            for j in 0..self.entities.players.len() {
                                if j != i {
                                    let player = &mut self.entities.players[j];
                                    if let Some(from) = player.set_state(EntityState::IDLE, self.state.step) {
                                        self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                            model_ref: player.data.model_ref,
                                            from,
                                            to: player.data.state
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
                // check the map..
                else {
                    // any npc close?
                    if let Some((npc_num, _)) = 
                        self.find_closest_npc(&player.data.pos, Some(player.data.radius)) {

                        let player = &mut self.entities.players[i];
                        //if moving, start attacking (and, at same time, don't let the player get too close to the this NPC)
                        if player.is_moving() {
                            player.reward += consts::REWARD_ATTACK_HIT;
    
                            if let Some(from) = player.set_state(EntityState::ATTACKING, self.state.step) {
                                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                    model_ref: player.data.model_ref,
                                    from,
                                    to: player.data.state
                                }));
                            }

                            let npc = &mut self.entities.npcs[npc_num];
                            npc.health += consts::PENALTY_HIT;
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
                            else {
                                if npc.data.state != EntityState::HIT && 
                                    (npc.data.state == EntityState::IDLE || npc.state_elapsed(self.state.step) >= 0.75) {
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

    fn update_npcs(
        &mut self, 
        delta_time: f32,
        rng: &mut StdRng,
    ) {
        // 1st: update the state
        let mut killed_idxs = vec![];
        for npc in self.entities.npcs.iter_mut()
            .filter(|n| n.is_visible()) {
            if let Some((from, to)) = npc.update_state(self.state.step) {
                self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                    model_ref: npc.data.model_ref,
                    from,
                    to
                }));

                if to == EntityState::DEAD {
                    killed_idxs.push(npc.data.index);
                }
            }
        }

        for num in killed_idxs {
            self.kill_npc(num);
        }
        
        // 2nd: check environment
        for i in 0..self.entities.npcs.len() {
            let npc = &self.entities.npcs[i];
            if npc.is_visible() && !npc.is_dead_or_dying() {
                // any player close enough?
                if let Some((player_num, player, distance_sq)) = 
                    self.find_closest_player_shuffled(&npc.data.pos, Some(3.0), rng) {
                    // close enough to be punched?
                    if distance_sq <= npc.data.radius * 2.0 {
                        // if npc is not punching, begin doing that if enough time elapsed since the last state started
                        if npc.data.state != EntityState::PUNCHING && 
                            npc.state_elapsed(self.state.step) >= 0.75 {
                            let npc = &mut self.entities.npcs[i];
                            if let Some(from) = npc.set_state(EntityState::PUNCHING, self.state.step) {
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
                                else {
                                    if player.data.state != EntityState::HIT && 
                                        (player.data.state == EntityState::IDLE || player.state_elapsed(self.state.step) >= 0.75) {
                                        if let Some(from) = player.set_state(EntityState::HIT, self.state.step) {
                                            self.events.push(GameEvent::EntityStateChanged(EntityStateChangedEvent {
                                                model_ref: player.data.model_ref,
                                                from,
                                                to: player.data.state
                                            }));
                                        }
                                    }
                                }

                                // and move player away from NPC
                                let inverse_rot = player.data.rot.mul(
                                    &Quaternion::from_axis_and_angle(&UP, 180.0 * PI / 180.0)
                                );
                                self.move_player(
                                    player_num, 
                                    &None, 
                                    &inverse_rot, 
                                    &Vector3::new(consts::PLAYER_VELOCITY, 0.0, consts::PLAYER_VELOCITY),
                                    delta_time, 
                                    true
                                );
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