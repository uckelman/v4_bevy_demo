use bevy::{
    ecs::{
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Commands, Query, With}
    },
    input::keyboard::KeyCode,
    prelude::{Entity, Resource, Sprite, trace}
};
use tracing::instrument;

use crate::{
    assets::ImageSource,
    config::KeyConfig,
    piece::{Faces, FaceUp},
    select::Selected,
};

#[derive(Resource)]
pub struct FlipForwardKey(pub KeyCode);

#[derive(Resource)]
pub struct FlipBackKey(pub KeyCode);

impl KeyConfig for FlipForwardKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for FlipBackKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

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
    flip: On<FlipEvent>,
    query: Query<(&Faces, &mut FaceUp, &mut Sprite)>
) -> Result
{
    trace!("");
    let entity = flip.event().event_target();
    do_flip(entity, query, flip.delta)
}

#[instrument(skip_all)]
pub fn handle_flip_forward(
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
)
{
    trace!("");
    query.iter()
        .for_each(|entity| commands.trigger(FlipEvent { entity, delta: 1 }));
}

#[instrument(skip_all)]
pub fn handle_flip_back(
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
)
{
    trace!("");
    query.iter()
        .for_each(|entity| commands.trigger(FlipEvent { entity, delta: -1 }));
}
