use neuralarena_cdk::models::{
    assets::{
        Asset, AssetData, 
        AssetRef, AssetType, AssetValue
    },
    game::{GameInfo, GameMapId}, 
    instance::InstancePlayer
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
    game: Option<&GameInfo>,
    map_id: GameMapId,
    players: &Vec<Option<&InstancePlayer>>
) -> HashMap<AssetRef, Asset> {
    let mut assets = HashMap::new();

    if let Some(game) = game {
        let map = &game.maps[map_id];

        assets.insert(
            (GameAssetRef::Map0 as usize) + map_id as usize, 
            Asset::new(
                AssetType::Map, 
                AssetData::new(&[
                    ("model_url", &AssetValue::Text(map.urls.model.clone())),
                    ("music_url", &AssetValue::Text(map.urls.music.clone()))
                ])
            )
        );
    }
    else {
        assets.insert(
            (GameAssetRef::Map0 as usize) + map_id as usize, 
            Asset::new(
                AssetType::Map, 
                AssetData::new(&[
                    (
                        "model_url", 
                        &AssetValue::Text("".to_string())
                    )
                ])
            )
        );
    }

    for i in 0..players.len() {
        if let Some(player) = players[i] {
            let set_num = player.nft.props.sfx_set;
            let mut sfx_set = HashMap::<String, AssetValue>::new();
            for e in &player.nft_col.nas1.sfx_sets[set_num] {
                sfx_set.insert(
                    e.0.clone(), 
                    AssetValue::Map(HashMap::from(
                        [
                            ("url".to_string(), AssetValue::Text(e.1.url.clone())),
                            ("repeat".to_string(), AssetValue::Bool(e.1.repeat)),
                        ]
                    ))
                );
            }

            assets.insert(
                (GameAssetRef::Player0 as usize) + i,
                Asset::new(
                    AssetType::Player as _, 
                    AssetData::new(&[
                        ("num", &AssetValue::U32(i as _)),
                        ("model_url", &AssetValue::Text(player.nft.props.model_url.clone())),
                        ("sfx_set", &AssetValue::Map(sfx_set)),
                    ])
                )
            );
        }
        else {
            assets.insert(
                (GameAssetRef::Player0 as usize) + i,
                Asset::new(
                    AssetType::Player as _, 
                    AssetData::new(&[
                        ("num", &AssetValue::U32(i as _)),
                        ("model_url", &AssetValue::Text("".to_string())),
                    ])
                )
            );
        }
    }

    let npcs = vec!["man"];

    for i in 0..3 {
        if let Some(game) = game {
            let assets_url = &game.urls.assets;

            let mut sfx_set = HashMap::<String, AssetValue>::new();
            for e in &game.sfx_sets[i % 1] {
                sfx_set.insert(
                    e.0.clone(), 
                    AssetValue::Map(HashMap::from(
                        [
                            ("url".to_string(), AssetValue::Text(e.1.url.clone())),
                            ("repeat".to_string(), AssetValue::Bool(e.1.repeat)),
                        ]
                    ))
                );
            }
            
            assets.insert(
                (GameAssetRef::Npc0 as usize) + i,
                Asset::new(
                    AssetType::Npc as _, 
                    AssetData::new(&[
                        ("num", &AssetValue::U32(i as _)),
                        ("model_url", &AssetValue::Text(
                            assets_url.replace("{}", &format!("models/npcs/{}/default.gltf", npcs[i % 1]))
                        )),
                        ("sfx_set", &AssetValue::Map(sfx_set)),
                    ])
                )
            );
        }
        else {
            assets.insert(
                (GameAssetRef::Npc0 as usize) + i,
                Asset::new(
                    AssetType::Npc as _, 
                    AssetData::new(&[
                        ("num", &AssetValue::U32(i as _)),
                        ("model_url", &AssetValue::Text("".to_string())),
                    ])
                )
            );
        }
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

