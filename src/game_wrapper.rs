use wasm_bindgen::prelude::*;
use crate::game::{
    Game, GameOptions, LevelOptions, ObjectFOV
};
use crate::env::Env;
use crate::js_renderer::JsRenderer;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct ObjectFOVWrapper {
    #[wasm_bindgen(skip)]
    pub wrapped: ObjectFOV,
}

#[wasm_bindgen]
impl ObjectFOVWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rays: usize,
        angle: f32,
        max_dist: f32
    ) -> Self {
        Self {
            wrapped: ObjectFOV {
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
        fov: ObjectFOVWrapper,
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
            game: Game::new(&[], &[], options.wrapped),
        }
    }

    #[wasm_bindgen]
    pub fn deserialize_entities(
        &mut self,
        buf: Vec<u8>
    ) -> Result<(), String> {
        self.game.deserialize_entities(&buf)
    }

    #[wasm_bindgen]
    pub fn deserialize_events(
        &mut self,
        buf: Vec<u8>
    ) -> Result<(), String> {
        self.game.deserialize_events(&buf)
    }

    #[wasm_bindgen]
    pub fn render(
        &mut self,
        renderer: JsRenderer
    ) {
        self.game.render(renderer)
    }

    #[wasm_bindgen]
    pub fn reset(
        &mut self,
        renderer: JsRenderer
    ) {
        self.game.reset(renderer);
    }
}