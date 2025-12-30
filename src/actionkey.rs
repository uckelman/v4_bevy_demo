use bevy::input::keyboard::KeyCode;
use serde::Deserialize;
use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize)]
pub struct MaybeActionKey {
    #[serde(default)]
    pub modifiers: Vec<String>,
    pub key: String
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(try_from = "MaybeActionKey")]
pub struct ActionKey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: KeyCode
}

#[derive(Debug, Error)]
#[error("{0:?} is malformed")]
pub struct ActionKeyError(pub MaybeActionKey);

impl TryFrom<MaybeActionKey> for ActionKey {
    type Error = ActionKeyError;

    fn try_from(ma: MaybeActionKey) -> Result<Self, Self::Error> {
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;

        for m in &ma.modifiers {
            match m.as_ref() {
                "Ctrl" => { ctrl = true; },
                "Alt" => { alt = true; },
                "Shift" => { shift = true; },
                _ => return Err(ActionKeyError(ma))
            }
        }

        let key = match ma.key.as_ref() {
            "]" => KeyCode::BracketRight,
            "C" => KeyCode::KeyC,
            "Del" => KeyCode::Delete,
            "," => KeyCode::Comma,
            "." => KeyCode::Period,
            _ => return Err(ActionKeyError(ma))
        };

        Ok(Self { ctrl, alt, shift, key })
    }
}

impl Display for ActionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            write!(f, "Ctrl+")?;
        }

        if self.alt {
            write!(f, "Alt+")?;
        }

        if self.shift {
            write!(f, "Shift+")?;
        }

        let key = match self.key {
            KeyCode::BracketRight => "]",
            KeyCode::KeyC => "C",
            KeyCode::Delete => "Del",
            KeyCode::Comma => ",",
            KeyCode::Period => ".",
            _ => unreachable!()
        };

        write!(f, "{key}")
    }
}


