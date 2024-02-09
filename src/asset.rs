#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;
use crate::env::AssetType;
#[cfg(not(feature = "js"))]
use crate::{
    env::{Asset, AssetRef, AssetData, AssetValue},
    consts
};
#[cfg(not(feature = "js"))]
use std::collections::HashMap;

#[repr(usize)]
#[cfg_attr(feature = "js", wasm_bindgen)]
#[derive(PartialEq)]
pub enum GameAssetType {
    Level   = 0,
    Player  = 1,
    Npc     = 2,
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
    Level   = 0,
    Player1 = 1,
    Player2 = 2,
    Player3 = 3,
    Player4 = 4,
    Player5 = 5,
    Player6 = 6,
    Player7 = 7,
    Player8 = 8,
    Npc1    = 50,
    Npc2    = 51,
    Npc3    = 52,
    Exit    = 99,
}

#[cfg(not(feature = "js"))]
pub fn get_game_assets(
) -> HashMap<AssetRef, Asset> {
    let mut assets = HashMap::new();
    
    assets.insert(
        GameAssetRef::Level as _, 
        Asset::new(
            GameAssetType::Level as _, 
            AssetData::new(&[
                ("path", &AssetValue::None)
            ])
        )
    );
    
    for i in 0..consts::MAX_PLAYERS {
        assets.insert(
            (GameAssetRef::Player1 as usize) + i,
            Asset::new(
                GameAssetType::Player as _, 
                AssetData::new(&[
                    ("num", &AssetValue::U32(i as _))
                ])
            )
        );
    }

    for i in 0..3 {
        assets.insert(
            (GameAssetRef::Npc1 as usize) + i,
            Asset::new(
                GameAssetType::Npc as _, 
                AssetData::new(&[
                    ("num", &AssetValue::U32(i as _))
                ])
            )
        );
    }
    
    assets.insert(
        GameAssetRef::Exit as _, 
        Asset::new(
            GameAssetType::Exit as _, 
            AssetData::new(&[
            ])
        )
    );
    
    assets
}

