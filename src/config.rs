use bevy::{
    ecs::{
        error::Result,
        prelude::Commands
    },
    prelude::Resource
};
use serde::Deserialize;

use crate::KeyBinding;

#[derive(Debug, Deserialize)]
pub struct Steps {
    pub pan_step: f32,
    pub rotate_step: f32,
    pub key_scale_step: f32,
    pub wheel_scale_step: f32
}

#[derive(Clone, Debug, Deserialize)]
pub struct Keys {
    pub pan_left: KeyBinding,
    pub pan_right: KeyBinding,
    pub pan_up: KeyBinding,
    pub pan_down: KeyBinding,
    pub zoom_in: KeyBinding,
    pub zoom_out: KeyBinding,
    pub rotate_ccw: KeyBinding,
    pub rotate_cw: KeyBinding,
    pub undo: KeyBinding,
    pub redo: KeyBinding
}

#[derive(Debug, Deserialize, Resource)]
pub struct Config {
    pub steps: Steps,
    pub keys: Keys
}

pub fn load_config(mut commands: Commands) -> Result {
    let config_str = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_str)?;

    commands.insert_resource(config);

    Ok(())
}
