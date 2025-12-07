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
    Faces, FaceUp,
    config::KeyConfig,
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
pub struct FlipForwardEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct FlipBackEvent {
    pub entity: Entity
}

#[instrument(skip_all)]
pub fn on_flip_forward(
    flip: On<FlipForwardEvent>,
    mut query: Query<(&Faces, &mut FaceUp, &mut Sprite), With<Selected>>
) -> Result
{
    trace!("");

    let entity = flip.event().event_target();

    let (faces, mut up, mut sprite) = query.get_mut(entity)?;
    up.0 = (up.0 + 1) % faces.0.len();
    sprite.image = faces.0[up.0].clone();

    Ok(())
}

#[instrument(skip_all)]
pub fn on_flip_back(
    flip: On<FlipBackEvent>,
    mut query: Query<(&Faces, &mut FaceUp, &mut Sprite), With<Selected>>
) -> Result
{
    trace!("");

    let entity = flip.event().event_target();

    let (faces, mut up, mut sprite) = query.get_mut(entity)?;
    up.0 = (up.0 + faces.0.len() - 1) % faces.0.len();
    sprite.image = faces.0[up.0].clone();

    Ok(())
}

#[instrument(skip_all)]
pub fn handle_flip_forward(
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
)
{
    trace!("");
    query.iter()
        .for_each(|entity| commands.trigger(FlipForwardEvent { entity }));
}

#[instrument(skip_all)]
pub fn handle_flip_back(
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
)
{
    trace!("");
    query.iter()
        .for_each(|entity| commands.trigger(FlipBackEvent { entity }));
}
