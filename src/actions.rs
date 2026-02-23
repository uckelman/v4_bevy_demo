use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        prelude::Commands
    },
    prelude::{Entity, EntityCommands, Query}
};
use itertools::Itertools;
use std::mem;

use crate::{
    action::{Action, PieceData},
    actionfunc::ActionFunc,
    clone::{CloneEvent, on_clone},
    delete::{DeleteEvent, on_delete},
    flip::{FlipEvent, on_flip},
    log::{ActionLog, handle_do},
    object::{NextObjectId, ObjectId, ObjectIdMap},
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
*/

pub fn make_action(
    entity: Entity,
    pid: u32, 
    action: ActionFunc,
    mut next_object_id: &mut ResMut<NextObjectId>,
) -> Action
{
    match action {
        ActionFunc::Clone => {
            let clone_id = next_object_id.0;
            next_object_id.0 += 1;
            Action::Clone(clone_id, pid)
        },
        ActionFunc::Delete => Action::Delete(PieceData {
            piece_id: pid
        }),
        ActionFunc::Flip(delta) => Action::Flip(pid, delta),
        ActionFunc::Rotate(dtheta) => Action::Rotate(pid, dtheta.0)
    }
}
