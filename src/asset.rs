use neuralarena_cdk::models::{
    assets::{
        Asset, AssetData, 
        AssetRef, AssetType, AssetValue
    },
    game::GameMapId
};
use std::collections::HashMap;

#[repr(C)]
pub enum GameAssetRef {
    Map0    = 10,
    Map1    = 11,
    Map2    = 12,
    Map3    = 13,
    Map4    = 14,
    Map5    = 15,
    Map6    = 16,
    Map7    = 17,
    Map8    = 18,
    Map9    = 19,
    Player0 = 30,
    Player1 = 31,
    Player2 = 32,
    Player3 = 33,
    Player4 = 34,
    Player5 = 35,
    Player6 = 36,
    Player7 = 37,
    Player8 = 38,
    Player9 = 39,
    Npc0    = 50,
    Npc1    = 51,
    Npc2    = 52,
    Exit    = 70,
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
            (GameAssetRef::Player0 as usize) + i,
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
            (GameAssetRef::Npc0 as usize) + i,
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

