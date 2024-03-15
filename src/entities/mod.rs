use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumCount, Default)]
#[repr(C)]
pub enum EntityType {
    #[default]
    UNKNOWN = 0,
    PLAYER  = 1,
    NPC     = 2,
    OBJECT  = 3,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Serialize, Deserialize, EnumCount, Default)]
#[repr(C)]
pub enum EntityState {
    #[default]
    WAITING     = 0,
    IDLE        = 1,
    WALKING     = 2,
    ATTACKING   = 3,
    CELEBRATING = 4,
    PUNCHING    = 5,
    HIT         = 6,
    DYING       = 7,
    DEAD        = 8,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
#[repr(C)]
pub enum EntityVisibility {
    #[default]
    HIDDEN  = 0,
    VISIBLE = 1,
}


#[cfg(not(feature = "js"))]
pub mod rs;
#[cfg(feature = "js")]
pub mod js;

#[cfg(not(feature = "js"))]
pub use rs::*;
#[cfg(feature = "js")]
pub use js::*;
