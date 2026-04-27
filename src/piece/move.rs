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

fn apply_move<const DO: bool>(
    entity: Entity,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    mut mov_query: Query<(&mut Parent, &mut Location, &mut Transform, &GlobalTransform)>,
    mut parent_query: Query<&GlobalTransform>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(mov) = edit.get(entity) else { return Ok(()); };

    // get the entity being edited
    let entity = *objmap.0.get(&mov.object_id).unwrap();

    let (mut mov_parent, mut mov_loc, mut mov_t, mov_gt) = mov_query.get_mut(entity)?;

    if mov.src_parent_id != mov.dst_parent_id {
        let new_parent_id = if DO { mov.dst_parent_id } else { mov.src_parent_id };

        let new_parent = *objmap.0.get(&new_parent_id).unwrap();

        // maintain the child's rotation
        let new_parent_gt = parent_query.get(new_parent)?;
        *mov_t = mov_gt.reparented_to(new_parent_gt);

        // reparent the child
        mov_parent.0 = new_parent;
        commands.entity(new_parent).add_child(entity);
    }

    // update the location
    let new_loc = if DO { mov.dst } else { mov.src };
    mov_loc.0 = new_loc;
    mov_t.translation = new_loc;

    Ok(())
}

#[instrument(skip_all)]
pub fn on_move_undo(
    evt: On<UndoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    dst_query: Query<(&mut Parent, &mut Location, &mut Transform, &GlobalTransform)>,
    src_query: Query<&GlobalTransform>,
    commands: Commands
) -> Result
{
    apply_move::<false>(
        evt.entity,
        edit,
        objmap,
        dst_query,
        src_query,
        commands
    )
}

#[instrument(skip_all)]
pub fn on_move_redo(
    evt: On<RedoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    src_query: Query<(&mut Parent, &mut Location, &mut Transform, &GlobalTransform)>,
    dst_query: Query<&GlobalTransform>,
    commands: Commands
) -> Result
{
    apply_move::<true>(
        evt.entity,
        edit,
        objmap,
        src_query,
        dst_query,
        commands
    )
}
