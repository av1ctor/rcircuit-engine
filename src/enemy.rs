use serde::{Serialize, Deserialize};
use three_rs::math::vector3::Vector3;
use crate::object::Object;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Enemy {
    pub base: Object,
    pub dir: Vector3,
    pub health: f32,
    pub vel: Vector3,
    pub accel: Vector3,
}

impl Enemy {
}