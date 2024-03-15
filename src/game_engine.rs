use wasm_bindgen::prelude::*;
use crate::consts;
use crate::engine::{
    Game, MapInfo
};
use neuralarena_cdk::models::{
    env::Env,
    js_renderer::JsRenderer
};
use crate::map::Map;

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
            game: Game::new(
                Map::new(&[], &[]), 
                MapInfo {
                    map_id: consts::MAP_ID_CONTAINERS,
                    ..Default::default()
                }
            ),
        }
    }

    #[wasm_bindgen]
    pub fn render(
        &mut self,
        entities: Vec<u8>,
        events: Vec<u8>,
        renderer: JsRenderer
    ) -> Result<(), String> {
        if entities.len() > 0 {
            self.game.deserialize_entities(&entities)?;
        }
        if events.len() > 0 {
            self.game.deserialize_events(&events)?;
        }
        
        self.game.render(renderer);
        
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