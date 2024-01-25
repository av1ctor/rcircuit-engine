use std::f32::consts::PI;
use circular_buffer::CircularBuffer;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::{
    Box3, Vector3, Quaternion
};
#[cfg(not(feature = "js"))]
use three_rs::math::vector3::{FORWARD, UP};
use crate::{
    env::EnvObs, 
    object::{Object, ObjectVisibility, ObjectState, ObjectType}, 
    sensor::{Sensor, SensorReading}, 
    renderer::ModelRef, 
};
#[cfg(not(feature = "js"))]
use crate::{
    sensor::{SensorReadingHit, SensorReadingTarget}, 
    game::ObjectFOV, 
    map::Map
};

pub const NUM_SENSOR_FEATURES: usize = 1;

#[derive(EnumCount)]
#[repr(C)]
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
    pub rot: Quaternion,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub base: Object,
    pub turn_angle: f32,
    pub vel: Vector3,
    pub num_keys: usize,
    pub health: f32,
    pub reward: f32,
    #[serde(skip)]
    pub path_ref: ModelRef,
    #[serde(skip)]
    pub sensor_ref: ModelRef,
    #[serde(skip)]
    pub readings_ref: ModelRef,
    #[serde(skip)]
    pub sensor: Sensor,
    #[serde(skip)]
    pub history: CircularBuffer<10, PlayerHistory>,
}

impl Player {
    pub fn new(
        pos: Vector3,
        rot: Quaternion,
        dim: Vector3,
        model_ref: ModelRef,
        path_ref: ModelRef,
        sensor_ref: ModelRef,
        readings_ref: ModelRef,
        rng: &mut StdRng
    ) -> Player {
        let base = Object::new(
            ObjectType::PLAYER,
            pos,
            rot,
            dim,
            ObjectState::IDLE,
            ObjectVisibility::HIDDEN,
            model_ref,
            rng
        );

        Self { 
            base,
            turn_angle: 15.0 * (PI / 180.0),
            vel: Vector3::new(0.1, 0.1, 0.1),
            num_keys: 0, 
            health: 100.0, 
            reward: 0.0,
            path_ref,
            sensor_ref,
            readings_ref,
            sensor: Sensor::default(),
            history: CircularBuffer::<10, PlayerHistory>::new(),
        }
    }

    pub fn calc_bb(
        &self 
    ) -> Box3 {
        self.base.calc_bb()
    }
    
    #[cfg(not(feature = "js"))]
    pub fn mean_rot(
        &self
    ) -> Quaternion {
        Quaternion::average(&self.history.iter().map(|h| h.rot).collect())
    }

    #[cfg(not(feature = "js"))]
    pub fn calc_rot_away_from_hit(
        &self
    ) -> Quaternion {
        let mut acc = Vector3::zero();
        let mut cnt = 0;
        
        for r in &self.sensor.readings {
            if let SensorReading::Target(s) = r {
                acc.add_mut(&s.rot.to_angular_velocity());
                cnt += 1;
            }
        }

        if cnt > 0 {
            Quaternion::from_angular_velocity(&acc.mul_scalar(1.0 / cnt as f32))
        }
        else {
            self.base.rot.mul(&Quaternion::from_axis_and_angle(&UP, 180.0 * PI / 180.0))
        }
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
            rot: self.base.rot.clone()
        });
    }

    #[cfg(not(feature = "js"))]
    pub fn get_sensor_readings(
        &self,
        target_pos: &Vector3,
        fov: &ObjectFOV,
        map: &mut Map
    ) -> (bool, Vec<SensorReading>) {
        let target_dir = FORWARD.rotate_y(target_pos.sub(&self.base.pos).angle_xz());

        let mut rot = self.base.rot.mul(
            &Quaternion::from_axis_and_angle(&UP, (fov.angle / 2.0) % (PI * 2.0))
        );
        let step = Quaternion::from_axis_and_angle(&UP, -fov.angle / ((fov.rays - 1) as f32));

        let mut readings = vec![];
        let mut has_hits = false;

        for _ in 0..fov.rays {
            let dir = FORWARD.apply_quaternion(&rot);
            let sensor_pos = self.base.pos.add(&dir.mul_scalar(fov.max_dist));
            
            readings.push(
                if let Some(t) = map.calc_distance_to_closest_wall(
                    &self.base.pos, &sensor_pos) {
                    has_hits = true;
                    SensorReading::Hit(SensorReadingHit{
                        t,
                        pos: sensor_pos,
                        dir,
                    })
                } else {
                    SensorReading::Target(SensorReadingTarget { 
                        sim: dir.dot(&target_dir).max(0.0), 
                        pos: sensor_pos,
                        dir,
                        rot,
                    })
                }
            );

            rot = rot.mul(&step);
        }

        (has_hits, readings)
    }

    #[cfg(not(feature = "js"))]
    pub fn gen_vis_sensor_readings(
        pos: &Vector3,
        readings: &Vec<SensorReading>,
        max_dist: f32
    ) -> (Vec<Vector3>, Vec<Vector3>) {
        
        let mut sensor_vtx = vec![];
        let mut readings_vtx = vec![];

        for r in readings {

            match r {
                SensorReading::Hit(_) => {
                }
                SensorReading::Target(r) => {
                    sensor_vtx.push(pos.clone());
                    sensor_vtx.push(r.pos);
                    
                    readings_vtx.push(pos.clone());
                    readings_vtx.push(pos.add(&r.dir.mul_scalar(r.sim * max_dist)));
                }
            }
        }

        (sensor_vtx, readings_vtx)
    }
}
