use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use three_rs::math::euler::{Euler, EulerOrder};
use crate::{player::{Player, PlayerAction, NUM_SENSOR_FEATURES}, object::Object, enemy::Enemy, map::Map, env::{Env, EnvAction, EnvObs}, spawn_loc::SpawLocations};

const PLAYER_SPAWN_LOCATION_NAME: &str = "spl.";
const ENEMY_SPAWN_LOCATION_NAME: &str = "epl.";
const KEY_SPAWN_LOCATION_NAME: &str = "kpl.";
const EXIT_SPAWN_LOCATION_NAME: &str = "xpl.";

#[derive(PartialEq)]
pub enum GameStatus {
    IDLE,
    RUNNING,
    TIMEDOUT,
    FINISHED,
}

pub struct ObjectFOV {
    pub rays: usize,
    pub angle: f32,
    pub max_dist: f32,
}

pub struct GameState {
    pub status: GameStatus,
    pub winner: Option<usize>,
    pub step: usize,
}

pub struct MapInfo {
    pub num_keys: usize,
}

pub struct GameOptions {
    pub fov: ObjectFOV,
    pub fps: f32,
    pub num_players: usize,
    pub min_keys: usize,
    pub player_radius: f32,
    pub debug: bool,
    pub training: bool,
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
    pub rng: StdRng,
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
        &mut self
    ) -> EnvAction {
        self.rng.gen::<usize>() % self.get_num_actions()
    }

    fn reset(
        &mut self
    ) -> Vec<EnvObs> {
        
        let (objects, players, enemies, num_keys) = 
            Self::initialize(&self.map, &self.options, &mut self.rng);

        self.entities.objects = objects;
        self.entities.players = players;
        self.entities.enemies = enemies;

        self.map_info = MapInfo { 
            num_keys
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

    fn step(&mut self, actions: &Vec<Option<crate::env::EnvAction>>, delta_time: f32) -> crate::env::EnvStep {
        todo!()
    }

    fn render(&self, pov: &three_rs::math::vector3::Vector3, scale: f32, renderer: &mut dyn crate::renderer::Renderer) {
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
    fn initialize(
        map: &Map,
        options: &GameOptions,
        rng: &mut StdRng
    ) -> (
        Vec<Object>, 
        Vec<Player>, 
        Vec<Enemy>, 
        usize
    ) {
        let mut objects = vec![];
        let mut enemies = vec![];
        let mut num_keys = 0;
        
        let esp = Map::get_nodes_transform(map.find_nodes_by_partial_name(ENEMY_SPAWN_LOCATION_NAME));
        let ksp = Map::get_nodes_transform(map.find_nodes_by_partial_name(KEY_SPAWN_LOCATION_NAME));
        let xsp = Map::get_nodes_transform(map.find_nodes_by_partial_name(EXIT_SPAWN_LOCATION_NAME));

        let players = Self::spawn_players(options.num_players, map, options, rng);
        
        (objects, players, enemies, num_keys)
    }

    fn spawn_players(
        num: usize,
        map: &Map,
        options: &GameOptions,
        rng: &mut StdRng
    ) -> Vec<Player> {
        let psl = map.find_nodes_by_partial_name(PLAYER_SPAWN_LOCATION_NAME);
        let mut locs = SpawLocations::new(Map::get_nodes_transform(psl));


        let mut players = vec![];

        for i in 0..num {
            let loc = &locs.next(i, rng);
            let player = Player::new(&loc.0, &Euler::from_quartenion(&loc.1, EulerOrder::XYZ), options, rng);
            players.push(player);
        }

        players
    }
}