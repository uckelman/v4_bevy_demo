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
    assets::SpriteHandles,
    edittype::EditType,
    gamebox::GameBox,
    log::{EditIndex, Edits, handle_do},
    object::{ObjectId, ObjectIdMap},
    piece::{FaceUp, PieceTypeId, spawn_piece}
};

#[derive(Clone, EntityEvent)]
pub struct DoDeleteEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct UndoDeleteEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoDeleteEvent {
    pub entity: Entity
}

fn do_delete(
    entity: Entity,
    commands: &mut Commands
)
{
    commands.entity(entity).despawn();
}

#[derive(Component, Debug, Deserialize, Serialize)]
#[serde(rename = "delete", tag = "type")]
pub struct DeleteEdit {
    pub object_id: u32,
    pub ptype_id: u32,
    pub location: Vec3,
    pub angle: f32,
    pub faceup: usize
}

// TODO: should pieces have an id for their piece type?
// or should we be able to get the face images from the piece somehow?

#[instrument(skip_all)]
pub fn on_delete(
    evt: On<DoDeleteEvent>,
    piece_query: Query<(&ObjectId, &PieceTypeId, &Transform, &FaceUp)>,
    edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let (object_id, ptype_id, t, faceup) = piece_query.get(entity)?;

    handle_do(
        edit_query,
        EditType::Delete,
        DeleteEdit {
            object_id: object_id.0,
            ptype_id: ptype_id.0,
            location: t.translation,
            angle: t.rotation.to_axis_angle().1,
            faceup: faceup.0
        },
        commands
    )
}

#[instrument(skip_all)]
pub fn on_delete_undo(
    evt: On<UndoDeleteEvent>,
    edit: Query<&DeleteEdit>,
    gamebox: Res<GameBox>,
    sprite_handles: Res<SpriteHandles>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(del) = edit.get(evt.entity) else { return Ok(()); };

    // apply the change
    spawn_piece(
        del.object_id,
        del.ptype_id,
        &gamebox.piece[&del.ptype_id],
        del.location,
        del.angle,
        del.faceup,
        &sprite_handles,
        &mut commands
    );

    Ok(())
}

#[instrument(skip_all)]
pub fn on_delete_redo(
    evt: On<RedoDeleteEvent>,
    edit: Query<&DeleteEdit>,
    objmap: Res<ObjectIdMap>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(del) = edit.get(evt.entity) else { return Ok(()); };
    // get the source entity
    let entity = *objmap.0.get(&del.object_id).unwrap();
    // apply the change
    do_delete(entity, &mut commands);
    Ok(())
}
