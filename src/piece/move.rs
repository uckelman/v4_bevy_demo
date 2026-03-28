use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Commands, Query}
    },
    math::Vec3,
    prelude::{Entity, trace, Transform}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    edittype::EditType,
    log::{EditIndex, Edits, handle_do},
    object::{ObjectId, ObjectIdMap}
};

#[derive(Clone, EntityEvent)]
pub struct DoMoveEvent {
    pub entity: Entity,
    pub src: Vec3,
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

fn do_move(t: &mut Transform, to: Vec3)
{
    t.translation = to;
}

#[derive(Component, Debug, Deserialize, Serialize)]
#[serde(rename = "move", tag = "type")]
pub struct MoveEdit {
    pub object_id: u32,
    pub src: Vec3,
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
    let object_id = piece_query.get(entity)?;

    handle_do(
        edit_query,
        EditType::Move,
        MoveEdit { object_id: object_id.0, src: evt.src, dst: evt.dst },
        commands
    )
}

#[instrument(skip_all)]
pub fn on_move_undo(
    evt: On<UndoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    mut query: Query<&mut Transform>
) -> Result
{
    // get the edit
    let Ok(mov) = edit.get(evt.entity) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&mov.object_id).unwrap();
    // get the components of the entity being edited
    let mut t = query.get_mut(entity)?;
    // apply the change to the entity
    do_move(&mut t, mov.src);
    Ok(())
}

#[instrument(skip_all)]
pub fn on_move_redo(
    evt: On<RedoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    mut query: Query<&mut Transform>
) -> Result
{
    // get the edit
    let Ok(mov) = edit.get(evt.entity) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&mov.object_id).unwrap();
    // get the components of the entity being edited
    let mut t = query.get_mut(entity)?;
    // apply the change to the entity
    do_move(&mut t, mov.dst);
    Ok(())
}
