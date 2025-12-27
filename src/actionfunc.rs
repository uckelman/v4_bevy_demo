use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;
use thiserror::Error;

use crate::angle::Angle;

#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum ActionFunc {
    Clone,
    Delete,
    Flip(i32),
    Rotate(Angle)
}

#[derive(Debug, Error)]
#[error("{0} is malformed")]
pub struct ActionFuncError(pub String);

impl TryFrom<String> for ActionFunc {
    type Error = ActionFuncError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        static FLIP: LazyLock<Regex> = LazyLock::new(||
            Regex::new(r#"^flip\((-?\d+)\)$"#).expect("bad regex")
        );

        static ROTATE: LazyLock<Regex> = LazyLock::new(||
            Regex::new(r#"^rotate\((-?\d+(\.\d+)?)\)$"#).expect("bad regex")
        );

        match s.as_ref() {
            "clone" => Ok(ActionFunc::Clone),
            "delete" => Ok(ActionFunc::Delete),
            s if s.starts_with("flip(") => {
                if let Some(c) = FLIP.captures(s) && let Some(m) = c.get(1) {
                    m.as_str()
                        .parse::<i32>()
                        .map(|n| ActionFunc::Flip(n))
                        .map_err(|_| ActionFuncError(s.into()))
                }
                else {
                    Err(ActionFuncError(s.into()))
                }
            },
            s if s.starts_with("rotate(") => {
                if let Some(c) = ROTATE.captures(s) && let Some(m) = c.get(1) {
                    m.as_str()
                        .parse::<f32>()
                        .ok()
                        .and_then(Angle::new)
                        .and_then(|a| Some(ActionFunc::Rotate(a)))
                        .ok_or_else(|| ActionFuncError(s.into()))
                }
                else {
                    Err(ActionFuncError(s.into()))
                }
            },
            _ => Err(ActionFuncError(s.into()))
        }
    }
}
