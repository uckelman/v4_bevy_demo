use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Commands, Query}
    },
    prelude::{Entity, trace, Transform}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    edittype::EditType,
    log::{EditIndex, Edits, handle_do},
    object::{ObjectId, ObjectIdMap},
    piece::Angle
};

#[derive(Clone, EntityEvent)]
pub struct DoRotateEvent {
    pub entity: Entity,
    pub dtheta: f32
}

#[derive(EntityEvent)]
pub struct UndoRotateEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoRotateEvent {
    pub entity: Entity
}

#[derive(Component, Debug, Deserialize, Serialize)]
#[serde(rename = "rotate", tag = "type")]
pub struct RotateEdit {
    pub object_id: u32,
    pub dtheta: f32
}

#[instrument(skip_all)]
pub fn on_rotate(
    evt: On<DoRotateEvent>,
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
        EditType::Rotate,
        RotateEdit { object_id: object_id.0, dtheta: evt.dtheta },
        commands
    )
}

fn apply_rotate(
    event_target: Entity,
    edit: Query<&RotateEdit>,
    objmap: Res<ObjectIdMap>,
    mut query: Query<(&mut Angle, &mut Transform)>,
    dir: f32
) -> Result
{
    // get the edit
    let Ok(rot) = edit.get(event_target) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&rot.object_id).unwrap();
    // get the components of the entity being edited
    let (mut a, mut t) = query.get_mut(entity)?;

    // apply the change to the entity
    use std::f32::consts::PI;
    const DEG_TO_RAD: f32 = PI / 180.0;

    a.0 = dir * rot.dtheta;
    t.rotate_local_z(a.0 * DEG_TO_RAD);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_rotate_undo(
    evt: On<UndoRotateEvent>,
    edit: Query<&RotateEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<(&mut Angle, &mut Transform)>,
) -> Result
{
    apply_rotate(evt.entity, edit, objmap, query, -1.0)
}

#[instrument(skip_all)]
pub fn on_rotate_redo(
    evt: On<RedoRotateEvent>,
    edit: Query<&RotateEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<(&mut Angle, &mut Transform)>,
) -> Result
{
    apply_rotate(evt.entity, edit, objmap, query, 1.0)
}
