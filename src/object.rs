use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::vector3::Vector3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumCount, Default)]
pub enum ObjectType {
    #[default]
    UNKNOWN,
    PLAYER,
    ENEMY,
    KEY,
    TRAP,
    EXIT,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum ObjectState {
    #[default]
    IDLE,
    MOVING,
    RUNNING,
    JUMPING,
    DANCING,
    CROUCHING,
    SITTING,
    SHOOTING,
    PUNCHING,
    KICKING,
    CLIMBING,
    HURTING,
    EXPLODING,
    AGONIZING,
    DEAD,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum ObjectVisibility {
    #[default]
    HIDDEN,
    VISIBLE,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub id: usize,
    pub typ: ObjectType,
    pub vis: ObjectVisibility,
    pub state: ObjectState,
    pub pos: Vector3,
    pub dim: Vector3,
    pub radius: f32,
}

impl Default for Object {
    fn default() -> Self {
        Self { 
            id: Default::default(), 
            typ: Default::default(), 
            pos: Default::default(), 
            dim: Default::default(), 
            radius: Default::default(), 
            state: Default::default(), 
            vis: Default::default(),
        }
    }
}

impl Object {
    pub fn scale(
        &self,
        value: f32
    ) -> Self {
        Self {
            pos: self.pos.mul_scalar(value),
            dim: self.dim.mul_scalar(value),
            ..self.clone()
        }
    }
}