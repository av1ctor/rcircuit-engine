use std::f32::consts::PI;
use circular_buffer::CircularBuffer;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::{
    Vector3, Quaternion
};
#[cfg(not(feature = "js"))]
use three_rs::math::vector3::{FORWARD, UP};
use crate::{
    entity::{Entity, EntityData, EntityState, EntityType, EntityVisibility}, env::EnvObs, renderer::ModelRef, sensor::{Sensor, SensorReading} 
};
#[cfg(not(feature = "js"))]
use crate::{
    sensor::{SensorReadingHit, SensorReadingTarget}, 
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

const STATE_TRANSITIONS: &[Option<EntityState>] = &[
    /* WAITING: */      Some(EntityState::WALKING),
    /* IDLE: */         Some(EntityState::WALKING),
    /* WALKING: */      None,
    /* RUNNING: */      None,
    /* JUMPING: */      None,
    /* BITING: */       Some(EntityState::IDLE),
    /* CELEBRATING: */  None,
    /* DANCING: */      None,
    /* CROUCHING: */    None,
    /* SITTING: */      None,
    /* SHOOTING: */     None,
    /* PUNCHING: */     None,
    /* KICKING: */      None,
    /* CLIMBING: */     None,
    /* HIT: */          Some(EntityState::IDLE),        
    /* EXPLODING: */    None,
    /* AGONIZING: */    None,
    /* DYING: */        Some(EntityState::DEAD),        
    /* DEAD: */         None,                           
];

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PlayerFOV {
    pub rays: usize,
    pub angle: f32,
    pub max_dist: f32,
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

#[derive(Clone, Debug, Default)]
pub struct PlayerHistory {
    pub rot: Quaternion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub data: EntityData,
    pub turn_angle: f32,
    pub vel: Vector3,
    pub num_brains: usize,
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
        state: EntityState,
        vis: EntityVisibility,
        model_ref: ModelRef,
        path_ref: ModelRef,
        sensor_ref: ModelRef,
        readings_ref: ModelRef,
        rng: &mut StdRng
    ) -> Player {
        let data = EntityData::new(
            EntityType::PLAYER,
            pos,
            rot,
            dim,
            state,
            vis,
            model_ref,
            rng
        );

        Self { 
            data,
            turn_angle: 45.0 * (PI / 180.0),
            vel: Vector3::new(0.1, 0.1, 0.1),
            num_brains: 0, 
            health: 100.0, 
            reward: 0.0,
            path_ref,
            sensor_ref,
            readings_ref,
            sensor: Sensor::default(),
            history: CircularBuffer::<10, PlayerHistory>::new(),
        }
    }

    pub fn update_state(
        &mut self,
        step: usize
    ) -> Option<(EntityState, EntityState)> {
        self.update_state_ex(STATE_TRANSITIONS, step)
    }

    pub fn is_biting(
        &self 
    ) -> bool {
        self.data.state == EntityState::BITING
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
        let mut sum_sim = 0.0;
        
        let mut rots_weights = self.sensor.readings.iter()
            .filter(|r| 
                if let SensorReading::Target(_) = r {
                    true
                } 
                else {
                    false
                }
            )
            .map(|r| {
                if let SensorReading::Target(t) = r {
                    sum_sim += t.sim;
                    (t.rot, t.sim)
                }
                else {
                    (Quaternion::identity(), 0.0)
                }
            })
            .collect::<Vec<_>>();

        rots_weights = rots_weights.iter()
            .map(|e| (e.0, e.1 / sum_sim))
            .collect();

        if rots_weights.len() > 0 && sum_sim > 1.0 {
            Quaternion::average_weighted(&rots_weights)
        }
        else {
            self.data.rot.mul(
                &Quaternion::from_axis_and_angle(&UP, 180.0 * PI / 180.0)
            )
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
            rot: self.data.rot.clone()
        });
    }

    #[cfg(not(feature = "js"))]
    pub fn get_sensor_readings(
        &self,
        target_pos: &Vector3,
        fov: &PlayerFOV,
        map: &mut Map
    ) -> (bool, Vec<SensorReading>) {
        let target_dir = FORWARD.apply_quaternion(
            &Quaternion::from_axis_and_angle(
                &UP,
                target_pos.sub(&self.data.pos).angle_xz()
            )
        );

        let mut rot = self.data.rot.mul(
            &Quaternion::from_axis_and_angle(&UP, (fov.angle / 2.0) % (PI * 2.0))
        );
        let step = Quaternion::from_axis_and_angle(&UP, -fov.angle / ((fov.rays - 1) as f32));

        let mut readings = vec![];
        let mut has_hits = false;

        for _ in 0..fov.rays {
            let dir = FORWARD.apply_quaternion(&rot);
            let sensor_pos = self.data.pos.add(&dir.mul_scalar(fov.max_dist));
            
            readings.push(
                if let Some(t) = map.calc_distance_to_closest_wall(
                    &self.data.pos, &sensor_pos) {
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

impl Entity for Player {
    fn get_data(
        &self
    ) -> &EntityData {
        &self.data
    }

    fn get_data_mut(
        &mut self
    ) -> &mut EntityData {
        &mut self.data
    }
}
