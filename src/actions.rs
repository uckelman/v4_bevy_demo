use bevy::{
    ecs::prelude::Commands,
    prelude::{Entity, EntityCommands}
};
use itertools::Itertools;
use std::mem;

use crate::{
    actionfunc::ActionFunc,
    clone::{CloneEvent, on_clone},
    delete::{DeleteEvent, on_delete},
    flip::{FlipEvent, on_flip},
    rotate::{RotateEvent, on_rotate}
};

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
    mut commands: &mut EntityCommands<'_>
)
where
    A: IntoIterator<Item = ActionFunc>
{
    // add each type of action once
    actions.into_iter()
        .unique_by(mem::discriminant)
        .for_each(|a| add_action_observer(a, commands));
}

pub fn trigger_action(
    entity: Entity,
    action: ActionFunc,
    commands: &mut Commands
)
{
    match action {
        ActionFunc::Clone => commands.trigger(CloneEvent { entity }),
        ActionFunc::Delete => commands.trigger(DeleteEvent { entity }),
        ActionFunc::Flip(delta) => commands.trigger(FlipEvent { entity, delta }),
        ActionFunc::Rotate(dtheta) => commands.trigger(RotateEvent { entity, dtheta: dtheta.0 })
    }
}
