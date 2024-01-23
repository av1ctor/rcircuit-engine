use three_rs::math::{
    Vector3, 
    Quaternion, 
    Euler
};

#[cfg(not(target_arch = "wasm32"))]
use three_rs::core::RGB;

pub type ModelRef = usize;

pub trait Renderer {
    fn reset(
        &mut self
    );
    
    fn add_model(
        &mut self,
        id: String
    ) -> Result<ModelRef, String>;

    #[cfg(not(target_arch = "wasm32"))]
    fn add_from_linestrips(
        &mut self,
        color: RGB,
        vertices: Vec<Vector3>
    ) -> Result<ModelRef, String>;

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
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

