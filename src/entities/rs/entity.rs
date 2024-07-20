use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use three_rs::math::{
    box3::Box3, vector3::Vector3, Quaternion
};
use neuralarena_cdk::models::renderer::ModelRef;
use crate::{
    consts, 
    entities::{EntityState, EntityType, EntityVisibility}
};

const STATE_DURATIONS: &[usize] = &[
    /* WAITING: */      consts::FPS * 3,
    /* IDLE: */         consts::FPS * 1,
    /* WALKING: */      usize::MAX,
    /* ATTACKING: */    consts::FPS * 1,
    /* CELEBRATING: */  consts::FPS * 3,
    /* PUNCHING: */     (consts::FPS * 3) / 4,
    /* HIT: */          consts::FPS * 1,
    /* DYING: */        consts::FPS * 5,
    /* DEAD: */         usize::MAX,
];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EntityData {
    pub id: usize,
    pub index: usize,
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
        index: usize,
        pos: Vector3,
        rot: Quaternion,
        dim: Vector3,
        radius: f32,
        state: EntityState,
        vis: EntityVisibility,
        model_ref: ModelRef,
        rng: &mut StdRng
    ) -> Self {
        Self { 
            id: rng.gen::<usize>(),
            index,
            pos,
            rot,
            dim,
            radius,
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
        assert!(STATE_DURATIONS.len() == EntityState::COUNT);
        
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
    fn is_alive(
        &self 
    ) -> bool {
        self.get_data().state != EntityState::DYING &&
        self.get_data().state != EntityState::DEAD
    }

    #[inline(always)]
    fn is_dead(
        &self 
    ) -> bool {
        self.get_data().state == EntityState::DEAD
    }

    #[inline(always)]
    fn is_moving(
        &self 
    ) -> bool {
        self.get_data().state == EntityState::WALKING
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