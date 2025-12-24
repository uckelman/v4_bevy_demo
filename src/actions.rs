use bevy::{
    ecs::prelude::Commands,
    prelude::{Entity, EntityCommands}
};

use crate::{
    clone::{CloneEvent, on_clone},
    delete::{DeleteEvent, on_delete},
    flip::{FlipForwardEvent, FlipBackEvent, on_flip_forward, on_flip_back}
};

pub fn add_action_observer<S: AsRef<str>>(
    name: S,
    ec: &mut EntityCommands<'_>
)
{
    match name.as_ref() {
        "clone" => ec.observe(on_clone),
        "delete" => ec.observe(on_delete),
        "flip_forward" => ec.observe(on_flip_forward),
        "flip_back" => ec.observe(on_flip_back),
        _ => todo!()
    };
}

pub fn add_action_observers<S, A>(
    actions: A,
    mut commands: &mut EntityCommands<'_>
)
where
    S: AsRef<str>,
    A: IntoIterator<Item = S>
{
    actions.into_iter()
        .for_each(|a| add_action_observer(a, &mut commands));
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
        "flip_forward" => commands.trigger(FlipForwardEvent { entity }),
        "flip_back" => commands.trigger(FlipBackEvent { entity }),
        _ => todo!()
    };
}
