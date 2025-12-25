use bevy::{
    ecs::prelude::Commands,
    prelude::{Entity, EntityCommands}
};
use std::collections::HashSet;

use crate::{
    clone::{CloneEvent, on_clone},
    delete::{DeleteEvent, on_delete},
    flip::{FlipEvent, on_flip},
    rotate::{RotateEvent, on_rotate}
};

pub fn add_action_observer<S: AsRef<str>>(
    name: S,
    ec: &mut EntityCommands<'_>
)
{
    match name.as_ref() {
        "clone" => ec.observe(on_clone),
        "delete" => ec.observe(on_delete),
        "flip" => ec.observe(on_flip),
        "rotate" => ec.observe(on_rotate),
        _ => todo!()
    };
}

pub fn add_action_observers<S, A>(
    actions: A,
    mut commands: &mut EntityCommands<'_>
)
where
    S: Into<String> + AsRef<str>,
    A: IntoIterator<Item = S>
{
    actions.into_iter()
        .map(|a| match a.as_ref() {
            a if a.starts_with("flip(") => "flip".to_string(),
            a if a.starts_with("rotate(") => "rotate".to_string(),
            a => a.to_string()
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .for_each(|a| add_action_observer(a, commands));
}

pub fn trigger_action(
    entity: Entity,
    name: &str,
    commands: &mut Commands
)
{
    match name {
        "clone" => commands.trigger(CloneEvent { entity }),
        "delete" => commands.trigger(DeleteEvent { entity }),
        "flip(1)" => commands.trigger(FlipEvent { entity, delta: 1 }),
        "flip(-1)" => commands.trigger(FlipEvent { entity, delta: -1 }),
        "rotate(60)" => commands.trigger(RotateEvent { entity, dtheta: 60.0 }),
        "rotate(-60)" => commands.trigger(RotateEvent { entity, dtheta: -60.0 }),
        _ => todo!()
    };
}
