use wasm_bindgen::prelude::*;
use crate::game::{Game, GameOptions, ObjectFOV};
use crate::env::Env;
use crate::rendererwrapper::RendererWrapper;
use crate::rendererwrapper::JsRenderer;

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
        debug: bool,
        fov: ObjectFOVWrapper,
        fps: f32,
        num_players: usize,
        min_keys: usize,
        player_radius: f32,
        training: bool,
        max_steps: usize
    ) -> Self {
        Self {
            wrapped: GameOptions {
                debug,
                fov: fov.wrapped,
                fps,
                num_players,
                min_keys,
                player_radius,
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
    #[wasm_bindgen(skip)]
    pub renderer: RendererWrapper,
}

#[wasm_bindgen]
impl GameWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        level_bytes: &[u8], 
        navmesh_bytes: &[u8],
        options: GameOptionsWrapper,
        js_renderer: JsRenderer
    ) -> Self {
        Self {
            game: Game::new(level_bytes, navmesh_bytes, options.wrapped),
            renderer: RendererWrapper::new(js_renderer),
        }
    }

    #[wasm_bindgen]
    pub fn deserialize_entities(
        &mut self,
        buf: Vec<u8>
    ) -> Result<(), String> {
        self.game.deserialize_entities(&buf)
    }
}