use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{ChildOf, Commands, Query}
    },
    math::Vec3,
    prelude::{Entity, trace, Transform},
    sprite::Anchor
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    assets::SpriteHandles,
    edittype::EditType,
    gamebox::{self, GameBox},
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
    pub type_id: u32,
    pub parent_id: u32,
    pub location: Vec3,
    pub angle: f32,
    pub anchor: gamebox::Anchor,
    pub faceup: usize
}

// TODO: should pieces have an id for their piece type?
// or should we be able to get the face images from the piece somehow?

#[instrument(skip_all)]
pub fn on_delete(
    evt: On<DoDeleteEvent>,
    piece_query: Query<(&ObjectId, &PieceTypeId, &ChildOf, &Transform, &Anchor, &FaceUp)>,
    parent_query: Query<&ObjectId>,
    edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let (object_id, type_id, parent, t, anchor, faceup) = piece_query.get(entity)?;
    let parent_id = parent_query.get(parent.0)?;

    handle_do(
        edit_query,
        EditType::Delete,
        DeleteEdit {
            object_id: object_id.0,
            type_id: type_id.0,
            parent_id: parent_id.0,
            location: t.translation,
            angle: t.rotation.to_axis_angle().1,
            anchor: (*anchor).into(),
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
    objmap: Res<ObjectIdMap>,
    sprite_handles: Res<SpriteHandles>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(del) = edit.get(evt.entity) else { return Ok(()); };
    // get the parent entity
    let parent = *objmap.0.get(&del.parent_id).unwrap();

    // apply the change
    spawn_piece(
        del.object_id,
        del.type_id,
        &gamebox.piece[&del.type_id],
        parent,
        del.location,
        del.angle,
        del.anchor,
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
