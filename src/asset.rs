#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use crate::env::AssetType;
#[cfg(not(feature = "js"))]
use crate::env::{Asset, AssetData, AssetValue};

#[repr(usize)]
#[cfg_attr(feature = "js", wasm_bindgen)]
pub enum GameAssetType {
    Level   = 0,
    Player  = 1,
    Key     = 2,
    Exit    = 3,
}

impl From<AssetType> for GameAssetType {
    fn from(
        value: AssetType
    ) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<GameAssetType> for AssetType {
    fn from(
        value: GameAssetType
    ) -> Self {
        value as _
    }
}

#[cfg(not(feature = "js"))]
#[repr(C)]
#[allow(dead_code)]
pub(crate) enum GameAssetRef {
    Level = 0,
    Player1 = 1,
    Player2 = 2,
    Key = 3,
    Exit = 4,
}

#[cfg(not(feature = "js"))]
pub fn get_game_assets(
) -> Vec<Asset> {
    vec![
        Asset::new(GameAssetType::Level as _, AssetData::new(&[("path", &AssetValue::None)])),
        Asset::new(GameAssetType::Player as _, AssetData::new(&[("num", &AssetValue::U32(0))])),
        Asset::new(GameAssetType::Player as _, AssetData::new(&[("num", &AssetValue::U32(1))])),
        Asset::new(GameAssetType::Key as _, AssetData::new(&[])),
        Asset::new(GameAssetType::Exit as _, AssetData::new(&[])),
    ]
}

