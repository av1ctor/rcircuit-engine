use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use three_rs::math::{
    vector3::Vector3, 
    Quaternion
};
use neuralarena_cdk::models::renderer::ModelRef;
use crate::entities::{
    Entity, EntityData, EntityState, EntityType, EntityVisibility
};

const STATE_TRANSITIONS: &[Option<EntityState>] = &[
    /* WAITING: */      Some(EntityState::IDLE),
    /* IDLE: */         None,
    /* WALKING: */      None,
    /* BITING: */       None,
    /* CELEBRATING: */  None,
    /* PUNCHING: */     None,
    /* HIT: */          None,
    /* DYING: */        None,
    /* DEAD: */         None,                           
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub enum ObjectType {
    UNKNOWN = 0,
    EXIT    = 1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub data: EntityData,
    pub typ: ObjectType,
}

impl Object {
    pub fn new(
        typ: ObjectType,
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
        let data = EntityData::new(
            EntityType::OBJECT,
            index,
            pos,
            rot,
            dim,
            radius,
            state,
            vis,
            model_ref,
            rng
        );

        Self { 
            data,
            typ
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

impl Entity for Object {
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