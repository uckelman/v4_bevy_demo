use bevy::{
    ecs::{
        component::Component,
        prelude::Commands
    },
    picking::{
        Pickable,
    },
    prelude::{DespawnOnExit, Transform}
};

use crate::{
    GameState,
    maxz::MaxZ,
    object::ObjectId
};

pub mod create;

#[derive(Component, Default)]
pub struct Surface;

pub fn spawn_surface(
    oid: u32,
    commands: &mut Commands
)
{
    commands.spawn((
        Surface,
        ObjectId(oid),
//        Name::from(m.name.as_ref()),
        Transform::IDENTITY,
        MaxZ(0.0),
        Pickable::default(),
        DespawnOnExit(GameState::Game)
    ));
}
