use three_rs::math::{vector3::Vector3, quaternion::Quaternion};

pub type ModelRef = usize;

pub trait Renderer {
    fn load_model(
        &mut self,
        name: &str
    ) -> ModelRef;
    
    fn add_model(
        &mut self,
        model_ref: ModelRef
    );

    fn show_model(
        &mut self,
        model_ref: ModelRef
    );

    fn hide_model(
        &mut self,
        model_ref: ModelRef
    );

    fn set_position(
        &mut self,
        model_ref: ModelRef,
        pos: &Vector3
    );

    fn set_rotation(
        &mut self,
        model_ref: ModelRef,
        rot: &Vector3,
        q: &Quaternion
    );

    fn set_scale(
        &mut self,
        model_ref: ModelRef,
        scale: &Vector3
    );
}

