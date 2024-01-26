use std::collections::HashMap;
#[cfg(not(feature = "js"))]
use rand::rngs::StdRng;
#[cfg(not(feature = "js"))]
use crate::renderer::Renderer;
#[cfg(feature = "js")]
use crate::js_renderer::JsRenderer;

pub type EnvObs = Vec<f32>;
pub type EnvAction = usize;
pub type AssetId = usize;

pub struct EnvPlayer {
    pub reward: f32,
    pub obs: EnvObs,
}

pub struct EnvStep {
    pub terminated: bool,
    pub timedout: bool,
    pub winner: Option<usize>,
    pub players: Vec<EnvPlayer>,
}

pub trait Env<T> {
    fn get_assets(
        &self
    ) -> HashMap<AssetId, T>;
    
    fn get_num_actions(
        &self
    ) -> usize;
    
    fn get_num_obs(
        &self
    ) -> usize;
    
    #[cfg(not(feature = "js"))]
    fn reset(
        &mut self, 
        renderer: &mut dyn Renderer<T>,
        rng: &mut StdRng
    ) -> Vec<EnvObs>;

    #[cfg(feature = "js")]
    fn reset(
        &mut self, 
        renderer: JsRenderer
    ) -> Vec<EnvObs>;

    #[cfg(not(feature = "js"))]
    fn step(
        &mut self, 
        actions: &Vec<Option<EnvAction>>, 
        delta_time: f32,
        renderer: &mut dyn Renderer<T>
    ) -> EnvStep;
    
    #[cfg(not(feature = "js"))]
    fn random_action(
        &mut self, 
        rng: &mut StdRng
    ) -> EnvAction;
    
    #[cfg(not(feature = "js"))]
    fn render(
        &mut self,
        renderer: &mut dyn Renderer<T>
    );

    #[cfg(feature = "js")]
    #[allow(async_fn_in_trait)]
    async fn render(
        &mut self,
        renderer: JsRenderer
    );

    fn serialize_entities(
        &self
    ) -> Result<Vec<u8>, String>;
	
    fn deserialize_entities(
        &mut self, 
        buf: &Vec<u8>
    ) -> Result<(), String>;

    fn serialize_events(
        &self
    ) -> Result<Vec<u8>, String>;
	
    fn deserialize_events(
        &mut self, 
        buf: &Vec<u8>
    ) -> Result<(), String>;

    fn clear_events(
        &mut self
    );
}