use wasm_bindgen::prelude::*;
use crate::engine::Game;
use neuralarena_cdk::models::{
    env::Env,
    js_renderer::JsRenderer
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct GameEngine {
    #[wasm_bindgen(skip)]
    pub game: Game,
}

#[wasm_bindgen]
impl GameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(
    ) -> Self {
        Self {
            game: Game::new(),
        }
    }

    #[wasm_bindgen]
    pub fn render(
        &mut self,
        entities: Vec<u8>,
        events: Vec<u8>,
        renderer: JsRenderer,
        delta_time: f32
    ) -> Result<(), String> {
        if entities.len() > 0 {
            self.game.deserialize_entities(&entities)?;
        }
        if events.len() > 0 {
            self.game.deserialize_events(&events)?;
        }
        
        self.game.render(renderer, delta_time);
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn reset(
        &mut self,
        renderer: JsRenderer
    ) {
        self.game.reset(renderer);
    }
}