use std::f32::consts::PI;
use circular_buffer::CircularBuffer;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::{
    vector3::{Vector3, FORWARD}, 
    box3::Box3
};
use crate::{
    env::EnvObs, 
    object::{Object, ObjectVisibility, ObjectState, ObjectType}, 
    sensor::{Sensor, SensorReading, SensorReadingHit, SensorReadingTarget}, renderer::ModelRef, game::ObjectFOV, map::Map
};

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
        PlayerSensor::Target(PlayerSensorTarget { 
            similarity: 0.0, 
            angle: 0.0 
        })
    }
}

#[derive(Clone, Default)]
pub struct PlayerHistory {
    pub dir: Vector3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub base: Object,
    pub turn_angle: f32,
    pub max_vel: Vector3,
    pub vel: Vector3,
    pub num_keys: usize,
    pub health: f32,
    pub reward: f32,
    #[serde(skip)]
    pub path_ref: ModelRef,
    #[serde(skip)]
    pub sensor_ref: ModelRef,
    #[serde(skip)]
    pub sensor: Sensor,
    #[serde(skip)]
    pub history: CircularBuffer<10, PlayerHistory>,
}

impl Player {
    pub fn new(
        pos: Vector3,
        dir: Vector3,
        dim: Vector3,
        model_ref: ModelRef,
        path_ref: ModelRef,
        sensor_ref: ModelRef,
        rng: &mut StdRng
    ) -> Player {
        let base = Object::new(
            ObjectType::PLAYER,
            pos,
            dir,
            dim,
            ObjectState::IDLE,
            ObjectVisibility::HIDDEN,
            model_ref,
            rng
        );

        Self { 
            base,
            turn_angle: 15.0 * (PI / 180.0),
            vel: Vector3::new(0.05, 0.05, 0.05),
            max_vel: Vector3::new(0.1, 0.1, 0.1),
            num_keys: 0, 
            health: 100.0, 
            reward: 0.0,
            path_ref,
            sensor_ref,
            sensor: Sensor::default(),
            history: CircularBuffer::<10, PlayerHistory>::new(),
        }
    }

    pub fn calc_bb(
        &self 
    ) -> Box3 {
        self.base.calc_bb()
    }
    
    pub fn mean_dir(
        &self
    ) -> Vector3 {
        let mut dir = Vector3::zero();
        for h in &self.history {
            dir.add_mut(&h.dir);
        }

        dir.mul_scalar(1.0 / (self.history.len() as f32).max(1.0))
    }

    pub fn calc_dir_away_from_hit(
        &self
    ) -> Vector3 {
        let mut res = Vector3::zero();
        for r in &self.sensor.readings {
            match r {
                SensorReading::Target(s) => {
                    res.add_mut(&s.dir);
                },
                SensorReading::Hit(_) => (),
            }
        }
        res.normalize()
    }
    
    pub fn to_obs(
        &self
    ) -> EnvObs {
        let mut obs = vec![];

        for r in &self.sensor.readings {
            obs.push(match r {
                SensorReading::Target(r) => {
                    r.sim
                },
                SensorReading::Hit(_) => {
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
            dir: self.base.dir.clone()
        });
    }

    pub fn get_sensor_readings(
        &self,
        target_pos: &Vector3,
        fov: &ObjectFOV,
        map: &Map
    ) -> Vec<SensorReading> {
        let target_dir = FORWARD.rotate_y(target_pos.sub(&self.base.pos).angle_z());

        let mut angle = (self.base.dir.angle_z() + 
                (fov.angle / 2.0)) % (PI * 2.0);
        let step = fov.angle / ((fov.rays - 1) as f32);

        let mut reading = vec![];

        for _ in 0..fov.rays {
            let dir = FORWARD.rotate_y(angle);
            let sensor_pos = self.base.pos.add(&dir.mul_scalar(fov.max_dist));
            
            reading.push(
                if let Some(t) = map.calc_distance_to_closest_wall(
                    &self.base.pos, &sensor_pos) {
                    dbg!(t);
                    SensorReading::Hit(SensorReadingHit{t})
                } else {
                    SensorReading::Target(SensorReadingTarget { 
                        sim: dir.dot(&target_dir).max(0.0), 
                        dir,
                    })
                }
            );

            angle -= step;
            if angle < 0.0 {
                angle += PI * 2.0;
            }
        }

        reading
    }

    pub fn gen_vis_sensor_readings(
        pos: &Vector3,
        readings: &Vec<SensorReading>,
        max_dist: f32
    ) -> Vec<Vector3> {
        
        let mut vertices = vec![];

        vertices.push(pos.clone());

        for r in readings {
            match r {
                SensorReading::Hit(_) => {
                }
                SensorReading::Target(r) => {
                    let to = pos.add(&r.dir.mul_scalar(r.sim * max_dist));
                    vertices.push(to);
                }
            }
        }

        vertices
    }
}
