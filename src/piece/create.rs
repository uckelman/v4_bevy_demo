use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        error::Result,
        event::{EntityEvent, Event},
        observer::On,
        prelude::{ChildOf, Commands, Entity, Query}
    },
    math::Vec3,
    prelude::{GlobalTransform, trace}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    assets::SpriteHandles,
    edittype::EditType,
    gamebox::{Anchor, GameBox},
    log::{EditIndex, Edits, handle_do},
    maxz::MaxZ,
    object::{NextObjectId, ObjectId, ObjectIdMap},
    piece::spawn_piece
};

#[derive(Clone, Event)]
pub struct DoCreateEvent {
    pub type_id: u32,
    pub parent: Entity,
    pub dst: Vec3,
    pub angle: f32,
    pub anchor: Anchor
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
    pub parent_id: u32,
    pub dst: Vec3,
    #[serde(default)]
    pub angle: f32,
    #[serde(default)]
    pub anchor: Anchor
}

#[instrument(skip_all)]
pub fn on_create(
    evt: On<DoCreateEvent>,
    mut next_object_id: ResMut<NextObjectId>,
    edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    parent_query: Query<&ObjectId>,
    commands: Commands
) -> Result
{
    trace!("");

    let object_id = next_object_id.0;
    next_object_id.0 += 1;

    let parent_id = parent_query.get(evt.parent)?;

    handle_do(
        edit_query,
        EditType::Create,
        CreateEdit {
            object_id,
            type_id: evt.type_id,
            parent_id: parent_id.0,
            dst: evt.dst,
            angle: evt.angle,
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
    root_query: Query<&ChildOf>,
    mut maxz_query: Query<&mut MaxZ>,
    global_transform_query: Query<&GlobalTransform>,
    gamebox: Res<GameBox>,
    objmap: Res<ObjectIdMap>,
    sprite_handles: Res<SpriteHandles>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cr) = edit.get(evt.entity) else { return Ok(()); };
    // get the parent entity
    let parent = *objmap.0.get(&cr.parent_id).unwrap();

    // update max z
    let root = root_query.root_ancestor(parent);
    let mut max_z = maxz_query.get_mut(root)?;

    let parent_t = global_transform_query.get(parent)?;
    let z = parent_t.translation().z + cr.dst.z;

    if z > max_z.0 {
        max_z.0 = z;
    }

    // apply the change
    spawn_piece(
        cr.object_id,
        cr.type_id,
        &gamebox.piece[&cr.type_id],
        parent,
        cr.dst,
        cr.angle,
        cr.anchor,
        0,
        &sprite_handles,
        &mut commands
    );
    Ok(())
}
