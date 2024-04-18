pub mod game;
pub mod collision_mng;
pub mod spawn_loc_mng;
pub mod asset;
pub mod sensor;

use neuralarena_cdk::models::instance::{
    InstancePlayerEntity,
    InstanceNpcEntity,
    InstanceEntities, 
    InstanceObjectEntity
};
use serde::{Deserialize, Serialize};
use crate::{entities::PlayerFOV, map::Map};
use collision_mng::CollisionAvoidanceManager;
use super::{GameEntities, GameEvent, GameState, MapInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptions {
    pub fov: PlayerFOV,
    pub num_players: usize,
    pub min_keys: usize,
    pub debug: bool,
    pub training: bool,
}

impl Into<InstanceEntities> for GameEntities {
    fn into(
        self
    ) -> InstanceEntities {
        InstanceEntities {
            players: self.players.iter().map(|p| InstancePlayerEntity {
                health: p.health,
                score: p.num_brains as _,
                xp: p.xp,
            }).collect(),
            npcs: self.npcs.iter().map(|n| InstanceNpcEntity {
                health: n.health,
            }).collect(),
            objects: self.objects.iter().map(|_| InstanceObjectEntity {
            }).collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    pub map_info: MapInfo,
    pub events: Vec<GameEvent>,
    pub entities: GameEntities,
    pub options: GameOptions,
    pub state: GameState,
    pub map: Map,
    pub collision: CollisionAvoidanceManager,
}

pub struct GameMap {
    pub map: &'static [u8],
    pub navmesh: &'static [u8],
}

pub const MAPS: &[GameMap] = &[
    GameMap {
        map: include_bytes!("../../../maps/0/map.glb"),
        navmesh: include_bytes!("../../../maps/0/navmesh.bin"),
    },
    GameMap {
        map: include_bytes!("../../../maps/1/map.glb"),
        navmesh: include_bytes!("../../../maps/1/navmesh.bin"),
    },
    GameMap {
        map: include_bytes!("../../../maps/2/map.glb"),
        navmesh: include_bytes!("../../../maps/2/navmesh.bin"),
    }
];
