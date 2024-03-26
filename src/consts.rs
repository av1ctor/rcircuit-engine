use neuralarena_cdk::models::game::GameMapId;

pub const FPS: usize = 30;
pub const ANIM_FPS: usize = 60;
pub const INTERVAL: f32 = 1000.0 / FPS as f32;

pub const MIN_PLAYERS: usize = 1;
pub const MAX_PLAYERS: usize = 10;

pub const PLAYER_SPAWN_LOCATION_NAME: &str = "psl.";
pub const NPC_SPAWN_LOCATION_NAME: &str = "ksl.";
pub const EXIT_SPAWN_LOCATION_NAME: &str = "xsl.";

pub const PENALTY_WALL: f32 = -10.0;
pub const PENALTY_HIT: f32 = -20.0;
pub const PENALTY_PUNCHED: f32 = PENALTY_HIT / 5.0;
pub const REWARD_ATTACK_HIT: f32 = 100.0;
pub const REWARD_EXIT_REACHED: f32 = 1000.0;

pub const PLAYER_WIDTH: f32 = 1.0;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_TURNING_ANGLE: f32 = 6.0 * (ANIM_FPS as f32 / FPS as f32);
pub const PLAYER_VELOCITY: f32 = 0.1 * (ANIM_FPS as f32 / FPS as f32);

pub const NPC_WIDTH: f32 = 1.0;
pub const NPC_HEIGHT: f32 = 1.8;

pub const EXIT_WIDTH: f32 = 1.0;
pub const EXIT_HEIGHT: f32 = 3.0;

// maps
pub const MAP_ID_CONTAINERS: GameMapId = 0;
pub const NUM_MAPS: usize = 1;