use bevy::{
    ecs::change_detection::Res,
    input::{
        ButtonInput,
        keyboard::KeyCode
    },
    prelude::Resource
};
use serde::Deserialize;

pub fn shift_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

pub fn ctrl_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
}

pub fn alt_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
}

pub fn cfg_input_pressed<T>(
    key: Res<T>,
    inputs: Res<ButtonInput<KeyCode>>
) -> bool
where
    T: Resource + AsRef<KeyBinding>
{
    let kb = (*key).as_ref();
    inputs.pressed(kb.code) && inputs.modifiers_pressed(&kb.modifiers)
}

pub fn cfg_input_just_pressed<T>(
    key: Res<T>,
    inputs: Res<ButtonInput<KeyCode>>
) -> bool
where
    T: Resource + AsRef<KeyBinding>
{
    let kb = (*key).as_ref();
    inputs.just_pressed(kb.code) && inputs.modifiers_pressed(&kb.modifiers)
}

#[derive(Clone, Copy, Debug)]
pub enum Handedness {
    Left,
    Right,
    Either
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Modifiers {
    pub alt_key: Option<Handedness>,
    pub ctrl_key: Option<Handedness>,
    pub shift_key: Option<Handedness>,
    pub super_key: Option<Handedness>
}

trait ModifiersExt {
    fn modifiers_pressed(&self, m: &Modifiers) -> bool;

    fn alt_matched(&self, m: &Modifiers) -> bool;

    fn ctrl_matched(&self, m: &Modifiers) -> bool;

    fn shift_matched(&self, m: &Modifiers) -> bool;

    fn super_matched(&self, m: &Modifiers) -> bool;

    fn modifier_matched(
        &self,
        h: Option<Handedness>,
        l: KeyCode,
        r: KeyCode
    ) -> bool;
}

impl ModifiersExt for ButtonInput<KeyCode> {
    fn modifiers_pressed(
        &self,
        m: &Modifiers
    ) -> bool
    {
        self.alt_matched(m) &&
        self.ctrl_matched(m) &&
        self.shift_matched(m) &&
        self.super_matched(m)
    }

    fn alt_matched(&self, m: &Modifiers) -> bool {
        self.modifier_matched(m.alt_key, KeyCode::AltLeft, KeyCode::AltRight)
    }

    fn ctrl_matched(&self, m: &Modifiers) -> bool {
        self.modifier_matched(m.ctrl_key, KeyCode::ControlLeft, KeyCode::ControlRight)
    }

    fn shift_matched(&self, m: &Modifiers) -> bool {
        self.modifier_matched(m.shift_key, KeyCode::ShiftLeft, KeyCode::ShiftRight)
    }

    fn super_matched(&self, m: &Modifiers) -> bool {
        self.modifier_matched(m.super_key, KeyCode::SuperLeft, KeyCode::SuperRight)
    }

    fn modifier_matched(
        &self,
        h: Option<Handedness>,
        l: KeyCode,
        r: KeyCode
    ) -> bool
    {
        match h {
            None => !self.pressed(l) && !self.pressed(r),
            Some(Handedness::Left) => self.pressed(l) && !self.pressed(r),
            Some(Handedness::Right) => !self.pressed(l) && self.pressed(r),
            Some(Handedness::Either) => self.pressed(l) || self.pressed(r)
        }
    }
}

#[derive(Debug, Deserialize)]
enum Key {
    Alt,
    Ctrl,
    Shift,
    Super,
    #[serde(untagged)]
    Code(KeyCode)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "Vec<Key>")]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: Modifiers
}

// TODO: set key bindings for context menu actions

#[derive(Debug, thiserror::Error)]
#[error("invalid data {0:?}")]
pub struct KeyBindingError(Vec<Key>);

impl TryFrom<Vec<Key>> for KeyBinding {
    type Error = KeyBindingError;

    fn try_from(v: Vec<Key>) -> Result<Self, Self::Error> {
        let mut code = None;
        let mut modifiers = Modifiers::default();

        for k in &v {
            match k {
                Key::Alt => match modifiers.alt_key {
                    None => { modifiers.alt_key = Some(Handedness::Either); }
                    Some(_) => { return Err(KeyBindingError(v)); }
                },
                Key::Ctrl => match modifiers.ctrl_key {
                    None => { modifiers.ctrl_key = Some(Handedness::Either); }
                    Some(_) => { return Err(KeyBindingError(v)); }
                },
                Key::Shift => match modifiers.shift_key {
                    None => { modifiers.shift_key = Some(Handedness::Either); }
                    Some(_) => { return Err(KeyBindingError(v)); }
                },
                Key::Super => match modifiers.shift_key {
                    None => { modifiers.shift_key = Some(Handedness::Either); }
                    Some(_) => { return Err(KeyBindingError(v)); }
                },
                Key::Code(c) => match code {
                    None => match c {
                        KeyCode::AltLeft => match modifiers.alt_key {
                            None => { modifiers.alt_key = Some(Handedness::Left); }
                            Some(_) => { return Err(KeyBindingError(v)); }
                        },
                        KeyCode::AltRight => match modifiers.alt_key {
                            None => { modifiers.alt_key = Some(Handedness::Right); }
                            Some(_) => { return Err(KeyBindingError(v)); }
                        },
                        KeyCode::ControlLeft => match modifiers.ctrl_key {
                            None => { modifiers.ctrl_key = Some(Handedness::Left); }
                            Some(_) => { return Err(KeyBindingError(v)); }
                        },
                        KeyCode::ControlRight => match modifiers.ctrl_key {
                            None => { modifiers.ctrl_key = Some(Handedness::Right); }
                            Some(_) => { return Err(KeyBindingError(v)); }
                        },
                        KeyCode::ShiftLeft => match modifiers.shift_key {
                            None => { modifiers.shift_key = Some(Handedness::Left); }
                            Some(_) => { return Err(KeyBindingError(v)); }
                        },
                        KeyCode::ShiftRight => match modifiers.shift_key {
                            None => { modifiers.shift_key = Some(Handedness::Right); }
                            Some(_) => { return Err(KeyBindingError(v)); }
                        },
                        KeyCode::SuperLeft => match modifiers.super_key {
                            None => { modifiers.super_key = Some(Handedness::Left); }
                            Some(_) => { return Err(KeyBindingError(v)); }
                        },
                        KeyCode::SuperRight => match modifiers.super_key {
                            None => { modifiers.super_key = Some(Handedness::Right); }
                            Some(_) => { return Err(KeyBindingError(v)); }
                        },
                        c => { code = Some(c); }
                    },
                    Some(_) => { return Err(KeyBindingError(v)); }
                }
            }
        }

        match code {
            Some(code) => Ok(Self { code: *code, modifiers }),
            None => Err(KeyBindingError(v))
        }
    }
}
