use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};
use serde::{Deserialize, Serialize};

pub type CharacterRef = usize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Character {
    pub mass: f32,
    pub rigid_body_handle: Option<RigidBodyHandle>,
    pub collider_handle: ColliderHandle,
}