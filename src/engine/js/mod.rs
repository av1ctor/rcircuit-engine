use super::{GameEntities, GameEvent};

pub mod game;
pub mod wrapper;

pub struct Game {
    pub events: Vec<GameEvent>,
    pub entities: GameEntities,
}
