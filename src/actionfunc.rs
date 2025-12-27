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
            Regex::new(r#"^(-?\d+)\)$"#).expect("bad regex")
        );

        static ROTATE: LazyLock<Regex> = LazyLock::new(||
            Regex::new(r#"^(-?\d+(\.\d+)?)\)$"#).expect("bad regex")
        );

        let (fname, args) = s.split_once('(').unwrap_or((&s, ""));

        match fname {
            "clone" => Ok(ActionFunc::Clone),
            "delete" => Ok(ActionFunc::Delete),
            "flip" => FLIP.captures(args)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str()
                    .parse::<i32>()
                    .ok()
                    .map(ActionFunc::Flip)
                )
                .ok_or(ActionFuncError(s)),
            "rotate" => ROTATE.captures(args)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str()
                    .parse::<f32>()
                    .ok()
                    .and_then(Angle::new)
                    .map(ActionFunc::Rotate)
                )
                .ok_or(ActionFuncError(s)),
            _ => Err(ActionFuncError(s))
        }
    }
}
