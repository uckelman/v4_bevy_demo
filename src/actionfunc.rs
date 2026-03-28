use bevy::{
    ecs::prelude::{EntityCommands, Commands},
    prelude::Entity
};
use itertools::Itertools;
use regex::Regex;
use serde::Deserialize;
use std::{
    mem,
    sync::LazyLock
};
use thiserror::Error;

use crate::{
    angle::Angle,
    piece::{
        clone::{DoCloneEvent, on_clone},
        delete::{DoDeleteEvent, on_delete},
        flip::{DoFlipEvent, on_flip},
        rotate::{DoRotateEvent, on_rotate}
    }
};

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

pub fn add_action_observer(
    action: ActionFunc,
    ec: &mut EntityCommands<'_>
)
{
    match action {
        ActionFunc::Clone => ec.observe(on_clone),
        ActionFunc::Delete => ec.observe(on_delete),
        ActionFunc::Flip(_) => ec.observe(on_flip),
        ActionFunc::Rotate(_) => ec.observe(on_rotate)
    };
}

pub fn add_action_observers<A>(
    actions: A,
    commands: &mut EntityCommands<'_>
)
where
    A: IntoIterator<Item = ActionFunc>
{
    // add each type of action once
    actions.into_iter()
        .unique_by(mem::discriminant)
        .for_each(|a| add_action_observer(a, commands));
}

pub fn trigger_action_func(
    entity: Entity,
    action: ActionFunc,
    commands: &mut Commands
)
{
    match action {
        ActionFunc::Clone => commands.trigger(DoCloneEvent { entity }),
        ActionFunc::Delete => commands.trigger(DoDeleteEvent { entity } ),
        ActionFunc::Flip(delta) => commands.trigger(DoFlipEvent { entity, delta } ),
        ActionFunc::Rotate(dtheta) => commands.trigger(DoRotateEvent { entity, dtheta: dtheta.0 })
    }
}
