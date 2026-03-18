use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Commands, Query}
    },
    input::keyboard::KeyCode,
    prelude::{Entity, Resource, trace, Transform}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    keys::KeyConfig,
    log::{DoRotateEvent, EditIndex, EditType, Edits, handle_do, RedoRotateEvent, UndoRotateEvent},
    object::{ObjectId, ObjectIdMap}
};

#[derive(Resource)]
pub struct RotateCWKey(pub KeyCode);

#[derive(Resource)]
pub struct RotateCCWKey(pub KeyCode);

impl KeyConfig for RotateCWKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for RotateCCWKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

fn do_rotate(t: &mut Transform, dtheta: f32)
{
    use std::f32::consts::PI;
    const DEG_TO_RAD: f32 = PI / 180.0;

    t.rotate_local_z(dtheta * DEG_TO_RAD);
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
    mut query: Query<&mut Transform>,
    dir: f32
) -> Result
{
    // get the edit
    let Ok(rot) = edit.get(event_target) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&rot.object_id).unwrap();
    // get the components of the entity being edited
    let mut t = query.get_mut(entity)?;
    // apply the change to the entity
    do_rotate(&mut t, dir * rot.dtheta);
    Ok(())
}

#[instrument(skip_all)]
pub fn on_rotate_undo(
    evt: On<UndoRotateEvent>,
    edit: Query<&RotateEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<&mut Transform>
) -> Result
{
    apply_rotate(evt.entity, edit, objmap, query, -1.0)
}

#[instrument(skip_all)]
pub fn on_rotate_redo(
    evt: On<RedoRotateEvent>,
    edit: Query<&RotateEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<&mut Transform>
) -> Result
{
    apply_rotate(evt.entity, edit, objmap, query, 1.0)
}
