#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use crate::env::AssetId;

#[repr(usize)]
#[cfg_attr(feature = "js", wasm_bindgen)]
pub enum GameAssetId {
    Level0  = 0,
    Player  = 1,
    Key     = 2,
    Exit    = 3,
}

impl From<AssetId> for GameAssetId {
    fn from(
        value: AssetId
    ) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<GameAssetId> for AssetId {
    fn from(
        value: GameAssetId
    ) -> Self {
        value as _
    }
}

#[cfg(not(feature = "js"))]
#[derive(Clone)]
pub struct GameAsset {
    pub path: Option<String>,
}

#[cfg(feature = "js")]
#[derive(Clone)]
#[wasm_bindgen]
pub struct GameAsset {
    #[wasm_bindgen(skip)]
    pub path: Option<String>,
}

pub const ASSETS: &[(AssetId, GameAsset)] = &[
    (GameAssetId::Level0 as _, GameAsset{path: None}),
    (GameAssetId::Player as _, GameAsset{path: None}),
    (GameAssetId::Key as _, GameAsset{path: None}),
    (GameAssetId::Exit as _, GameAsset{path: None}),
];

