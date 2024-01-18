use wasm_bindgen::prelude::*;
use crate::renderer::{Renderer, ModelRef};
use three_rs::math::{
    vector3::Vector3,
    quaternion::Quaternion,
    euler::Euler
};

#[wasm_bindgen]
extern "C" {
    pub type JsRenderer;

    #[wasm_bindgen(method)]
    fn add_from_file(
        this: &JsRenderer,
        name: &str
    ) -> ModelRef;

    #[wasm_bindgen(method)]
    fn show_model(
        this: &JsRenderer,
        model_ref: ModelRef
    );

    #[wasm_bindgen(method)]
    fn hide_model(
        this: &JsRenderer,
        model_ref: ModelRef
    );

    #[wasm_bindgen(method)]
    fn set_position(
        this: &JsRenderer,
        model_ref: ModelRef,
        pos: &[f32]
    );

    #[wasm_bindgen(method)]
    fn set_rotation(
        this: &JsRenderer,
        model_ref: ModelRef,
        q: &[f32]
    );

    #[wasm_bindgen(method)]
    fn set_rotation_from_euler(
        this: &JsRenderer,
        model_ref: ModelRef,
        vector: &[f32],
        order: u32
    );

    #[wasm_bindgen(method)]
    fn set_scale(
        this: &JsRenderer,
        model_ref: ModelRef,
        pos: &[f32]
    );

    #[wasm_bindgen(method)]
    fn render(
        this: &JsRenderer
    );

    #[wasm_bindgen(method)]
    fn reset(
        this: &JsRenderer
    );
}

pub struct RendererWrapper {
    pub js_renderer: JsRenderer,
}

impl RendererWrapper {
    pub fn new(
        js_renderer: JsRenderer
    ) -> Self {
        Self {
            js_renderer
        }
    }
}

impl Renderer for RendererWrapper {
    fn add_from_file(
        &mut self,
        name: &str
    ) -> Result<ModelRef, String> {
        Ok(self.js_renderer.add_from_file(name))
    }

    fn show_model(
        &mut self,
        model_ref: ModelRef
    ) {
        self.js_renderer.show_model(model_ref)
    }

    fn hide_model(
        &mut self,
        model_ref: ModelRef
    ) {
        self.js_renderer.hide_model(model_ref)
    }

    fn set_position(
        &mut self,
        model_ref: ModelRef,
        pos: &Vector3
    ) {
        self.js_renderer.set_position(model_ref, &pos.to_slice())
    }

    fn set_rotation(
        &mut self,
        model_ref: ModelRef,
        q: &Quaternion
    ) {
        self.js_renderer.set_rotation(model_ref, &q.to_slice())
    }

    fn set_rotation_from_euler(
        &mut self,
        model_ref: ModelRef,
        euler: &Euler
    ) {
        self.js_renderer.set_rotation_from_euler(model_ref, &euler.v.to_slice(), euler.order as u32)
    }

    fn set_scale(
        &mut self,
        model_ref: ModelRef,
        scale: &Vector3
    ) {
        self.js_renderer.set_scale(model_ref, &scale.to_slice())
    }

    fn render(
        &mut self
    ) {
        self.js_renderer.render()
    }

    fn reset(
        &mut self
    ) {
        self.js_renderer.reset()
    }
}