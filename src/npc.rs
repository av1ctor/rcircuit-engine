use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use three_rs::math::{
    vector3::Vector3, 
    Box3, 
    Quaternion
};
use crate::{
    object::{Object, ObjectState, ObjectType, ObjectVisibility}, 
    renderer::ModelRef
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Npc {
    pub base: Object,
    pub vel: Vector3,
    pub health: f32,
}

impl Npc {
    pub fn new(
        pos: Vector3,
        rot: Quaternion,
        dim: Vector3,
        state: ObjectState,
        vis: ObjectVisibility,
        model_ref: ModelRef,
        rng: &mut StdRng
    ) -> Self {
        let base = Object::new(
            ObjectType::NPC,
            pos,
            rot,
            dim,
            state,
            vis,
            model_ref,
            rng
        );

        Self { 
            base,
            vel: Vector3::new(0.1, 0.1, 0.1),
            health: 100.0, 
        }
    }

    pub fn set_state(
        &mut self,
        to: ObjectState,
        step: usize
    ) {
        self.base.set_state(to, step);
    }

    pub fn update_state(
        &mut self,
        step: usize
    ) -> Option<(ObjectState, ObjectState)> {
        self.base.update_state(step)
    }

    pub fn is_moving(
        &self 
    ) -> bool {
        self.base.is_moving()
    }
    
    pub fn calc_bb(
        &self 
    ) -> Box3 {
        self.base.calc_bb()
    }

}