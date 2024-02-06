use wasm_bindgen::prelude::*;
use crate::game::{
    Game, GameOptions, LevelOptions
};
use crate::player::PlayerFOV;
use crate::env::Env;
use crate::js_renderer::JsRenderer;
use crate::map::Map;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct PlayerFOVWrapper {
    #[wasm_bindgen(skip)]
    pub wrapped: PlayerFOV,
}

#[wasm_bindgen]
impl PlayerFOVWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rays: usize,
        angle: f32,
        max_dist: f32
    ) -> Self {
        Self {
            wrapped: PlayerFOV {
                rays,
                angle,
                max_dist,
            }
        }
    }
}

#[wasm_bindgen]
pub struct GameOptionsWrapper {
    #[wasm_bindgen(skip)]
    pub wrapped: GameOptions,
}

#[wasm_bindgen]
impl GameOptionsWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        fov: PlayerFOVWrapper,
        num_players: usize,
        min_keys: usize,
        player_radius: f32,
        debug: bool,
        training: bool,
        max_steps: usize
    ) -> Self {
        Self {
            wrapped: GameOptions {
                level: LevelOptions {
                    num: 0,
                },
                fov: fov.wrapped,
                num_players,
                min_keys,
                player_radius,
                debug,
                training,
                max_steps
            }
        }
    }
}

#[wasm_bindgen]
pub struct GameWrapper {
    #[wasm_bindgen(skip)]
    pub game: Game,
}

#[wasm_bindgen]
impl GameWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        options: GameOptionsWrapper
    ) -> Self {
        Self {
            game: Game::new(
                Map::new(&[], &[]), 
                options.wrapped
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