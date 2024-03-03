use neuralarena_cdk::models::assets::{
    Asset, AssetData, 
    AssetRef, AssetType, AssetValue
};
use std::collections::HashMap;

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

pub fn get_game_assets(
    path: &String,
    players_paths: &Vec<String>
) -> HashMap<AssetRef, Asset> {
    let mut assets = HashMap::new();

    assets.insert(
        GameAssetRef::Level as _, 
        Asset::new(
            AssetType::Level as _, 
            AssetData::new(&[
                ("path", &AssetValue::Text(path.replace("{}", "maps/level-0/map.gltf")))
            ])
        )
    );

    for i in 0..players_paths.len() {
        assets.insert(
            (GameAssetRef::Player1 as usize) + i,
            Asset::new(
                AssetType::Player as _, 
                AssetData::new(&[
                    ("num", &AssetValue::U32(i as _)),
                    ("path", &AssetValue::Text(players_paths[i].clone()))
                ])
            )
        );
    }

    let npcs = vec!["man"];

    for i in 0..3 {
        assets.insert(
            (GameAssetRef::Npc1 as usize) + i,
            Asset::new(
                AssetType::Npc as _, 
                AssetData::new(&[
                    ("num", &AssetValue::U32(i as _)),
                    ("path", &AssetValue::Text(
                        path.replace("{}", &format!("models/npcs/{}/default.gltf", npcs[i % 1]))
                    ))
                ])
            )
        );
    }
    
    assets.insert(
        GameAssetRef::Exit as _, 
        Asset::new(
            AssetType::Object as _, 
            AssetData::new(&[
                ("num", &AssetValue::U32(GameAssetRef::Exit as _))
            ])
        )
    );
    
    assets
}

