use neuralarena_cdk::models::{
    assets::{
        Asset, AssetData, 
        AssetRef, AssetType, AssetValue
    },
    game::GameMapId
};
use std::collections::HashMap;

#[repr(C)]
#[allow(dead_code)]
pub(crate) enum GameAssetRef {
    Player1 = 1,
    Player2 = 2,
    Player3 = 3,
    Player4 = 4,
    Player5 = 5,
    Player6 = 6,
    Player7 = 7,
    Player8 = 8,
    Map0    = 20,
    Map1    = 21,
    Map2    = 22,
    Map3    = 23,
    Map4    = 24,
    Map5    = 25,
    Map6    = 26,
    Map7    = 27,
    Map8    = 28,
    Map9    = 29,
    Npc1    = 50,
    Npc2    = 51,
    Npc3    = 52,
    Exit    = 99,
}

pub fn get_game_assets(
    path: &String,
    map_id: GameMapId,
    players_paths: &Vec<String>
) -> HashMap<AssetRef, Asset> {
    let mut assets = HashMap::new();

    assets.insert(
        (GameAssetRef::Map0 as usize) + map_id as usize, 
        Asset::new(
            AssetType::Map, 
            AssetData::new(&[
                (
                    "path", 
                    &AssetValue::Text(
                        path.replace("{}", &format!("maps/{}/map.gltf", map_id as usize))
                    )
                )
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

