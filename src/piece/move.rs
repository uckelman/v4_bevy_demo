use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Commands, Entity, Query}
    },
    math::Vec3,
    prelude::{GlobalTransform, trace, Transform}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    edittype::EditType,
    log::{EditIndex, Edits, handle_do},
    object::{ObjectId, ObjectIdMap},
    piece::{Location, Parent}
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

#[instrument(skip_all)]
pub fn on_move_undo(
    evt: On<UndoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    mut dst_query: Query<(&mut Parent, &mut Location, &mut Transform, &GlobalTransform)>,
    mut src_query: Query<&GlobalTransform>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(mov) = edit.get(evt.entity) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&mov.object_id).unwrap();

    let (mut dst_parent, mut dst_loc, mut dst_t, dst_gt) = dst_query.get_mut(entity)?;

    if mov.src_parent_id != mov.dst_parent_id {
        let src_parent = *objmap.0.get(&mov.src_parent_id).unwrap();

        // maintain the child's rotation
        let src_gt = src_query.get(src_parent)?;
        *dst_t = dst_gt.reparented_to(src_gt);

        // reparent the child
        dst_parent.0 = src_parent;
        commands.entity(src_parent).add_child(entity);
    }

    // update the location
    dst_loc.0 = mov.src;
    dst_t.translation = mov.src;

    Ok(())
}

#[instrument(skip_all)]
pub fn on_move_redo(
    evt: On<RedoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    mut src_query: Query<(&mut Parent, &mut Location, &mut Transform, &GlobalTransform)>,
    mut dst_query: Query<&GlobalTransform>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(mov) = edit.get(evt.entity) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&mov.object_id).unwrap();

    let (mut src_parent, mut src_loc, mut src_t, src_gt) = src_query.get_mut(entity)?;

    if mov.src_parent_id != mov.dst_parent_id {
        let dst_parent = *objmap.0.get(&mov.dst_parent_id).unwrap();

        // maintain the child's rotation
        let dst_gt = dst_query.get(dst_parent)?;
        *src_t = src_gt.reparented_to(dst_gt);

        // reparent the child
        src_parent.0 =  dst_parent;
        commands.entity(dst_parent).add_child(entity);
    }

    // update the location
    src_loc.0 = mov.dst;
    src_t.translation = mov.dst;

    Ok(())
}
