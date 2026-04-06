use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        error::Result,
        event::{EntityEvent, Event},
        observer::On,
        prelude::{Commands, Entity, Query, With}
    },
    prelude::trace,
    window::{PrimaryWindow, Window}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    edittype::EditType,
    log::{EditIndex, Edits, handle_do},
    object::{NextObjectId, ObjectIdMap},
    surface::spawn_surface
};

#[derive(Clone, Event)]
pub struct DoCreateEvent {
    pub type_id: u32
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
#[serde(rename = "create_surface", tag = "type")]
pub struct CreateEdit {
    pub object_id: u32,
    pub type_id: u32
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
        EditType::CreateSurface,
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
// TODO: the edit not existing should be impossible, maybe we should panic?
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
    q_window: Query<Entity, (With<Window>, With<PrimaryWindow>)>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cr) = edit.get(evt.entity) else { return Ok(()); };

    let window = q_window.single()?;

    // apply the change
    spawn_surface(cr.object_id, window, &mut commands);
    Ok(())
}
