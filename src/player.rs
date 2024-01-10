use std::f32::consts::PI;
use circular_buffer::CircularBuffer;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::{
    vector3::{Vector3, FORWARD}, 
    quaternion::Quaternion, 
    box3::Box3, euler::{Euler, EulerOrder}
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
    pub accel: Vector3,
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
        dir: Quaternion,
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
            vel: Vector3::zero(),
            max_vel: Vector3::new(0.0, 0.0, 0.25),
            accel: Vector3::new(0.0, 0.0, 0.01),
            num_keys: 0, 
            health: 100.0, 
            reward: 0.0,
            path_ref,
            sensor_ref,
            sensor: Sensor::default(),
            history: CircularBuffer::<10, PlayerHistory>::new(),
        }
    }

    pub fn rotate_x(
        &mut self,
        angle: f32
    ) {
        self.base.rotate_x(angle);
    }

    pub fn rotate_y(
        &mut self,
        angle: f32
    ) {
        self.base.rotate_y(angle);
    }

    pub fn rotate_z(
        &mut self,
        angle: f32
    ) {
        self.base.rotate_z(angle);
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
                    r.similarity
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
            dir: self.base.dir.rot.clone()
        });
    }

    fn calc_distance_to_closest_wall(
        from: &Vector3,
        to: &Vector3,
        map: &Map
    ) -> Option<f32> {
        if let Some(hit) = map.query.raycast(from, to) {
            Some(hit.0)
        }
        else {
            None
        }
    }

    fn calc_similarity_with_target(
        dir: &Vector3,
        target_dir: &Vector3
    ) -> f32 {
        dir.dot(&target_dir).max(0.0)
    }

    pub fn get_sensor_readings(
        &self,
        target_pos: &Vector3,
        fov: &ObjectFOV,
        map: &Map
    ) -> Vec<SensorReading> {
        //FIXME!!!
        let target_dir = Euler::from_quaternion(
            &Quaternion::from_axis_and_angle(
                &FORWARD, 
                self.base.pos.angle_to(target_pos)
            ),
            EulerOrder::XYZ
        ).v;
        //target_pos.sub(&player.base.pos).normalize();

        let mut rot = self.base.dir.q.rotate_z((fov.angle / 2.0) % (PI * 2.0));
        let rot_step = fov.angle / ((fov.rays - 1) as f32);

        let mut reading = vec![];

        for _ in 0..fov.rays {
            let dir = Euler::from_quaternion(&rot, EulerOrder::XYZ).v;
            let sensor_pos = dir.mul_scalar(fov.max_dist);
            
            reading.push(
                if let Some(t) = Self::calc_distance_to_closest_wall(
                    &self.base.pos, &sensor_pos, map) {
                    SensorReading::Hit(SensorReadingHit{t})
                } else {
                    SensorReading::Target(SensorReadingTarget { 
                        similarity: Self::calc_similarity_with_target(&dir, &target_dir), 
                        angle: rot.angle(),
                        dir,
                    })
                }
            );

            rot = rot.rotate_z(rot_step);
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
                    let q = Quaternion::from_axis_and_angle(&FORWARD, r.angle);
                    let v = Vector3::new(0.0, 0.0, r.similarity * max_dist)
                        .rotate_by_quaternion(&q);
                    let to = pos.add(&v);

                    vertices.push(to);
                }
            }
        }

        vertices
    }
}
