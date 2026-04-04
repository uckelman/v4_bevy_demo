use bevy::{
    camera::visibility::Visibility,
    ecs::{
        component::Component,
        prelude::Commands
    },
    prelude::Transform
};

use crate::object::ObjectId;

pub mod create;

#[derive(Component, Default)]
pub struct Surface;

#[derive(Component, Debug)]
pub struct MaxZ(f32);

pub fn spawn_surface(
    oid: u32,
    commands: &mut Commands
)
{
    commands.spawn((
        Surface,
        ObjectId(oid),
        Transform::IDENTITY,
//        Visibility::Hidden,
        MaxZ(0.0))
    );
} 
