use three_rs::math::vector3::Vector3;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SensorReadingTarget {
    pub dir: Vector3,
    pub sim: f32,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SensorReadingHit {
    pub t: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SensorReading {
    Target(SensorReadingTarget),
    Hit(SensorReadingHit),
}

impl Default for SensorReading {
    fn default() -> Self {
        SensorReading::Target(SensorReadingTarget { 
            dir: Vector3::zero(),
            sim: 0.0
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sensor {
    pub readings: Vec<SensorReading>,
}