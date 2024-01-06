use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use three_rs::math::{vector3::Vector3, euler::{Euler, EulerOrder}, quaternion::Quaternion};
use crate::{player::{Player, PlayerAction, NUM_SENSOR_FEATURES}, object::{Object, ObjectType, ObjectState, ObjectVisibility}, enemy::Enemy, map::Map, env::{Env, EnvAction, EnvObs, EnvStep, EnvPlayer}, spawn_loc::SpawnLocations, renderer::Renderer, sensor::SensorReading};

const PLAYER_SPAWN_LOCATION_NAME: &str = "spl.";
const _ENEMY_SPAWN_LOCATION_NAME: &str = "epl.";
const KEY_SPAWN_LOCATION_NAME: &str = "kpl.";
const EXIT_SPAWN_LOCATION_NAME: &str = "xpl.";

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

#[derive(Clone, Serialize, Deserialize)]
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
                            Self::calc_dir_away_from_hit(player)
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
                                Euler::from_quartenion(
                                    &player.base.dir.q.rotate_z(-player.turn_angle),
                                    EulerOrder::XYZ
                                ).v,
                            PlayerAction::TurnLeft => 
                                Euler::from_quartenion(
                                    &player.base.dir.q.rotate_z(player.turn_angle),
                                    EulerOrder::XYZ
                                ).v
                        }
                    }
                };

                if dir.x != 0.0 || dir.y != 0.0 || dir.x != 0 {
                    let vel = player.vel.mul_scalar(delta_time).mul(&dir);
                    self.move_player(i, &dir, &vel);
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
        pov: &Vector3, 
        scale: f32, 
        renderer: &mut dyn Renderer
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
            options, 
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
        options: &GameOptions,
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
                Vector3::one(),
                options, 
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
                Vector3::one(),
                ObjectState::IDLE,
                if is_active {ObjectVisibility::VISIBLE} else {ObjectVisibility::HIDDEN},
                rng
            );
            objects.push(obj);
        }

        objects
    }

    fn find_closest_object(
        &self,
        pos: &Vector3,
        typ: ObjectType
    ) -> Object {

        let mut min_distance = f32::MAX;
        let mut res = Object::default();

        for obj in self.entities.objects.iter().filter(|o| 
            o.vis == ObjectVisibility::VISIBLE && o.typ == typ) {
            let distance = pos.sub(&obj.pos).length();
            if distance < min_distance {
                min_distance = distance;
                res = obj.clone();
            }
        }

        res
    }
}