use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use three_rs::math::{
    vector3::Vector3, 
    Quaternion
};
use crate::{
    entity::{Entity, EntityData, EntityState, EntityType, EntityVisibility}, 
    renderer::ModelRef
};

const STATE_TRANSITIONS: &[Option<EntityState>] = &[
    /* WAITING: */      Some(EntityState::IDLE),
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
    /* PUNCHING: */     Some(EntityState::IDLE),
    /* KICKING: */      None,
    /* CLIMBING: */     None,
    /* HIT: */          Some(EntityState::IDLE),
    /* EXPLODING: */    None,
    /* AGONIZING: */    None,
    /* DYING: */        Some(EntityState::DEAD),
    /* DEAD: */         None,                           
];



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Npc {
    pub data: EntityData,
    pub vel: Vector3,
    pub health: f32,
}

impl Npc {
    pub fn new(
        index: usize,
        pos: Vector3,
        rot: Quaternion,
        dim: Vector3,
        state: EntityState,
        vis: EntityVisibility,
        model_ref: ModelRef,
        rng: &mut StdRng
    ) -> Self {
        let data = EntityData::new(
            EntityType::NPC,
            index,
            pos,
            rot,
            dim,
            state,
            vis,
            model_ref,
            rng
        );

        Self { 
            data,
            vel: Vector3::zero(),
            health: 100.0, 
        }
    }

    pub fn update_state(
        &mut self,
        step: usize
    ) -> Option<(EntityState, EntityState)> {
        self.update_state_ex(STATE_TRANSITIONS, step)
    }
}

impl Entity for Npc {
    fn get_data(
        &self
    ) -> &EntityData {
        &self.data
    }

    fn get_data_mut(
        &mut self
    ) -> &mut EntityData {
        &mut self.data
    }
}