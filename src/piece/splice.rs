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
    prelude::trace
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    edittype::EditType,
    log::{EditIndex, Edits, handle_do},
    object::{ObjectId, ObjectIdMap},
    piece::{Above, Location}
};

#[derive(Clone, Debug, EntityEvent)]
pub struct DoSpliceEvent {
    pub entity: Entity,
    pub src_parent: Entity,
    pub src_child: Option<Entity>,
    pub src: Vec3,
    pub dst_parent: Entity,
    pub dst_child: Option<Entity>,
    pub dst: Vec3
}

#[derive(EntityEvent)]
pub struct UndoSpliceEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoSpliceEvent {
    pub entity: Entity
}

#[derive(Component, Debug, Deserialize, Serialize)]
#[serde(rename = "splice", tag = "type")]
pub struct SpliceEdit {
    pub object_id: u32,
    pub src_parent_id: u32,
    pub src_child_id: Option<u32>,
    pub src: Vec3,
    pub dst_parent_id: u32,
    pub dst_child_id: Option<u32>,
    pub dst: Vec3
}

#[instrument(skip_all)]
pub fn on_splice(
    evt: On<DoSpliceEvent>,
    piece_query: Query<&ObjectId>,
    edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let object_id = piece_query.get(entity)?.0;

    let src_parent_id = piece_query.get(evt.src_parent)?.0;

    let src_child_id = evt.src_child
        .map(|ch| piece_query.get(ch))
        .transpose()?
        .map(|ch| ch.0);

    let dst_parent_id = piece_query.get(evt.dst_parent)?.0;

    let dst_child_id = evt.dst_child
        .map(|ch| piece_query.get(ch))
        .transpose()?
        .map(|ch| ch.0);

    handle_do(
        edit_query,
        EditType::Splice,
        SpliceEdit {
            object_id,
            src_parent_id,
            src_child_id,
            src: evt.src,
            dst_parent_id,
            dst_child_id,
            dst: evt.dst
        },
        commands
    )
}

fn apply_splice<const DO: bool>(
    entity: Entity,
    edit: Query<&SpliceEdit>,
    objmap: Res<ObjectIdMap>,
    mut loc_query: Query<&mut Location>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(spl) = edit.get(entity) else { return Ok(()); };

    // get the entity being edited
    let entity = *objmap.0.get(&spl.object_id).unwrap();

    let (old_parent_id, old_child_id, new_parent_id, new_child_id) = if DO {
        (
            spl.src_parent_id,
            spl.src_child_id,
            spl.dst_parent_id,
            spl.dst_child_id
        )
    }
    else {
        (
            spl.dst_parent_id,
            spl.dst_child_id,
            spl.src_parent_id,
            spl.src_child_id
        )
    };

    // splice out the entity
    if let Some(old_child_id) = old_child_id {
        let old_parent = *objmap.0.get(&old_parent_id).unwrap();
        let old_child = *objmap.0.get(&old_child_id).unwrap();

        commands.entity(old_parent)
            .add_one_related::<Above>(old_child);

        let mut old_child_loc = loc_query.get_mut(old_child)?;
        old_child_loc.0 = if DO { spl.src } else { spl.dst };
    }

    // splice in the entity
    let new_parent = *objmap.0.get(&new_parent_id).unwrap();

    commands.entity(new_parent)
        .add_one_related::<Above>(entity);

    if let Some(new_child_id) = new_child_id {
        let new_child = *objmap.0.get(&new_child_id).unwrap();

        commands.entity(entity)
            .add_one_related::<Above>(new_child);
    }

    // update the location
    let mut loc = loc_query.get_mut(entity)?;
    loc.0 = if DO { spl.dst } else { spl.src };

    Ok(())
}

#[instrument(skip_all)]
pub fn on_splice_undo(
    evt: On<UndoSpliceEvent>,
    edit: Query<&SpliceEdit>,
    objmap: Res<ObjectIdMap>,
    dst_query: Query<&mut Location>,
    commands: Commands
) -> Result
{
    trace!("");

    apply_splice::<false>(
        evt.entity,
        edit,
        objmap,
        dst_query,
        commands
    )
}

#[instrument(skip_all)]
pub fn on_splice_redo(
    evt: On<RedoSpliceEvent>,
    edit: Query<&SpliceEdit>,
    objmap: Res<ObjectIdMap>,
    src_query: Query<&mut Location>,
    commands: Commands
) -> Result
{
    trace!("");

    apply_splice::<true>(
        evt.entity,
        edit,
        objmap,
        src_query,
        commands
    )
}
