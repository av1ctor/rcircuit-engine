use serde::{Serialize, Deserialize};
use three_rs::math::{Quaternion, Vector3};
use crate::{
    env::AssetRef, 
    object::{ObjectType, ObjectState}, 
    renderer::ModelRef
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectCreatedEvent {
    pub model_ref: ModelRef,
    pub typ: ObjectType,
    pub asset_ref: AssetRef,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectSpawnedEvent {
    pub model_ref: ModelRef,
    pub pos: Vector3,
    pub rot: Quaternion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectMovedEvent {
    pub model_ref: ModelRef,
    pub pos: Vector3,
    pub rot: Quaternion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRemovedEvent {
    pub model_ref: ModelRef,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectVisChangedEvent {
    pub visible: bool,
    pub model_ref: ModelRef,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectStateChangedEvent {
    pub model_ref: ModelRef,
    pub from: ObjectState,
    pub to: ObjectState
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    ObjectCreated(ObjectCreatedEvent),
    ObjectSpawned(ObjectSpawnedEvent),
    ObjectVisChanged(ObjectVisChangedEvent),
    ObjectMoved(ObjectMovedEvent),
    ObjectStateChanged(ObjectStateChangedEvent),
    ObjectRemoved(ObjectRemovedEvent),
}