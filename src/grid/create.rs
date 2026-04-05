use bevy::{
    asset::Assets,
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        error::Result,
        event::{EntityEvent, Event},
        observer::On,
        prelude::{Commands, Entity, Query}
    },
    math::{Quat, Vec3},
    mesh::Mesh,
    prelude::{ColorMaterial, trace, Transform}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    edittype::EditType,
    gamebox::{GameBox, GridDefinition},
    grid::spawn_grid,
    log::{EditIndex, Edits, handle_do},
    object::{NextObjectId, ObjectIdMap}
};

#[derive(Clone, Event)]
pub struct DoCreateEvent {
    pub type_id: u32,
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
#[serde(rename = "create_grid", tag = "type")]
pub struct CreateEdit {
    pub object_id: u32,
    pub type_id: u32,
    pub parent_id: u32
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
        EditType::CreateGrid,
        CreateEdit {
            object_id,
            type_id: evt.type_id,
            parent_id: evt.parent_id
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
    objmap: Res<ObjectIdMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cr) = edit.get(evt.entity) else { return Ok(()); };

    // get the parent
    let parent = *objmap.0.get(&cr.parent_id).unwrap();

    // apply the change

    use std::f32::consts::PI;

    let gdef = &gamebox.grid[&cr.type_id];

    let t = match gdef {
        GridDefinition::Hex(h) => {
            Transform {
                translation: Vec3::new(h.x, h.y, 1.0),
                rotation: Quat::from_rotation_z(h.a * PI / 180.0),
                scale: Vec3::new(h.s, h.s, 1.0)
            }
        },
        GridDefinition::Rect(r) => {
            Transform {
                translation: Vec3::new(r.x, r.y, 0.0),
                rotation: Quat::from_rotation_z(r.a * PI / 180.0),
                scale: Vec3::new(r.s, r.s, 1.0)
            }
        }
    };

    spawn_grid(
//        cr.object_id,
        gdef,
        parent,
        t,
        &mut meshes,
        &mut materials,
        &mut commands
    );

    Ok(())
}
