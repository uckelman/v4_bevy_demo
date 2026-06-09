use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Changed, Commands, Query}
    },
    prelude::{Entity, Sprite, trace}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    assets::ImageSource,
    edittype::EditType,
    log::{EditIndex, Edits, handle_do},
    object::{ObjectId, ObjectIdMap},
    piece::{Faces, FaceUp}
};

#[derive(Clone, EntityEvent)]
pub struct DoFlipEvent {
    pub entity: Entity,
    pub delta: i32
}

#[derive(EntityEvent)]
pub struct UndoFlipEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoFlipEvent {
    pub entity: Entity
}

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

#[derive(Component, Debug, Deserialize, Serialize)]
#[serde(rename = "flip", tag = "type")]
pub struct FlipEdit {
    pub object_id: u32,
    pub delta: i32
}

#[instrument(skip_all)]
pub fn on_flip(
    evt: On<DoFlipEvent>,
    piece_query: Query<&ObjectId>,
    edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let object_id = piece_query.get(entity)?;

    handle_do(
        edit_query,
        EditType::Flip,
        FlipEdit { object_id: object_id.0, delta: evt.delta },
        commands
    )
}

fn apply_flip<const DO: bool>(
    event_target: Entity,
    edit: Query<&FlipEdit>,
    objmap: Res<ObjectIdMap>,
    mut query: Query<(&mut FaceUp, &Faces)>
) -> Result
{
    // get the edit
    let Ok(flip) = edit.get(event_target) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&flip.object_id).unwrap();
    // get the components of the entity being edited
    let (mut up, faces) = query.get_mut(entity)?;
    // apply the change to the entity
    let delta = if DO { flip.delta } else { -flip.delta };
    let len = faces.0.len() as i32;
    up.0 = (((up.0 as i32 + delta) % len + len) % len) as usize;
    Ok(())
}

#[instrument(skip_all)]
pub fn on_flip_undo(
    evt: On<UndoFlipEvent>,
    edit: Query<&FlipEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<(&mut FaceUp, &Faces)>
) -> Result
{
    apply_flip::<false>(evt.entity, edit, objmap, query)
}

#[instrument(skip_all)]
pub fn on_flip_redo(
    evt: On<RedoFlipEvent>,
    edit: Query<&FlipEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<(&mut FaceUp, &Faces)>
) -> Result
{
    apply_flip::<true>(evt.entity, edit, objmap, query)
}

#[instrument(skip_all)]
pub fn on_face_change(
    mut query: Query<(&mut Sprite, &Faces, &FaceUp), Changed<FaceUp>>
)
{
    for (mut sprite, faces, up) in query.iter_mut() {
        set_face(&mut sprite, faces, up);
    }
}
