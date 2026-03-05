use bevy::prelude::EntityCommands;
use itertools::Itertools;
use std::mem;

use crate::{
    actionfunc::ActionFunc,
    clone::on_clone,
    delete::on_delete,
    flip::on_flip,
    rotate::on_rotate
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

// TODO: insert edit components; for groups, make them children?
