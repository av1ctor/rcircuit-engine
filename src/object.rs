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
    /* PUNCHING: */     None,
    /* KICKING: */      None,
    /* CLIMBING: */     None,
    /* HIT: */          None,
    /* EXPLODING: */    None,
    /* AGONIZING: */    None,
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
        pos: Vector3,
        rot: Quaternion,
        dim: Vector3,
        state: EntityState,
        vis: EntityVisibility,
        model_ref: ModelRef,
        rng: &mut StdRng
    ) -> Self {
        let data = EntityData::new(
            EntityType::OBJECT,
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
            typ
        }
    }

    pub fn update_state(
        &mut self,
        step: usize
    ) -> Option<(EntityState, EntityState)> {
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