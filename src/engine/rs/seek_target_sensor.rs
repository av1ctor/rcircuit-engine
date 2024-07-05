use three_rs::math::{vector3::Vector3, Quaternion};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SeekTargetSensorReadingTarget {
    pub sim: f32,
    pub pos: Vector3,
    pub dir: Vector3,
    pub rot: Quaternion,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SeekTargetSensorReadingHit {
    pub t: f32,
    pub pos: Vector3,
    pub dir: Vector3,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SeekTargetSensorReading {
    Target(SeekTargetSensorReadingTarget),
    Hit(SeekTargetSensorReadingHit),
}

impl Default for SeekTargetSensorReading {
    fn default() -> Self {
        SeekTargetSensorReading::Target(SeekTargetSensorReadingTarget { 
            sim: 0.0,
            dir: Vector3::zero(),
            rot: Quaternion::identity(),
            pos: Vector3::zero(),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct SeekTargetSensor {
    pub readings: Vec<SeekTargetSensorReading>,
    pub has_hits: bool,
}