use serde::{Serialize, Deserialize};
use neuralarena_cdk::models::game::GameMapId;
use crate::{
    entities::{Npc, Object, Player}, 
    event::*
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GameEntities {
    pub players: Vec<Player>,
    pub npcs: Vec<Npc>,
    pub objects: Vec<Object>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MapInfo {
    pub map_id: GameMapId,
    pub num_keys: usize,
}

#[cfg(not(feature = "js"))]
pub mod rs;
#[cfg(feature = "js")]
pub mod js;

#[cfg(not(feature = "js"))]
pub use rs::*;
#[cfg(feature = "js")]
pub use js::*;
