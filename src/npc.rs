use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use three_rs::math::{
    vector3::Vector3, 
    Quaternion
};
use neuralarena_cdk::models::renderer::ModelRef;
use crate::entity::{
    Entity, EntityData, EntityState, EntityType, EntityVisibility
};

const STATE_TRANSITIONS: &[Option<EntityState>] = &[
    /* WAITING: */      Some(EntityState::IDLE),
    /* IDLE: */         None,
    /* WALKING: */      None,
    /* BITING: */       None,
    /* CELEBRATING: */  None,
    /* PUNCHING: */     Some(EntityState::IDLE),
    /* HIT: */          Some(EntityState::IDLE),
    /* DYING: */        Some(EntityState::DEAD),
    /* DEAD: */         None,                           
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Npc {
    pub data: EntityData,
    pub vel: Vector3,
    pub health: f32,
    pub targeted_cnt: usize,
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
            targeted_cnt: 0,
        }
    }

    pub fn update_state(
        &mut self,
        step: usize
    ) -> Option<(EntityState, EntityState)> {
        assert!(STATE_TRANSITIONS.len() == EntityState::COUNT);

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