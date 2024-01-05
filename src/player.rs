use std::f32::consts::PI;

use circular_buffer::CircularBuffer;
use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::{vector3::Vector3, path3::Path3, euler::Euler};
use crate::{env::EnvObs, object::{Object, ObjectVisibility, ObjectState, ObjectType}, game::GameOptions};

pub const NUM_SENSOR_FEATURES: usize = 1;

#[derive(EnumCount)]
pub enum PlayerAction {
    MoveForward,
    TurnRight,
    TurnLeft,
}

impl TryFrom<usize> for PlayerAction {
    fn try_from(v: usize) -> Result<Self, Self::Error> {
        if v == Self::MoveForward as usize {
            Ok(Self::MoveForward)
        } else if v == Self::TurnRight as usize {
            Ok(Self::TurnRight)
        } else if v == Self::TurnLeft as usize { 
            Ok(Self::TurnLeft)
        } else {
            Err("Unknown".to_string())
        }
    }

    type Error = String;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PlayerSensorTarget {
    pub similarity: f32,
    pub angle: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PlayerSensorHit {
    pub inter: Vector3,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlayerSensor {
    Target(PlayerSensorTarget),
    Hit(PlayerSensorHit),
}

impl Default for PlayerSensor {
    fn default() -> Self {
        PlayerSensor::Target(PlayerSensorTarget { similarity: 0.0, angle: 0.0 })
    }
}

#[derive(Clone, Default)]
pub struct PlayerHistory {
    pub dir: Euler,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub base: Object,
    pub dir: Euler,
    pub turn_angle: f32,
    pub max_vel: Vector3,
    pub vel: Vector3,
    pub accel: Vector3,
    pub num_keys: usize,
    pub health: f32,
    pub reward: f32,
    pub color: [u8; 3],
    #[serde(skip)]
    pub sensors: Vec<PlayerSensor>,
    #[serde(skip)]
    pub path: Option<Path3>,
    #[serde(skip)]
    pub history: CircularBuffer<10, PlayerHistory>,
}

impl Player {
    pub fn new(
        pos: &Vector3,
        dir: &Euler,
        options: &GameOptions,
        rng: &mut StdRng
    ) -> Player {
        let base = Object { 
            id: rng.gen::<usize>(),
            dim: Vector3::new(1.0, 1.0, 1.0),
            pos: pos.clone(),
            radius: 1.0,
            typ: ObjectType::PLAYER,
            state: ObjectState::IDLE,
            vis: ObjectVisibility::HIDDEN,
        };

        Self { 
            base,
            dir: dir.clone(),
            turn_angle: 15.0 * (PI / 180.0),
            vel: Vector3::zero(),
            max_vel: Vector3::new(1.0, 1.0, 0.0),
            accel: Vector3::new(0.1, 0.1, 0.0),
            num_keys: 0, 
            health: 100.0, 
            reward: 0.0,
            color: [
                127 + rng.gen_range(0..127),
                127 + rng.gen_range(0..127),
                127 + rng.gen_range(0..127)
            ],
            sensors: vec![
                PlayerSensor::default(); 
                options.fov.rays
            ],
            path: None,
            history: CircularBuffer::<10, PlayerHistory>::new(),
        }
    }
    
    pub fn scale(
        &self,
        value: f32
    ) -> Self {
        Self {
            base: self.base.scale(value),
            ..self.clone()
        }
    }
    
    pub fn mean_dir(
        &self
    ) -> Vector3 {
        let mut dir = Vector3::zero();
        for h in &self.history {
            dir.add_mut(&h.dir.v);
        }

        dir.mul_scalar(1.0 / (self.history.len() as f32).max(1.0))
    }
    
    pub fn to_obs(
        &self
    ) -> EnvObs {
        let mut obs = vec![];

        for s in &self.sensors {
            obs.push(match s {
                PlayerSensor::Target(s) => {
                    s.similarity
                },
                PlayerSensor::Hit(_) => {
                    0.0
                },
            });
        }

        obs
    }

    pub fn preserve(
        &mut self
    ) {
        self.history.push_back(PlayerHistory { 
            dir: self.dir.clone()
        });
    }
}
