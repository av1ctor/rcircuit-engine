#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum_macros::{Display, EnumCount};
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
    NPC     = 2,
    EXIT    = 3,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "js", wasm_bindgen)]
#[repr(C)]
pub enum ObjectState {
    #[default]
    WAITING     = 0,
    IDLE        = 1,
    WALKING     = 2,
    RUNNING     = 3,
    JUMPING     = 4,
    BITING      = 5,
    CELEBRATING = 6,
    DANCING     = 7,
    CROUCHING   = 8,
    SITTING     = 9,
    SHOOTING    = 10,
    PUNCHING    = 11,
    KICKING     = 12,
    CLIMBING    = 13,
    HIT         = 14,
    EXPLODING   = 15,
    AGONIZING   = 16,
    DYING       = 17,
    DEAD        = 18,
}

pub const STATE_DURATIONS: &[usize] = &[
    60 * 3,     // WAITING
    60,         // IDLE
    usize::MAX, // WALKING
    usize::MAX, // RUNNING
    15,         // JUMPING
    45,         // BITING
    60 * 3,     // CELEBRATING
    60 * 5,     // DANCING
    usize::MAX, // CROUCHING
    usize::MAX, // SITTING
    15,         // SHOOTING
    45,         // PUNCHING
    45,         // KICKING
    usize::MAX, // CLIMBING
    30,         // HIT
    60 * 3,     // EXPLODING
    60 * 3,     // AGONIZING
    60 * 3,     // DYING
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
    ) -> Option<ObjectState> {
        let from = self.state;
        if from != to {
            self.state = to;
            self.state_changed_at = step;
            Some(from)
        }
        else {
            None
        }
    }

    pub fn update_state(
        &mut self,
        step: usize
    ) -> Option<(ObjectState, ObjectState)> {
        let elapsed = step - self.state_changed_at;
        let duration = STATE_DURATIONS[self.state as usize];
        if duration != usize::MAX && elapsed >= duration {
            let to = match self.state {
                ObjectState::DYING => {
                    ObjectState::DEAD
                },
                _ => {
                    if self.typ == ObjectType::PLAYER {
                        ObjectState::WALKING
                    } 
                    else {
                        ObjectState::IDLE
                    }
                }
            };

            if let Some(from) = self.set_state(to, step) {
                Some((from, to))
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    pub fn state_elapsed(
        &self,
        step: usize
    ) -> f32 {
        let elapsed = step - self.state_changed_at;
        let duration = STATE_DURATIONS[self.state as usize];
        if elapsed >= duration || duration == 0 {
            1.0
        }
        else {
            elapsed as f32 / duration as f32
        }
    }

    pub fn is_dead(
        &self 
    ) -> bool {
        self.state == ObjectState::DYING || 
        self.state == ObjectState::DEAD
    }

    pub fn is_moving(
        &self 
    ) -> bool {
        self.state == ObjectState::WALKING || 
        self.state == ObjectState::RUNNING
    }

    pub fn is_visible(
        &self 
    ) -> bool {
        self.vis == ObjectVisibility::VISIBLE
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