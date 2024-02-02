use candid::CandidType;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
#[cfg(not(feature = "js"))]
use rand::rngs::StdRng;
#[cfg(not(feature = "js"))]
use crate::renderer::Renderer;
#[cfg(feature = "js")]
use crate::js_renderer::JsRenderer;

pub type EnvObs = Vec<f32>;
pub type EnvAction = usize;

pub type AssetRef = usize;
pub type AssetType = usize;
pub type AssetKey = String;

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub enum AssetValue {
    None,
    Bool(bool),
    U32(u32),
    F32(f32),
    Text(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct AssetData(pub HashMap<AssetKey, AssetValue>);

impl AssetData {
    pub fn new(
        entries: &[(&str, &AssetValue)]
    ) -> Self {
        let mut data = HashMap::default();
        for e in entries {
            data.insert(e.0.to_string(), e.1.clone());
        }
        Self(data)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct Asset {
    pub typ: AssetType,
    pub data: AssetData,
}

impl Asset {
    pub fn new(
        typ: AssetType,
        data: AssetData
    ) -> Self {
        Self {
            typ,
            data,
        }
    }
}

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

pub trait Env {
    #[cfg(not(feature = "js"))]
    fn get_assets(
        &self
    ) -> HashMap::<AssetRef, Asset>;
    
    fn get_num_actions(
        &self
    ) -> usize;
    
    fn get_num_obs(
        &self
    ) -> usize;
    
    #[cfg(not(feature = "js"))]
    fn reset(
        &mut self, 
        renderer: &mut dyn Renderer,
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
        renderer: &mut dyn Renderer
    ) -> EnvStep;
    
    #[cfg(not(feature = "js"))]
    fn random_action(
        &mut self, 
        rng: &mut StdRng
    ) -> EnvAction;
    
    #[cfg(not(feature = "js"))]
    fn render(
        &mut self,
        renderer: &mut dyn Renderer,
        events_only: bool
    );

    #[cfg(feature = "js")]
    #[allow(async_fn_in_trait)]
    fn render(
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
}