use bevy::{
    ecs::{
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
    piece::{Faces, FaceUp},
    select::Selected,
};

#[derive(EntityEvent)]
pub struct FlipEvent {
    pub entity: Entity,
    pub delta: i32
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

fn do_flip(
    entity: Entity,
    mut query: Query<(&Faces, &mut FaceUp, &mut Sprite)>,
    delta: i32
) -> Result
{
    let (faces, mut up, mut sprite) = query.get_mut(entity)?;

    let len = faces.0.len() as i32;
    up.0 = (((up.0 as i32 + delta) % len + len) % len) as usize;

    set_face(&mut sprite, faces, &up);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_flip(
    evt: On<FlipEvent>,
    query: Query<(&Faces, &mut FaceUp, &mut Sprite)>
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    do_flip(entity, query, evt.delta)
}
