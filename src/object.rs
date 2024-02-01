#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::{
    box3::Box3, vector3::Vector3, Quaternion
};

use crate::renderer::ModelRef;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumCount, Default)]
#[cfg_attr(feature = "js", wasm_bindgen)]
#[repr(C)]
pub enum ObjectType {
    #[default]
    UNKNOWN = 0,
    PLAYER  = 1,
    ENEMY   = 2,
    BRAIN     = 3,
    TRAP    = 4,
    EXIT    = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "js", wasm_bindgen)]
#[repr(C)]
pub enum ObjectState {
    #[default]
    IDLE        = 0,
    MOVING      = 1,
    RUNNING     = 2,
    JUMPING     = 3,
    EATING      = 4,
    CELEBRATING = 5,
    DANCING     = 6,
    CROUCHING   = 7,
    SITTING     = 8,
    SHOOTING    = 9,
    PUNCHING    = 10,
    KICKING     = 11,
    CLIMBING    = 12,
    HURTING     = 13,
    EXPLODING   = 14,
    AGONIZING   = 15,
    DEAD        = 16,
}

pub const STATE_DURATIONS: &[usize] = &[
    60 * 3,     // IDLE
    usize::MAX, // MOVING
    usize::MAX, // RUNNING
    30,         // JUMPING
    60 * 5,     // EATING
    60 * 3,     // CELEBRATING
    60 * 5,     // DANCING
    usize::MAX, // CROUCHING
    usize::MAX, // SITTING
    usize::MAX, // SHOOTING
    usize::MAX, // PUNCHING
    usize::MAX, // KICKING
    usize::MAX, // CLIMBING
    60 * 3,     // HURTING
    60 * 5,     // EXPLODING
    60 * 5,     // AGONIZING
    usize::MAX, // DEAD
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "js", wasm_bindgen)]
#[repr(C)]
pub enum ObjectVisibility {
    #[default]
    HIDDEN  = 0,
    VISIBLE = 1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub id: usize,
    pub typ: ObjectType,
    pub vis: ObjectVisibility,
    pub state: ObjectState,
    pub state_changed_at: usize,
    pub pos: Vector3,
    pub rot: Quaternion,
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
            rot: Default::default(),
            dim: Default::default(), 
            radius: Default::default(), 
            state: Default::default(), 
            state_changed_at: 0,
            vis: Default::default(),
            model_ref: 0,
        }
    }
}

impl Object {
    pub fn new(
        typ: ObjectType,
        pos: Vector3,
        rot: Quaternion,
        dim: Vector3,
        state: ObjectState,
        vis: ObjectVisibility,
        model_ref: ModelRef,
        rng: &mut StdRng
    ) -> Self {
        Self { 
            id: rng.gen::<usize>(),
            pos,
            rot,
            dim,
            radius: 1.0,
            typ,
            state,
            state_changed_at: 0,
            vis,
            model_ref,
        }
    }

    pub fn set_state(
        &mut self,
        to: ObjectState,
        step: usize
    ) {
        self.state = to;
        self.state_changed_at = step;
    }

    pub fn update_state(
        &mut self,
        step: usize
    ) -> Option<(ObjectState, ObjectState)> {
        let duration = STATE_DURATIONS[self.state as usize];
        if duration != usize::MAX && step - self.state_changed_at  >= duration {
            let old_state = self.state;
            self.state = ObjectState::MOVING;
            self.state_changed_at = step;
            Some((old_state, self.state))
        }
        else {
            None
        }
    }

    pub fn is_moving(
        &self 
    ) -> bool {
        self.state == ObjectState::MOVING || 
        self.state == ObjectState::RUNNING
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