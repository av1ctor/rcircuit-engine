use serde::{Serialize, Deserialize};
use three_rs::math::{Quaternion, Vector3};
use crate::{
    env::AssetId, 
    object::ObjectType, 
    renderer::ModelRef
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectCreatedEvent {
    pub model_ref: ModelRef,
    pub typ: ObjectType,
    pub id: AssetId,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectSpawnedEvent {
    pub model_ref: ModelRef,
    pub pos: Vector3,
    pub rot: Quaternion,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectMovedEvent {
    pub model_ref: ModelRef,
    pub pos: Vector3,
    pub rot: Quaternion,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectDiedEvent {
    pub model_ref: ModelRef,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectShownEvent {
    pub model_ref: ModelRef,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectHiddenEvent {
    pub model_ref: ModelRef,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum GameEvent {
    ObjectCreated(ObjectCreatedEvent),
    ObjectSpawned(ObjectSpawnedEvent),
    ObjectShown(ObjectShownEvent),
    ObjectHidden(ObjectHiddenEvent),
    ObjectMoved(ObjectMovedEvent),
    ObjectDied(ObjectDiedEvent),
}