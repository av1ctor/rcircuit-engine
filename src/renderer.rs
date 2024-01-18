#[cfg(not(feature = "js"))]
use std::{cell::RefCell, rc::Rc};
#[cfg(not(feature = "js"))]
use gltf::buffer::Data;
use three_rs::math::{
    vector3::Vector3, 
    quaternion::Quaternion, 
    Euler
};
#[cfg(not(feature = "js"))]
use three_rs::{
    math::triangle::Triangle, 
    core::RGB, object::Mesh
};

pub type ModelRef = usize;

pub trait Renderer {
    fn reset(
        &mut self
    );
    
    fn add_from_file(
        &mut self,
        name: &str
    ) -> Result<ModelRef, String>;

    #[cfg(not(feature = "js"))]
    fn add_from_bytes(
        &mut self,
        bytes: &[u8]
    ) -> Result<ModelRef, String>;

    #[cfg(not(feature = "js"))]
    fn add_from_gltf(
        &mut self,
        doc: &gltf::Document,
        buffers: &Vec<Data>
    ) -> Result<ModelRef, String>;

    #[cfg(not(feature = "js"))]
    fn add_from_mesh(
        &mut self,
        color: RGB,
        mesh: Rc<RefCell<Mesh>>
    ) -> Result<ModelRef, String>;

    #[cfg(not(feature = "js"))]
    fn add_from_triangles(
        &mut self,
        color: RGB,
        triangles: Vec<Triangle>
    ) -> Result<ModelRef, String>;

    #[cfg(not(feature = "js"))]
    fn add_from_linestrips(
        &mut self,
        color: RGB,
        vertices: Vec<Vector3>
    ) -> Result<ModelRef, String>;

    #[cfg(not(feature = "js"))]
    fn add_from_lines(
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

    #[cfg(not(feature = "js"))]
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
        q: &Quaternion
    );

    fn set_rotation_from_euler(
        &mut self,
        model_ref: ModelRef,
        euler: &Euler
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

