use bevy::{
    ecs::{
        component::Component
    }
};
use serde::{Deserialize, Serialize};

#[derive(Component, Deserialize, Serialize)]
#[serde(rename = "create", tag = "type")]
pub struct CreateEdit {
    pub object_id: u32,
    pub source_id: u32
}
