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
pub enum EntityType {
    #[default]
    UNKNOWN = 0,
    PLAYER  = 1,
    NPC     = 2,
    OBJECT  = 3,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "js", wasm_bindgen)]
#[repr(C)]
pub enum EntityState {
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

const STATE_DURATIONS: &[usize] = &[
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "js", wasm_bindgen)]
#[repr(C)]
pub enum EntityVisibility {
    #[default]
    HIDDEN  = 0,
    VISIBLE = 1,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EntityData {
    pub id: usize,
    pub typ: EntityType,
    pub vis: EntityVisibility,
    pub state: EntityState,
    pub state_changed_at: usize,
    pub pos: Vector3,
    pub rot: Quaternion,
    pub dim: Vector3,
    pub radius: f32,
    #[serde(skip)]
    pub model_ref: ModelRef,
}

impl EntityData {
    pub fn new(
        typ: EntityType,
        pos: Vector3,
        rot: Quaternion,
        dim: Vector3,
        state: EntityState,
        vis: EntityVisibility,
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
}

pub trait Entity {
    fn get_data(
        &self
    ) -> &EntityData;

    fn get_data_mut(
        &mut self
    ) -> &mut EntityData;

    fn set_state(
        &mut self,
        to: EntityState,
        step: usize
    ) -> Option<EntityState> {
        let data = self.get_data_mut();
        let from = data.state;
        if from != to {
            data.state = to;
            data.state_changed_at = step;
            Some(from)
        }
        else {
            None
        }
    }

    fn update_state_ex(
        &mut self,
        transitions: &[Option<EntityState>],
        step: usize
    ) -> Option<(EntityState, EntityState)> {
        let data = self.get_data();
        let elapsed = step - data.state_changed_at;
        let duration = STATE_DURATIONS[data.state as usize];
        if duration != usize::MAX && elapsed >= duration {
            let to = transitions[data.state as usize];

            if let Some(to) = to {
                if let Some(from) = self.set_state(to, step) {
                    return Some((from, to));
                }
            }
        }
        
        None
    }

    fn state_elapsed(
        &self,
        step: usize
    ) -> f32 {
        let data = self.get_data();
        let elapsed = step - data.state_changed_at;
        let duration = STATE_DURATIONS[data.state as usize];
        if elapsed >= duration || duration == 0 {
            1.0
        }
        else {
            elapsed as f32 / duration as f32
        }
    }
    
    #[inline(always)]
    fn is_idle(
        &self 
    ) -> bool {
        self.get_data().state == EntityState::IDLE
    }
    
    #[inline(always)]
    fn is_dead(
        &self 
    ) -> bool {
        self.get_data().state == EntityState::DYING || 
        self.get_data().state == EntityState::DEAD
    }

    #[inline(always)]
    fn is_moving(
        &self 
    ) -> bool {
        self.get_data().state == EntityState::WALKING || 
        self.get_data().state == EntityState::RUNNING
    }

    #[inline(always)]
    fn is_visible(
        &self 
    ) -> bool {
        self.get_data().vis == EntityVisibility::VISIBLE
    }

    fn calc_bb(
        &self 
    ) -> Box3 {
        let half_dim = self.get_data().dim.mul_scalar(0.5);
        Box3::new(
            self.get_data().pos.sub(&half_dim), 
            self.get_data().pos.add(&half_dim)
        )
    }
}