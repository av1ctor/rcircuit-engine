use serde::{Serialize, Deserialize};
use neuralarena_cdk::models::game::GameMapId;
#[cfg(not(feature = "js"))]
use neuralarena_cdk::models::instance::{
    InstancePlayerEntity,
    InstanceNpcEntity,
    InstanceEntities, 
    InstanceObjectEntity
};
use crate::{
    entities::{
        Npc, Object, Player
    }, 
    event::*, 
    map::Map 
};
#[cfg(not(feature = "js"))]
use crate::{
    entities::PlayerFOV, 
    collision_mng::CollisionAvoidanceManager, 
};

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
    pub map_id: GameMapId,
    pub num_keys: usize,
}

#[cfg(not(feature = "js"))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptions {
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

#[cfg(not(feature = "js"))]
impl Into<InstanceEntities> for GameEntities {
    fn into(
        self
    ) -> InstanceEntities {
        InstanceEntities {
            players: self.players.iter().map(|p| InstancePlayerEntity {
                health: p.health / 100.0,
                score: p.num_brains as _,
            }).collect(),
            npcs: self.npcs.iter().map(|n| InstanceNpcEntity {
                health: n.health / 100.0,
            }).collect(),
            objects: self.objects.iter().map(|_| InstanceObjectEntity {
            }).collect(),
        }
    }
}

#[cfg_attr(not(feature = "js"), derive(Clone, Serialize, Deserialize))]
pub struct Game {
    pub map_info: MapInfo,
    pub events: Vec<GameEvent>,
    pub entities: GameEntities,
    #[cfg(not(feature = "js"))]
    pub options: GameOptions,
    pub state: GameState,
    pub map: Map,
    #[cfg(not(feature = "js"))]
    pub collision: CollisionAvoidanceManager,
}

#[cfg(not(feature = "js"))]
pub struct GameMap {
    pub map: &'static [u8],
    pub navmesh: &'static [u8],
}

#[cfg(not(feature = "js"))]
pub const MAPS: &[GameMap] = &[
    GameMap {
        map: include_bytes!("../../maps/0/map.glb"),
        navmesh: include_bytes!("../../maps/0/navmesh.bin"),
    }
];

#[cfg(not(feature = "js"))]
pub mod rs;
#[cfg(feature = "js")]
pub mod js;
