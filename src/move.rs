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
use tracing::instrument;

use crate::{
    log::{EditIndex, EditOf, EditType, Edits, handle_do, RedoMoveEvent, UndoMoveEvent},
    object::{ObjectId, ObjectIdMap}
};

fn do_move(t: &mut Transform, delta: Vec3)
{
    t.translation += delta;
}

#[derive(Clone, Copy, EntityEvent)]
pub struct MoveEvent {
    pub entity: Entity,
    pub delta: Vec3
}

#[derive(Component)]
pub struct MoveEdit {
    pub object_id: u32,
    pub delta: Vec3
}

#[instrument(skip_all)]
pub fn on_move(
    evt: On<MoveEvent>,
    mut piece_query: Query<(&ObjectId, &mut Transform)>,
    mut edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    mut commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let (object_id, mut t) = piece_query.get_mut(entity)?;

    let (edits_entity, mut edits, mut edit_index) = edit_query.single_mut()?;
    handle_do(&mut edits, &mut edit_index, &mut commands);

    commands.spawn((
        EditOf(edits_entity),
        EditType::Move,
        MoveEdit { object_id: object_id.0, delta: evt.delta }
    ));

// TODO: need separate listeners for post-drag, pure move
//    do_move(&mut t, evt.delta);
    Ok(())
}

fn apply_move(
    event_target: Entity,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    mut query: Query<&mut Transform>,
    dir: f32
) -> Result
{
    // get the edit
    let Ok(mov) = edit.get(event_target) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&mov.object_id).unwrap();
    // get the components of the entity being edited
    let mut t = query.get_mut(entity)?;
    // apply the change to the entity
    do_move(&mut t, dir * mov.delta);
    Ok(())
}

#[instrument(skip_all)]
pub fn on_move_undo(
    evt: On<UndoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<&mut Transform>
) -> Result
{
    apply_move(evt.entity, edit, objmap, query, -1.0)
}

#[instrument(skip_all)]
pub fn on_move_redo(
    evt: On<RedoMoveEvent>,
    edit: Query<&MoveEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<&mut Transform>
) -> Result
{
    apply_move(evt.entity, edit, objmap, query, 1.0)
}
