use wasm_bindgen::prelude::*;
use crate::game::{
    Game, GameOptions, LevelOptions
};
use crate::player::PlayerFOV;
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
                GameOptions {
                    level: LevelOptions {
                        num: 0,
                    },
                    fov: PlayerFOV { 
                        rays: 0, 
                        angle: 0.0, 
                        max_dist: 0.0, 
                    },
                    num_players: 0,
                    min_keys: 0,
                    player_radius: 0.0,
                    debug: false,
                    training: false,
                    max_steps: 0
                }
            ),
        }
    }

    #[wasm_bindgen]
    pub fn render(
        &mut self,
        events: Vec<u8>,
        renderer: JsRenderer
    ) -> Result<(), String> {
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