pub const MAX_PLAYERS: usize = 8;

pub const PLAYER_SPAWN_LOCATION_NAME: &str = "psl.";
pub const NPC_SPAWN_LOCATION_NAME: &str = "ksl.";
pub const EXIT_SPAWN_LOCATION_NAME: &str = "xsl.";

pub const PENALTY_WALL: f32 = -10.0;
pub const PENALTY_BITTEN: f32 = -20.0;
pub const PENALTY_PUNCHED: f32 = PENALTY_BITTEN / 5.0;
pub const REWARD_BRAIN_EATEN: f32 = 100.0;
pub const REWARD_EXIT_REACHED: f32 = 1000.0;

pub const PLAYER_WIDTH: f32 = 1.0;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_TURNING_ANGLE: f32 = 45.0;

pub const NPC_WIDTH: f32 = 1.0;
pub const NPC_HEIGHT: f32 = 1.8;

pub const EXIT_WIDTH: f32 = 1.0;
pub const EXIT_HEIGHT: f32 = 3.0;
