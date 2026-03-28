use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        event::{EntityEvent, Event},
        error::Result,
        observer::On,
        prelude::{Commands, Query}
    },
    math::Vec3,
    prelude::{Entity, trace}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    assets::SpriteHandles,
    edittype::EditType,
    gamebox::GameBox,
    log::{EditIndex, Edits, handle_do},
    object::{NextObjectId, ObjectIdMap},
    piece::spawn_piece
};

#[derive(Clone, Event)]
pub struct DoCreateEvent {
    pub type_id: u32,
    pub dst: Vec3
}

#[derive(EntityEvent)]
pub struct UndoCreateEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoCreateEvent {
    pub entity: Entity
}

#[derive(Component, Debug, Deserialize, Serialize)]
#[serde(rename = "create", tag = "type")]
pub struct CreateEdit {
    pub object_id: u32,
    pub type_id: u32,
    pub dst: Vec3
}

#[instrument(skip_all)]
pub fn on_create(
    evt: On<DoCreateEvent>,
    mut next_object_id: ResMut<NextObjectId>,
    edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    commands: Commands
) -> Result
{
    trace!("");

    let object_id = next_object_id.0;
    next_object_id.0 += 1;

    handle_do(
        edit_query,
        EditType::Create,
        CreateEdit { object_id, type_id: evt.type_id, dst: evt.dst },
        commands
    )
}

#[instrument(skip_all)]
pub fn on_create_undo(
    evt: On<UndoCreateEvent>,
    edit: Query<&CreateEdit>,
    objmap: Res<ObjectIdMap>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cr) = edit.get(evt.entity) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&cr.object_id).unwrap();
    // apply the change
    commands.entity(entity).despawn();
    Ok(())
}

#[instrument(skip_all)]
pub fn on_create_redo(
    evt: On<RedoCreateEvent>,
    edit: Query<&CreateEdit>,
    gamebox: Res<GameBox>,
    sprite_handles: Res<SpriteHandles>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cr) = edit.get(evt.entity) else { return Ok(()); };
    // apply the change
    spawn_piece(
        cr.object_id,
        cr.type_id,
        &gamebox.piece[&cr.type_id],
        cr.dst,
        0.0,
        0,
        &sprite_handles,
        &mut commands
    );
    Ok(())
}
