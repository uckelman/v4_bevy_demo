use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        error::Result,
        event::{EntityEvent, Event},
        observer::On,
        prelude::{Commands, Entity, Query}
    },
    math::Vec3,
    prelude::{Sprite, trace}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    assets::SpriteHandles,
    edittype::EditType,
    gamebox::{Anchor, GameBox},
    log::{EditIndex, Edits, handle_do},
    map::spawn_map,
    object::{NextObjectId, ObjectIdMap}
};

#[derive(Clone, Event)]
pub struct DoCreateEvent {
    pub type_id: u32,
    pub dst: Vec3,
    pub angle: f32,
    pub scale: f32,
    pub anchor: Anchor,
    pub parent_id: u32
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
#[serde(rename = "create_map", tag = "type")]
pub struct CreateEdit {
    pub object_id: u32,
    pub type_id: u32,
    pub parent_id: u32,
    pub dst: Vec3,
    pub angle: f32,
    pub scale: f32,
    pub anchor: Anchor
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
        EditType::CreateMap,
        CreateEdit {
            object_id,
            type_id: evt.type_id,
            parent_id: evt.parent_id,
            dst: evt.dst,
            angle: evt.angle,
            scale: evt.scale,
            anchor: evt.anchor
        },
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
    gamebox: Res<GameBox>,
    sprite_handles: Res<SpriteHandles>,
    objmap: Res<ObjectIdMap>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cr) = edit.get(evt.entity) else { return Ok(()); };
    // get the parent
    let parent = *objmap.0.get(&cr.parent_id).unwrap();
    // apply the change
    spawn_map(
        cr.object_id,
        cr.type_id,
        &gamebox.map[&cr.type_id],
        parent,
        cr.dst,
        cr.angle,
        cr.scale,
        cr.anchor,
        &sprite_handles,
        &mut commands
    );
    Ok(())
}
