use bevy::{
    ecs::{
        component::Component,
        prelude::Commands
    },
    prelude::Transform
};

use crate::{
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
        Transform::IDENTITY,
        MaxZ(0.0)
    ));
}
