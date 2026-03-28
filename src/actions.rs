use bevy::{
    ecs::prelude::{EntityCommands, Commands},
    prelude::Entity
};
use itertools::Itertools;
use std::mem;

use crate::{
    actionfunc::ActionFunc,
    piece::{
        clone::{DoCloneEvent, on_clone},
        delete::{DoDeleteEvent, on_delete},
        flip::{DoFlipEvent, on_flip},
        rotate::{DoRotateEvent, on_rotate}
    }
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
