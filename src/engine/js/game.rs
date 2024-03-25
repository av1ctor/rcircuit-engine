use neuralarena_cdk::models::js_renderer::JsRenderer;
use neuralarena_cdk::models::env::{Env, EnvObs};
use crate::{
    event::*, 
    entities::EntityState 
};
use crate::engine::{
    Game, GameEntities
};

impl Env<EntityState> for Game {
    fn reset(
        &mut self,
        renderer: JsRenderer,
    ) -> Vec<EnvObs> {
        renderer.reset();
        vec![]
    }

    fn deserialize_events(
        &mut self,
        buf: &Vec<u8>
    ) -> Result<(), String> {
        self.events = rmp_serde::from_slice(buf)
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    fn deserialize_entities(
        &mut self,
        buf: &Vec<u8>
    ) -> Result<(), String> {
        self.entities = rmp_serde::from_slice(buf)
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    fn render(
        &mut self,
        renderer: JsRenderer,
        update_only: bool
    ) {
        for event in &self.events {
            match event {
                GameEvent::EntityCreated(e) => {
                    renderer.add_model(e.asset_ref, e.model_ref);
                },
                
                GameEvent::EntitySpawned(e) => {
                    renderer.set_model_position(
                        e.model_ref, &e.pos.to_slice()
                    );
                    renderer.set_model_rotation(
                        e.model_ref, &e.rot.to_slice()
                    );
                },
                
                GameEvent::EntityMoved(e) => {
                    renderer.set_model_position(
                        e.model_ref, &e.pos.to_slice()
                    );
                    renderer.set_model_rotation(
                        e.model_ref, &e.rot.to_slice()
                    );
                },

                GameEvent::EntityStateChanged(e) => {
                    renderer.change_model_state(
                        e.model_ref, 
                        e.from.to_string().to_lowercase(), 
                        e.to.to_string().to_lowercase()
                    );
                },

                GameEvent::EntityRemoved(e) => {
                    renderer.hide_model(e.model_ref);
                },
                
                GameEvent::EntityVisChanged(e) => {
                    if e.visible {
                        renderer.show_model(e.model_ref);
                    }
                    else {
                        renderer.hide_model(e.model_ref);
                    }
                }
            }
        }

        renderer.set_players(self.entities.players.clone());
        renderer.set_npcs(self.entities.npcs.clone());

        if !update_only {
            renderer.render();
        }

        self.events = vec![];
    }
}

impl Game {
    pub fn new(
    ) -> Self {
        Self { 
            entities: GameEntities::default(),
            events: vec![],
        }
    }
}