use bevy::{
    input::keyboard::KeyCode,
    prelude::Resource
};

#[derive(Resource)]
pub struct KeyPanStep(pub f32);

#[derive(Resource)]
pub struct KeyRotateStep(pub f32);

#[derive(Resource)]
pub struct KeyScaleStep(pub f32);

#[derive(Resource)]
pub struct WheelScaleStep(pub f32);

// TODO: make KeyConfig derivable?

pub trait KeyConfig {
    fn code(&self) -> KeyCode;
}

#[derive(Resource)]
pub struct PanLeftKey(pub KeyCode);

#[derive(Resource)]
pub struct PanRightKey(pub KeyCode);

#[derive(Resource)]
pub struct PanUpKey(pub KeyCode);

#[derive(Resource)]
pub struct PanDownKey(pub KeyCode);

#[derive(Resource)]
pub struct ZoomInKey(pub KeyCode);

#[derive(Resource)]
pub struct ZoomOutKey(pub KeyCode);

#[derive(Resource)]
pub struct RotateCCWKey(pub KeyCode);

#[derive(Resource)]
pub struct RotateCWKey(pub KeyCode);

impl KeyConfig for PanLeftKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for PanRightKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for PanUpKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for PanDownKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for ZoomInKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for ZoomOutKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for RotateCCWKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for RotateCWKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

