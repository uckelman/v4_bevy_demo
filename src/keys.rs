use bevy::{
    ecs::change_detection::Res,
    input::{
        ButtonInput,
        keyboard::KeyCode
    },
    prelude::Resource
};

pub fn shift_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

pub fn ctrl_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
}

pub fn alt_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
}

// TODO: make KeyConfig derivable?
pub trait KeyConfig {
    fn code(&self) -> KeyCode;
}

fn cfg_input_pressed<T>(
    key: Res<T>,
    inputs: Res<ButtonInput<KeyCode>>
) -> bool
where
    T: Resource + KeyConfig
{
    inputs.pressed(key.code())
}

fn cfg_input_just_pressed<T>(
    key: Res<T>,
    inputs: Res<ButtonInput<KeyCode>>
) -> bool
where
    T: Resource + KeyConfig
{
    inputs.just_pressed(key.code())
}
