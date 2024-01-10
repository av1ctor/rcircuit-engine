use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::{
    vector3::{Vector3, RIGHT, UP, FORWARD}, 
    quaternion::Quaternion, 
    euler::{EulerOrder, Euler}, 
    box3::Box3
};

use crate::renderer::ModelRef;

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
    pub dir: Vector3,
    pub dim: Vector3,
    pub radius: f32,
    #[serde(skip)]
    pub model_ref: ModelRef,
}

impl Default for Object {
    fn default() -> Self {
        Self { 
            id: Default::default(), 
            typ: Default::default(), 
            pos: Default::default(), 
            dir: Default::default(),
            dim: Default::default(), 
            radius: Default::default(), 
            state: Default::default(), 
            vis: Default::default(),
            model_ref: 0,
        }
    }
}

impl Object {
    pub fn new(
        typ: ObjectType,
        pos: Vector3,
        dir: Vector3,
        dim: Vector3,
        state: ObjectState,
        vis: ObjectVisibility,
        model_ref: ModelRef,
        rng: &mut StdRng
    ) -> Self {
        Self { 
            id: rng.gen::<usize>(),
            pos,
            dir,
            dim,
            radius: 1.0,
            typ,
            state,
            vis,
            model_ref,
        }
    }

    pub fn calc_bb(
        &self 
    ) -> Box3 {
        let half_dim = self.dim.mul_scalar(0.5);
        Box3::new(
            self.pos.sub(&half_dim), 
            self.pos.add(&half_dim)
        )
    }
}