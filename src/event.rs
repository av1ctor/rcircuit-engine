use serde::{Serialize, Deserialize};
use three_rs::math::{Quaternion, Vector3};
use crate::{
    env::AssetRef, 
    entity::{EntityType, EntityState}, 
    renderer::ModelRef
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityCreatedEvent {
    pub model_ref: ModelRef,
    pub typ: EntityType,
    pub asset_ref: AssetRef,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntitySpawnedEvent {
    pub model_ref: ModelRef,
    pub pos: Vector3,
    pub rot: Quaternion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityMovedEvent {
    pub model_ref: ModelRef,
    pub pos: Vector3,
    pub rot: Quaternion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityRemovedEvent {
    pub model_ref: ModelRef,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityVisChangedEvent {
    pub visible: bool,
    pub model_ref: ModelRef,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityStateChangedEvent {
    pub model_ref: ModelRef,
    pub from: EntityState,
    pub to: EntityState
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    EntityCreated(EntityCreatedEvent),
    EntitySpawned(EntitySpawnedEvent),
    EntityVisChanged(EntityVisChangedEvent),
    EntityMoved(EntityMovedEvent),
    EntityStateChanged(EntityStateChangedEvent),
    EntityRemoved(EntityRemovedEvent),
}