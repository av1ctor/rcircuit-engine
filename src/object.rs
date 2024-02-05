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
    /* WAITING: */      60 * 3,     
    /* IDLE: */         60,         
    /* WALKING: */      usize::MAX,
    /* RUNNING: */      usize::MAX,
    /* JUMPING: */      15,         
    /* BITING: */       60,         
    /* CELEBRATING: */  60 * 3,     
    /* DANCING: */      60 * 5,     
    /* CROUCHING: */    usize::MAX, 
    /* SITTING: */      usize::MAX, 
    /* SHOOTING: */     15,         
    /* PUNCHING: */     60,         
    /* KICKING: */      60,         
    /* CLIMBING: */     usize::MAX, 
    /* HIT: */          30,         
    /* EXPLODING: */    60 * 3,     
    /* AGONIZING: */    60 * 3,     
    /* DYING: */        60 * 5,     
    /* DEAD: */         usize::MAX, 
];

pub const PLAYER_STATE_TRANSITIONS: &[Option<ObjectState>] = &[
    /* WAITING:*/       Some(ObjectState::WALKING),
    /* IDLE: */         Some(ObjectState::WALKING),
    /* WALKING: */      None,
    /* RUNNING: */      None,
    /* JUMPING: */      None,
    /* BITING: */       Some(ObjectState::IDLE),
    /* CELEBRATING: */  None,
    /* DANCING: */      None,
    /* CROUCHING: */    None,
    /* SITTING: */      None,
    /* SHOOTING: */     None,
    /* PUNCHING: */     None,
    /* KICKING: */      None,
    /* CLIMBING: */     None,
    /* HIT: */          Some(ObjectState::IDLE),        
    /* EXPLODING: */    None,
    /* AGONIZING: */    None,
    /* DYING: */        Some(ObjectState::DEAD),        
    /* DEAD: */         None,                           
];

pub const NPC_STATE_TRANSITIONS: &[Option<ObjectState>] = &[
    /* WAITING:*/       Some(ObjectState::IDLE),
    /* IDLE: */         None,
    /* WALKING: */      None,
    /* RUNNING: */      None,
    /* JUMPING: */      None,
    /* BITING: */       None,
    /* CELEBRATING: */  None,
    /* DANCING: */      None,
    /* CROUCHING: */    None,
    /* SITTING: */      None,
    /* SHOOTING: */     None,
    /* PUNCHING: */     Some(ObjectState::IDLE),
    /* KICKING: */      None,
    /* CLIMBING: */     None,
    /* HIT: */          Some(ObjectState::IDLE),
    /* EXPLODING: */    None,
    /* AGONIZING: */    None,
    /* DYING: */        Some(ObjectState::DEAD),
    /* DEAD: */         None,                           
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
            let to = if self.typ == ObjectType::PLAYER {
                PLAYER_STATE_TRANSITIONS[self.state as usize]
            } 
            else {
                NPC_STATE_TRANSITIONS[self.state as usize]
            };

            if let Some(to) = to {
                if let Some(from) = self.set_state(to, step) {
                    return Some((from, to));
                }
            }
        }
        
        None
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

    pub fn is_idle(
        &self 
    ) -> bool {
        self.state == ObjectState::IDLE
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