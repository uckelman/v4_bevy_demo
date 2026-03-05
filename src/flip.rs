use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Commands, Query, With}
    },
    prelude::{Entity, Resource, Sprite, trace}
};
use tracing::instrument;

use crate::{
    assets::ImageSource,
    log::{EditIndex, EditOf, EditType, Edits, handle_do, RedoFlipEvent, UndoFlipEvent},
    object::{ObjectId, ObjectIdMap},
    piece::{Faces, FaceUp},
    select::Selected,
};

fn set_face(
    sprite: &mut Sprite,
    faces: &Faces,
    up: &FaceUp
)
{
    match &faces.0[up.0] {
        ImageSource::Single(handle) => {
            sprite.image = handle.clone();
            sprite.texture_atlas = None;
        },
        ImageSource::Crop { handle, atlas } => {
            sprite.image = handle.clone();
            sprite.texture_atlas = Some(atlas.clone());
        }
    }
}

fn do_flip(
    faces: &Faces,
    up: &mut FaceUp,
    sprite: &mut Sprite,
    delta: i32
)
{
    let len = faces.0.len() as i32;
    up.0 = (((up.0 as i32 + delta) % len + len) % len) as usize;

    set_face(sprite, faces, up);
}

#[derive(Clone, Copy, EntityEvent)]
pub struct FlipEvent {
    pub entity: Entity,
    pub delta: i32
}

#[derive(Component)]
pub struct FlipEdit {
    pub object_id: u32,
    pub delta: i32
}

#[instrument(skip_all)]
pub fn on_flip(
    evt: On<FlipEvent>,
    mut piece_query: Query<(&ObjectId, &Faces, &mut FaceUp, &mut Sprite)>,
    mut edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    mut commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let (object_id, faces, mut up, mut sprite) = piece_query.get_mut(entity)?;

    let (edits_entity, mut edits, mut edit_index) = edit_query.single_mut()?;
    handle_do(&mut edits, &mut edit_index, &mut commands);

    commands.spawn((
        EditOf(edits_entity),
        EditType::Flip,
        FlipEdit { object_id: object_id.0, delta: evt.delta }
    ));

    do_flip(faces, &mut up, &mut sprite, evt.delta);
    Ok(())
}

fn apply_flip(
    event_target: Entity,
    edit: Query<&FlipEdit>,
    objmap: Res<ObjectIdMap>,
    mut query: Query<(&Faces, &mut FaceUp, &mut Sprite)>,
    dir: i32
) -> Result
{
    // get the edit
    let Ok(flip) = edit.get(event_target) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&flip.object_id).unwrap();
    // get the components of the entity being edited
    let (faces, mut up, mut sprite) = query.get_mut(entity)?;
    // apply the change to the entity
    do_flip(faces, &mut up, &mut sprite, dir * flip.delta);
    Ok(())
}

#[instrument(skip_all)]
pub fn on_flip_undo(
    evt: On<UndoFlipEvent>,
    edit: Query<&FlipEdit>,
    objmap: Res<ObjectIdMap>,
    mut query: Query<(&Faces, &mut FaceUp, &mut Sprite)>
) -> Result
{
    apply_flip(evt.entity, edit, objmap, query, -1)
}

#[instrument(skip_all)]
pub fn on_flip_redo(
    evt: On<RedoFlipEvent>,
    edit: Query<&FlipEdit>,
    objmap: Res<ObjectIdMap>,
    mut query: Query<(&Faces, &mut FaceUp, &mut Sprite)>
) -> Result
{
    apply_flip(evt.entity, edit, objmap, query, 1)
}
