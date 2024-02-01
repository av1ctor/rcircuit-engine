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
    Brain   = 2,
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
    Player3 = 3,
    Player4 = 4,
    Brain = 5,
    Exit = 6,
}

#[cfg(not(feature = "js"))]
pub fn get_game_assets(
) -> Vec<Asset> {
    vec![
        Asset::new(GameAssetType::Level as _, AssetData::new(&[("path", &AssetValue::None)])),
        Asset::new(GameAssetType::Player as _, AssetData::new(&[("num", &AssetValue::U32(0))])),
        Asset::new(GameAssetType::Player as _, AssetData::new(&[("num", &AssetValue::U32(1))])),
        Asset::new(GameAssetType::Player as _, AssetData::new(&[("num", &AssetValue::U32(2))])),
        Asset::new(GameAssetType::Player as _, AssetData::new(&[("num", &AssetValue::U32(3))])),
        Asset::new(GameAssetType::Brain as _, AssetData::new(&[])),
        Asset::new(GameAssetType::Exit as _, AssetData::new(&[])),
    ]
}

