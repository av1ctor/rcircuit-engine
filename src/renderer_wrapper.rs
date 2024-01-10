use wasm_bindgen::prelude::*;
use crate::renderer::{Renderer, ModelRef};
use three_rs::math::vector3::Vector3;
use three_rs::math::quaternion::Quaternion;

#[wasm_bindgen]
extern "C" {
    pub type JsRenderer;

    #[wasm_bindgen(method)]
    fn load_model(
        this: &JsRenderer,
        name: &str
    ) -> ModelRef;

    #[wasm_bindgen(method)]
    fn add_model(
        this: &JsRenderer,
        model_ref: ModelRef
    );

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
        rot: &[f32],
        q: &[f32]
    );

    #[wasm_bindgen(method)]
    fn set_scale(
        this: &JsRenderer,
        model_ref: ModelRef,
        pos: &[f32]
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
    fn load_model(
        &mut self,
        name: &str
    ) -> ModelRef {
        self.js_renderer.load_model(name)
    }

    fn add_model(
        &mut self,
        model_ref: ModelRef
    ) {
        self.js_renderer.add_model(model_ref)
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
        rot: &Vector3,
        q: &Quaternion
    ) {
        self.js_renderer.set_rotation(model_ref, &rot.to_slice(), &q.to_slice())
    }

    fn set_scale(
        &mut self,
        model_ref: ModelRef,
        scale: &Vector3
    ) {
        self.js_renderer.set_scale(model_ref, &scale.to_slice())
    }
}