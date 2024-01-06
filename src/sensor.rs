use three_rs::math::vector3::Vector3;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SensorReadingTarget {
    pub similarity: f32,
    pub angle: f32,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SensorReadingHit {
    pub inter: Vector3,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SensorReading {
    Target(SensorReadingTarget),
    Hit(SensorReadingHit),
}

impl Default for SensorReading {
    fn default() -> Self {
        SensorReading::Target(SensorReadingTarget { 
            similarity: 0.0, 
            angle: 0.0 
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sensor {
    pub readings: Vec<SensorReading>,
}