use gltf::buffer::Data;
use three_rs::math::{vector3::Vector3, quaternion::Quaternion};

pub type ModelRef = usize;

pub trait Renderer {
    fn add_model_from_file(
        &mut self,
        name: &str
    ) -> Result<ModelRef, String>;

    fn add_model_from_bytes(
        &mut self,
        bytes: &[u8]
    ) -> Result<ModelRef, String>;

    fn add_model_from_gltf(
        &mut self,
        doc: &gltf::Document,
        buffers: &Vec<Data>
    ) -> Result<ModelRef, String>;

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

    fn render(
        &mut self
    );
}

