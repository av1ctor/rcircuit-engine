use three_rs::math::vector3::Vector3;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SensorReadingTarget {
    pub sim: f32,
    pub dir: Vector3,
    pub pos: Vector3,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SensorReadingHit {
    pub t: f32,
    pub dir: Vector3,
    pub pos: Vector3,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SensorReading {
    Target(SensorReadingTarget),
    Hit(SensorReadingHit),
}

impl Default for SensorReading {
    fn default() -> Self {
        SensorReading::Target(SensorReadingTarget { 
            sim: 0.0,
            dir: Vector3::zero(),
            pos: Vector3::zero(),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sensor {
    pub readings: Vec<SensorReading>,
    pub has_hits: bool,
}