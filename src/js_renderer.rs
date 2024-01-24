use wasm_bindgen::prelude::*;
use crate::renderer::ModelRef;

#[wasm_bindgen]
extern "C" {
    pub type JsRenderer;

    #[wasm_bindgen(method)]
    pub async fn add_model(
        this: &JsRenderer,
        id: &str,
        model_ref: ModelRef
    ) -> JsValue;

    #[wasm_bindgen(method)]
    pub fn show_model(
        this: &JsRenderer,
        model_ref: ModelRef
    );

    #[wasm_bindgen(method)]
    pub fn hide_model(
        this: &JsRenderer,
        model_ref: ModelRef
    );

    #[wasm_bindgen(method)]
    pub fn set_position(
        this: &JsRenderer,
        model_ref: ModelRef,
        pos: &[f32]
    );

    #[wasm_bindgen(method)]
    pub fn set_rotation(
        this: &JsRenderer,
        model_ref: ModelRef,
        q: &[f32]
    );

    #[wasm_bindgen(method)]
    pub fn set_rotation_from_euler(
        this: &JsRenderer,
        model_ref: ModelRef,
        vector: &[f32],
        order: u32
    );

    #[wasm_bindgen(method)]
    pub fn set_scale(
        this: &JsRenderer,
        model_ref: ModelRef,
        scale: &[f32]
    );

    #[wasm_bindgen(method)]
    pub fn render(
        this: &JsRenderer
    );

    #[wasm_bindgen(method)]
    pub fn reset(
        this: &JsRenderer
    );
}
