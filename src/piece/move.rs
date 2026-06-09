use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Changed, ChildOf, Commands, Entity, Or, Query, With, Without}
    },
    math::Vec3,
    prelude::{GlobalTransform, trace, Transform}
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    edittype::EditType,
    log::{EditIndex, Edits, handle_do},
    object::{ObjectId, ObjectIdMap},
    piece::{Above, Below, Location, StackingGroup},
    stack::{self, Expanded, StackBelowQueryExt}
};

#[derive(Clone, Debug, EntityEvent)]
pub struct DoMoveEvent {
    pub entity: Entity,
    pub src_parent: Entity,
    pub src: Vec3,
    pub dst_parent: Entity,
    pub dst: Vec3
}

#[derive(EntityEvent)]
pub struct UndoMoveEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoMoveEvent {
    pub entity: Entity
}

#[derive(Component, Debug, Deserialize, Serialize)]
#[serde(rename = "move", tag = "type")]
pub struct MoveEdit {
    pub object_id: u32,
    pub src_parent_id: u32,
    pub src: Vec3,
    pub dst_parent_id: u32,
    pub dst: Vec3
}

#[instrument(skip_all)]
pub fn on_move(
    evt: On<DoMoveEvent>,
    piece_query: Query<&ObjectId>,
    edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let object_id = piece_query.get(entity)?.0;
    let src_parent_id = piece_query.get(evt.src_parent)?.0;
    let dst_parent_id = piece_query.get(evt.dst_parent)?.0;

    handle_do(
        edit_query,
        EditType::Move,
        MoveEdit {
            object_id,
            src_parent_id,
            src: evt.src,
            dst_parent_id,
            dst: evt.dst
        },
        commands
    )
}

fn apply_move<const DO: bool>(
    entity: Entity,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    mut mov_query: Query<&mut Location>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(mov) = edit.get(entity) else { return Ok(()); };

    // get the entity being edited
    let entity = *objmap.0.get(&mov.object_id).unwrap();

    let mut mov_loc = mov_query.get_mut(entity)?;

    if mov.src_parent_id != mov.dst_parent_id {
        let new_parent_id = if DO { mov.dst_parent_id } else { mov.src_parent_id };

        let new_parent = *objmap.0.get(&new_parent_id).unwrap();

        // reparent the child
        commands.entity(new_parent)
            .add_one_related::<Above>(entity);
    }

    // update the location
    mov_loc.0 = if DO { mov.dst } else { mov.src };

    Ok(())
}

#[instrument(skip_all)]
pub fn on_move_undo(
    evt: On<UndoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    dst_query: Query<&mut Location>,
    commands: Commands
) -> Result
{
    apply_move::<false>(
        evt.entity,
        edit,
        objmap,
        dst_query,
        commands
    )
}

#[instrument(skip_all)]
pub fn on_move_redo(
    evt: On<RedoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    src_query: Query<&mut Location>,
    commands: Commands
) -> Result
{
    apply_move::<true>(
        evt.entity,
        edit,
        objmap,
        src_query,
        commands
    )
}

#[instrument(skip_all)]
pub fn on_location_change(
    mut query: Query<(Entity, &ChildOf, &mut Transform, &GlobalTransform, &Above, &Location), (Without<Expanded>, Or<(Changed<Above>, Changed<Location>)>)>,
    gt_query: Query<&GlobalTransform>,
    mut commands: Commands
) -> Result
{
    trace!("");

    for (e, par_ui, mut t, gt, par_g, loc) in query.iter_mut() {
        // destination is not an expanded stack

        // update the parent
        if par_g.0 != par_ui.0 {
            // maintain the child's rotation
            let par_g_gt = gt_query.get(par_g.0)?;
            *t = gt.reparented_to(par_g_gt);

            // reparent the child
            commands.entity(par_g.0).add_child(e);
        }

        // update the location
        t.translation = loc.0;
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn on_stack_change(
    query: Query<Entity, (With<Expanded>, Or<(Changed<Below>, Changed<Above>)>)>,
    a_query: Query<(Option<&Above>, &StackingGroup)>,
    mut commands: Commands
)
{
    trace!("");

    query.iter()
        .map(|e| a_query.bottom(e))
        .unique()
        .for_each(|e| stack::restack_stack(e, &mut commands));
}
