use rapier3d::prelude::*;
use three_rs::math::{Quaternion, Vector3};
use super::character::CharacterRef;

pub struct PhysicsManager {
}

impl PhysicsManager {
    pub fn new(
    ) -> Self {
        Self {
        }
    }

    pub fn add_character(
        &mut self,
        _pos: &Vector3,
        _rot: &Quaternion,
        _height: f32,
        _radius: f32,
        _rb_typ: Option<RigidBodyType>
    ) -> CharacterRef {
        0
    }
}