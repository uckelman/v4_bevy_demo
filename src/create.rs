use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        error::Result,
        observer::On,
        prelude::{Commands, Query}
    },
    math::Vec3,
    prelude::{Entity, trace}
};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    Surface,
    assets::SpriteHandles,
    gamebox::GameBox,
    log::{DoCreateEvent, EditIndex, EditType, Edits, handle_do, RedoCreateEvent, UndoCreateEvent},
    object::{NextObjectId, ObjectIdMap},
    piece::spawn_piece
};

#[derive(Component, Deserialize, Serialize)]
#[serde(rename = "create", tag = "type")]
pub struct CreateEdit {
    pub object_id: u32,
    pub type_id: u32
}


// TODO: add initial location
fn do_create(
    object_id: u32,
    type_id: u32,
    gamebox: &GameBox,
    sprite_handles: &SpriteHandles,
    surface: &mut Surface,
    commands: &mut Commands
)
{
    let piece_type = &gamebox.piece[&type_id];

    let mut rng = rand::rng();

    let x = rng.random_range(-500.0..=500.0);
    let y = rng.random_range(-500.0..=500.0);

    surface.max_z += 1.0;

    spawn_piece(
        object_id,
        type_id,
        piece_type,
        Vec3::new(x, y, surface.max_z),
        0.0,
        0,
        sprite_handles,
        commands
    );
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
        CreateEdit { object_id, type_id: evt.type_id },
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
    mut surface: ResMut<Surface>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cr) = edit.get(evt.entity) else { return Ok(()); };
    // apply the change
    do_create(
        cr.object_id,
        cr.type_id,
        &gamebox,
        &sprite_handles,
        &mut surface,
        &mut commands
    );
    Ok(())
}
