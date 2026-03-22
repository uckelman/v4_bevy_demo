use bevy::{
    ecs::change_detection::Res,
    input::{
        ButtonInput,
        keyboard::KeyCode
    },
    prelude::Resource
};
use serde::Deserialize;
use std::fmt::{self, Display};

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Handedness {
    Left,
    Right,
    Either
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Modifiers {
    pub alt_key: Option<Handedness>,
    pub ctrl_key: Option<Handedness>,
    pub shift_key: Option<Handedness>,
    pub super_key: Option<Handedness>
}

pub trait ModifiersExt {
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

#[derive(Debug, thiserror::Error)]
#[error("invalid data {0:?}")]
pub struct KeyError(String);

impl TryFrom<String> for Key {
    type Error = KeyError;

    fn try_from(m: String) -> Result<Self, Self::Error> {
        match m.as_ref() {
            "Alt" => Ok(Key::Alt),
            "Ctrl" => Ok(Key::Ctrl),
            "Shift" => Ok(Key::Shift),
            "Super" => Ok(Key::Super),
            "`" => Ok(Key::Code(KeyCode::Backquote)),
            "\\" => Ok(Key::Code(KeyCode::Backslash)),
            "[" => Ok(Key::Code(KeyCode::BracketLeft)),
            "]" => Ok(Key::Code(KeyCode::BracketRight)),
            "," => Ok(Key::Code(KeyCode::Comma)),
            "0" => Ok(Key::Code(KeyCode::Digit0)),
            "1" => Ok(Key::Code(KeyCode::Digit1)),
            "2" => Ok(Key::Code(KeyCode::Digit2)),
            "3" => Ok(Key::Code(KeyCode::Digit3)),
            "4" => Ok(Key::Code(KeyCode::Digit4)),
            "5" => Ok(Key::Code(KeyCode::Digit5)),
            "6" => Ok(Key::Code(KeyCode::Digit6)),
            "7" => Ok(Key::Code(KeyCode::Digit7)),
            "8" => Ok(Key::Code(KeyCode::Digit8)),
            "9" => Ok(Key::Code(KeyCode::Digit9)),
            "=" => Ok(Key::Code(KeyCode::Equal)),
            "A" => Ok(Key::Code(KeyCode::KeyA)),
            "B" => Ok(Key::Code(KeyCode::KeyB)),
            "C" => Ok(Key::Code(KeyCode::KeyC)),
            "D" => Ok(Key::Code(KeyCode::KeyD)),
            "E" => Ok(Key::Code(KeyCode::KeyE)),
            "F" => Ok(Key::Code(KeyCode::KeyF)),
            "G" => Ok(Key::Code(KeyCode::KeyG)),
            "H" => Ok(Key::Code(KeyCode::KeyH)),
            "I" => Ok(Key::Code(KeyCode::KeyI)),
            "J" => Ok(Key::Code(KeyCode::KeyJ)),
            "K" => Ok(Key::Code(KeyCode::KeyK)),
            "L" => Ok(Key::Code(KeyCode::KeyL)),
            "M" => Ok(Key::Code(KeyCode::KeyM)),
            "N" => Ok(Key::Code(KeyCode::KeyN)),
            "O" => Ok(Key::Code(KeyCode::KeyO)),
            "P" => Ok(Key::Code(KeyCode::KeyP)),
            "Q" => Ok(Key::Code(KeyCode::KeyQ)),
            "R" => Ok(Key::Code(KeyCode::KeyR)),
            "S" => Ok(Key::Code(KeyCode::KeyS)),
            "T" => Ok(Key::Code(KeyCode::KeyT)),
            "U" => Ok(Key::Code(KeyCode::KeyU)),
            "V" => Ok(Key::Code(KeyCode::KeyV)),
            "W" => Ok(Key::Code(KeyCode::KeyW)),
            "X" => Ok(Key::Code(KeyCode::KeyX)),
            "Y" => Ok(Key::Code(KeyCode::KeyY)),
            "Z" => Ok(Key::Code(KeyCode::KeyZ)),
            "-" => Ok(Key::Code(KeyCode::Minus)),
            "." => Ok(Key::Code(KeyCode::Period)),
            "\"" => Ok(Key::Code(KeyCode::Quote)),
            ";" => Ok(Key::Code(KeyCode::Semicolon)),
            "/" => Ok(Key::Code(KeyCode::Slash)),
            "Left Alt" => Ok(Key::Code(KeyCode::AltLeft)),
            "Right Alt" => Ok(Key::Code(KeyCode::AltRight)),
            "Backspace" => Ok(Key::Code(KeyCode::Backspace)),
            "Caps Lock" => Ok(Key::Code(KeyCode::CapsLock)),
            "Left Ctrl" => Ok(Key::Code(KeyCode::ControlLeft)),
            "Right Ctrl" => Ok(Key::Code(KeyCode::ControlRight)),
            "Enter" => Ok(Key::Code(KeyCode::Enter)),
            "Left Super" => Ok(Key::Code(KeyCode::SuperLeft)),
            "Right Super" => Ok(Key::Code(KeyCode::SuperRight)),
            "Left Shift" => Ok(Key::Code(KeyCode::ShiftLeft)),
            "Right Shift" => Ok(Key::Code(KeyCode::ShiftRight)),
            "Space" => Ok(Key::Code(KeyCode::Space)),
            "Tab" => Ok(Key::Code(KeyCode::Tab)),
            "Del" => Ok(Key::Code(KeyCode::Delete)),
            "End" => Ok(Key::Code(KeyCode::End)),
            "Home" => Ok(Key::Code(KeyCode::Home)),
            "Ins" => Ok(Key::Code(KeyCode::Insert)),
            "PgDn" => Ok(Key::Code(KeyCode::PageDown)),
            "PgUp" => Ok(Key::Code(KeyCode::PageUp)),
            "Down Arrow" => Ok(Key::Code(KeyCode::ArrowDown)),
            "Left Arrow" => Ok(Key::Code(KeyCode::ArrowLeft)),
            "Right Arrow" => Ok(Key::Code(KeyCode::ArrowRight)),
            "Up Arrow" => Ok(Key::Code(KeyCode::ArrowUp)),
            "Num Lock" => Ok(Key::Code(KeyCode::NumLock)),
            "Numpad 0" => Ok(Key::Code(KeyCode::Numpad0)),
            "Numpad 1" => Ok(Key::Code(KeyCode::Numpad1)),
            "Numpad 2" => Ok(Key::Code(KeyCode::Numpad2)),
            "Numpad 3" => Ok(Key::Code(KeyCode::Numpad3)),
            "Numpad 4" => Ok(Key::Code(KeyCode::Numpad4)),
            "Numpad 5" => Ok(Key::Code(KeyCode::Numpad5)),
            "Numpad 6" => Ok(Key::Code(KeyCode::Numpad6)),
            "Numpad 7" => Ok(Key::Code(KeyCode::Numpad7)),
            "Numpad 8" => Ok(Key::Code(KeyCode::Numpad8)),
            "Numpad 9" => Ok(Key::Code(KeyCode::Numpad9)),
            "Numpad +" => Ok(Key::Code(KeyCode::NumpadAdd)),
            "Numpad ." => Ok(Key::Code(KeyCode::NumpadDecimal)),
            "Numpad /" => Ok(Key::Code(KeyCode::NumpadDivide)),
            "Numpad Enter" => Ok(Key::Code(KeyCode::NumpadEnter)),
            "Numpad =" => Ok(Key::Code(KeyCode::NumpadEqual)),
            "Numpad *" => Ok(Key::Code(KeyCode::NumpadMultiply)),
            "Numpad -" => Ok(Key::Code(KeyCode::NumpadSubtract)),
            "Esc" => Ok(Key::Code(KeyCode::Escape)),
            _ => Err(KeyError(m))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
enum Key {
    Alt,
    Ctrl,
    Shift,
    Super,
    Code(KeyCode)
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
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

impl Display for Handedness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Handedness::Left => write!(f, "Left "),
            Handedness::Right => write!(f, "Right "),
            Handedness::Either => Ok(())
        }
    }
}

impl Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ctrl_key) = self.modifiers.ctrl_key {
            write!(f, "{ctrl_key}")?;
            write!(f, "Ctrl+")?;
        }

        if let Some(alt_key) = self.modifiers.alt_key {
            write!(f, "{alt_key}")?;
            write!(f, "Alt+")?;
        }

        if let Some(shift_key) = self.modifiers.shift_key {
            write!(f, "{shift_key}")?;
            write!(f, "Shift+")?;
        }

        if let Some(super_key) = self.modifiers.super_key {
            write!(f, "{super_key}")?;
            write!(f, "Super+")?;
        }

        match self.code {
            KeyCode::Unidentified(_) => write!(f, "{:?}", self.code),
            KeyCode::Backquote => write!(f, "`"),
            KeyCode::Backslash => write!(f, "\\"),
            KeyCode::BracketLeft => write!(f, "["),
            KeyCode::BracketRight => write!(f, "]"),
            KeyCode::Comma => write!(f, ","),
            KeyCode::Digit0 => write!(f, "0"),
            KeyCode::Digit1 => write!(f, "1"),
            KeyCode::Digit2 => write!(f, "2"),
            KeyCode::Digit3 => write!(f, "3"),
            KeyCode::Digit4 => write!(f, "4"),
            KeyCode::Digit5 => write!(f, "5"),
            KeyCode::Digit6 => write!(f, "6"),
            KeyCode::Digit7 => write!(f, "7"),
            KeyCode::Digit8 => write!(f, "8"),
            KeyCode::Digit9 => write!(f, "9"),
            KeyCode::Equal => write!(f, "="),
            KeyCode::IntlBackslash => write!(f, "{:?}", self.code),
            KeyCode::IntlRo => write!(f, "{:?}", self.code),
            KeyCode::IntlYen => write!(f, "{:?}", self.code),
            KeyCode::KeyA => write!(f, "A"),
            KeyCode::KeyB => write!(f, "B"),
            KeyCode::KeyC => write!(f, "C"),
            KeyCode::KeyD => write!(f, "D"),
            KeyCode::KeyE => write!(f, "E"),
            KeyCode::KeyF => write!(f, "F"),
            KeyCode::KeyG => write!(f, "G"),
            KeyCode::KeyH => write!(f, "H"),
            KeyCode::KeyI => write!(f, "I"),
            KeyCode::KeyJ => write!(f, "J"),
            KeyCode::KeyK => write!(f, "K"),
            KeyCode::KeyL => write!(f, "L"),
            KeyCode::KeyM => write!(f, "M"),
            KeyCode::KeyN => write!(f, "N"),
            KeyCode::KeyO => write!(f, "O"),
            KeyCode::KeyP => write!(f, "P"),
            KeyCode::KeyQ => write!(f, "Q"),
            KeyCode::KeyR => write!(f, "R"),
            KeyCode::KeyS => write!(f, "S"),
            KeyCode::KeyT => write!(f, "T"),
            KeyCode::KeyU => write!(f, "U"),
            KeyCode::KeyV => write!(f, "V"),
            KeyCode::KeyW => write!(f, "W"),
            KeyCode::KeyX => write!(f, "X"),
            KeyCode::KeyY => write!(f, "Y"),
            KeyCode::KeyZ => write!(f, "Z"),
            KeyCode::Minus => write!(f, "-"),
            KeyCode::Period => write!(f, "."),
            KeyCode::Quote => write!(f, "\""),
            KeyCode::Semicolon => write!(f, ";"),
            KeyCode::Slash => write!(f, "/"),
            KeyCode::AltLeft => write!(f, "Left Alt"),
            KeyCode::AltRight => write!(f, "Right Alt"),
            KeyCode::Backspace => write!(f, "Backspace"),
            KeyCode::CapsLock => write!(f, "Caps Lock"),
            KeyCode::ContextMenu => write!(f, "{:?}", self.code),
            KeyCode::ControlLeft => write!(f, "Left Ctrl"),
            KeyCode::ControlRight => write!(f, "Right Ctrl"),
            KeyCode::Enter => write!(f, "Enter"),
            KeyCode::SuperLeft => write!(f, "Left Super"),
            KeyCode::SuperRight => write!(f, "Right Super"),
            KeyCode::ShiftLeft => write!(f, "Left Shift"),
            KeyCode::ShiftRight => write!(f, "Right Shift"),
            KeyCode::Space => write!(f, "Space"),
            KeyCode::Tab => write!(f, "Tab"),
            KeyCode::Convert => write!(f, "{:?}", self.code),
            KeyCode::KanaMode => write!(f, "{:?}", self.code),
            KeyCode::Lang1 => write!(f, "{:?}", self.code),
            KeyCode::Lang2 => write!(f, "{:?}", self.code),
            KeyCode::Lang3 => write!(f, "{:?}", self.code),
            KeyCode::Lang4 => write!(f, "{:?}", self.code),
            KeyCode::Lang5 => write!(f, "{:?}", self.code),
            KeyCode::NonConvert => write!(f, "{:?}", self.code),
            KeyCode::Delete => write!(f, "Del"),
            KeyCode::End => write!(f, "End"),
            KeyCode::Help => write!(f, "{:?}", self.code),
            KeyCode::Home => write!(f, "Home"),
            KeyCode::Insert => write!(f, "Ins"),
            KeyCode::PageDown => write!(f, "PgDn"),
            KeyCode::PageUp => write!(f, "PgUp"),
            KeyCode::ArrowDown => write!(f, "Down Arrow"),
            KeyCode::ArrowLeft => write!(f, "Left Arrow"),
            KeyCode::ArrowRight => write!(f, "Right Arrow"),
            KeyCode::ArrowUp => write!(f, "Up Arrow"),
            KeyCode::NumLock => write!(f, "Num Lock"),
            KeyCode::Numpad0 => write!(f, "Numpad 0"),
            KeyCode::Numpad1 => write!(f, "Numpad 1"),
            KeyCode::Numpad2 => write!(f, "Numpad 2"),
            KeyCode::Numpad3 => write!(f, "Numpad 3"),
            KeyCode::Numpad4 => write!(f, "Numpad 4"),
            KeyCode::Numpad5 => write!(f, "Numpad 5"),
            KeyCode::Numpad6 => write!(f, "Numpad 6"),
            KeyCode::Numpad7 => write!(f, "Numpad 7"),
            KeyCode::Numpad8 => write!(f, "Numpad 8"),
            KeyCode::Numpad9 => write!(f, "Numpad 9"),
            KeyCode::NumpadAdd => write!(f, "Numpad +"),
            KeyCode::NumpadBackspace => write!(f, "{:?}", self.code),
            KeyCode::NumpadClear => write!(f, "{:?}", self.code),
            KeyCode::NumpadClearEntry => write!(f, "{:?}", self.code),
            KeyCode::NumpadComma => write!(f, "{:?}", self.code),
            KeyCode::NumpadDecimal => write!(f, "Numpad ."),
            KeyCode::NumpadDivide => write!(f, "Numpad /"),
            KeyCode::NumpadEnter => write!(f, "Numpad Enter"),
            KeyCode::NumpadEqual => write!(f, "Numpad ="),
            KeyCode::NumpadHash => write!(f, "{:?}", self.code),
            KeyCode::NumpadMemoryAdd => write!(f, "{:?}", self.code),
            KeyCode::NumpadMemoryClear => write!(f, "{:?}", self.code),
            KeyCode::NumpadMemoryRecall => write!(f, "{:?}", self.code),
            KeyCode::NumpadMemoryStore => write!(f, "{:?}", self.code),
            KeyCode::NumpadMemorySubtract => write!(f, "{:?}", self.code),
            KeyCode::NumpadMultiply => write!(f, "Numpad *"),
            KeyCode::NumpadParenLeft => write!(f, "{:?}", self.code),
            KeyCode::NumpadParenRight => write!(f, "{:?}", self.code),
            KeyCode::NumpadStar => write!(f, "{:?}", self.code),
            KeyCode::NumpadSubtract => write!(f, "Numpad -"),
            KeyCode::Escape => write!(f, "Esc"),
            KeyCode::Fn => write!(f, "{:?}", self.code),
            KeyCode::FnLock => write!(f, "{:?}", self.code),
            KeyCode::PrintScreen => write!(f, "{:?}", self.code),
            KeyCode::ScrollLock => write!(f, "{:?}", self.code),
            KeyCode::Pause => write!(f, "{:?}", self.code),
            KeyCode::BrowserBack => write!(f, "{:?}", self.code),
            KeyCode::BrowserFavorites => write!(f, "{:?}", self.code),
            KeyCode::BrowserForward => write!(f, "{:?}", self.code),
            KeyCode::BrowserHome => write!(f, "{:?}", self.code),
            KeyCode::BrowserRefresh => write!(f, "{:?}", self.code),
            KeyCode::BrowserSearch => write!(f, "{:?}", self.code),
            KeyCode::BrowserStop => write!(f, "{:?}", self.code),
            KeyCode::Eject => write!(f, "{:?}", self.code),
            KeyCode::LaunchApp1 => write!(f, "{:?}", self.code),
            KeyCode::LaunchApp2 => write!(f, "{:?}", self.code),
            KeyCode::LaunchMail => write!(f, "{:?}", self.code),
            KeyCode::MediaPlayPause => write!(f, "{:?}", self.code),
            KeyCode::MediaSelect => write!(f, "{:?}", self.code),
            KeyCode::MediaStop => write!(f, "{:?}", self.code),
            KeyCode::MediaTrackNext => write!(f, "{:?}", self.code),
            KeyCode::MediaTrackPrevious => write!(f, "{:?}", self.code),
            KeyCode::Power => write!(f, "{:?}", self.code),
            KeyCode::Sleep => write!(f, "{:?}", self.code),
            KeyCode::AudioVolumeDown => write!(f, "{:?}", self.code),
            KeyCode::AudioVolumeMute => write!(f, "{:?}", self.code),
            KeyCode::AudioVolumeUp => write!(f, "{:?}", self.code),
            KeyCode::WakeUp => write!(f, "{:?}", self.code),
            KeyCode::Meta => write!(f, "Super"),
            KeyCode::Hyper => write!(f, "Hyper"),
            KeyCode::Turbo => write!(f, "{:?}", self.code),
            KeyCode::Abort => write!(f, "{:?}", self.code),
            KeyCode::Resume => write!(f, "{:?}", self.code),
            KeyCode::Suspend => write!(f, "{:?}", self.code),
            KeyCode::Again => write!(f, "{:?}", self.code),
            KeyCode::Copy => write!(f, "{:?}", self.code),
            KeyCode::Cut => write!(f, "{:?}", self.code),
            KeyCode::Find => write!(f, "{:?}", self.code),
            KeyCode::Open => write!(f, "{:?}", self.code),
            KeyCode::Paste => write!(f, "{:?}", self.code),
            KeyCode::Props => write!(f, "{:?}", self.code),
            KeyCode::Select => write!(f, "{:?}", self.code),
            KeyCode::Undo => write!(f, "{:?}", self.code),
            KeyCode::Hiragana => write!(f, "{:?}", self.code),
            KeyCode::Katakana => write!(f, "{:?}", self.code),
            KeyCode::F1 => write!(f, "{:?}", self.code),
            KeyCode::F2 => write!(f, "{:?}", self.code),
            KeyCode::F3 => write!(f, "{:?}", self.code),
            KeyCode::F4 => write!(f, "{:?}", self.code),
            KeyCode::F5 => write!(f, "{:?}", self.code),
            KeyCode::F6 => write!(f, "{:?}", self.code),
            KeyCode::F7 => write!(f, "{:?}", self.code),
            KeyCode::F8 => write!(f, "{:?}", self.code),
            KeyCode::F9 => write!(f, "{:?}", self.code),
            KeyCode::F10 => write!(f, "{:?}", self.code),
            KeyCode::F11 => write!(f, "{:?}", self.code),
            KeyCode::F12 => write!(f, "{:?}", self.code),
            KeyCode::F13 => write!(f, "{:?}", self.code),
            KeyCode::F14 => write!(f, "{:?}", self.code),
            KeyCode::F15 => write!(f, "{:?}", self.code),
            KeyCode::F16 => write!(f, "{:?}", self.code),
            KeyCode::F17 => write!(f, "{:?}", self.code),
            KeyCode::F18 => write!(f, "{:?}", self.code),
            KeyCode::F19 => write!(f, "{:?}", self.code),
            KeyCode::F20 => write!(f, "{:?}", self.code),
            KeyCode::F21 => write!(f, "{:?}", self.code),
            KeyCode::F22 => write!(f, "{:?}", self.code),
            KeyCode::F23 => write!(f, "{:?}", self.code),
            KeyCode::F24 => write!(f, "{:?}", self.code),
            KeyCode::F25 => write!(f, "{:?}", self.code),
            KeyCode::F26 => write!(f, "{:?}", self.code),
            KeyCode::F27 => write!(f, "{:?}", self.code),
            KeyCode::F28 => write!(f, "{:?}", self.code),
            KeyCode::F29 => write!(f, "{:?}", self.code),
            KeyCode::F30 => write!(f, "{:?}", self.code),
            KeyCode::F31 => write!(f, "{:?}", self.code),
            KeyCode::F32 => write!(f, "{:?}", self.code),
            KeyCode::F33 => write!(f, "{:?}", self.code),
            KeyCode::F34 => write!(f, "{:?}", self.code),
            KeyCode::F35 => write!(f, "{:?}", self.code)
        }
    }
}
