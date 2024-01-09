use gltf::buffer::Data;
use three_rs::math::{vector3::Vector3, quaternion::Quaternion, triangle::Triangle};

pub type ModelRef = usize;
pub type RGB = [u8; 3];

pub trait Renderer {
    fn reset(
        &mut self
    );
    
    fn add_from_file(
        &mut self,
        color: RGB,
        name: &str
    ) -> Result<ModelRef, String>;

    fn add_from_bytes(
        &mut self,
        color: RGB,
        bytes: &[u8]
    ) -> Result<ModelRef, String>;

    fn add_from_gltf(
        &mut self,
        color: RGB,
        doc: &gltf::Document,
        buffers: &Vec<Data>
    ) -> Result<ModelRef, String>;

    fn add_from_triangles(
        &mut self,
        color: RGB,
        triangles: Vec<Triangle>
    ) -> Result<ModelRef, String>;

    fn add_from_vertices(
        &mut self,
        color: RGB,
        vertices: Vec<Vector3>
    ) -> Result<ModelRef, String>;

    fn show_model(
        &mut self,
        model_ref: ModelRef
    );

    fn hide_model(
        &mut self,
        model_ref: ModelRef
    );

    fn set_vertices(
        &mut self,
        model_ref: ModelRef,
        vertices: Vec<Vector3>
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

