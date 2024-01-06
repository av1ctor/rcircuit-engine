use rand::rngs::StdRng;
use three_rs::math::vector3::Vector3;

use crate::renderer::Renderer;

pub type EnvObs = Vec<f32>;
pub type EnvAction = usize;

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
    fn get_num_actions(&self) -> usize;
    fn get_num_obs(&self) -> usize;
    fn reset(&mut self, rng: &mut StdRng) -> Vec<EnvObs>;
    fn step(&mut self, actions: &Vec<Option<EnvAction>>, delta_time: f32) -> EnvStep;
    fn render(&self, pov: &Vector3, scale: f32, renderer: &mut dyn Renderer);
    fn random_action(&mut self, rng: &mut StdRng) -> EnvAction;
    fn serialize_entities(&self) -> Result<Vec<u8>, String>;
	fn deserialize_entities(&mut self, buf: &Vec<u8>) -> Result<(), String>;
}