use wasm_bindgen::prelude::*;
use crate::renderer::ModelRef;
use crate::env::AssetId;
use crate::asset::GameAsset;

#[wasm_bindgen]
extern "C" {
    pub type JsRenderer;

    pub fn load_asset(
        this: &JsRenderer,
        id: AssetId,
        asset: GameAsset
    ) -> JsValue;

    #[wasm_bindgen(method)]
    pub fn add_model(
        this: &JsRenderer,
        id: AssetId,
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
