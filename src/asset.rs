use crate::env::AssetId;

#[repr(usize)]
pub enum GameAssetId {
    Level0 = 0,
    Player,
    Key,
    Exit,
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

#[derive(Clone)]
pub struct GameAsset {
    pub path: Option<String>,
}

pub const ASSETS: &[(AssetId, GameAsset)] = &[
    (GameAssetId::Level0 as _, GameAsset{path: None}),
    (GameAssetId::Player as _, GameAsset{path: None}),
    (GameAssetId::Key as _, GameAsset{path: None}),
    (GameAssetId::Exit as _, GameAsset{path: None}),
];

