use bevy::{
    camera::visibility::Visibility,
    ecs::{
        change_detection::Res,
        component::Component,
        name::Name,
        prelude::{ChildOf, Commands, Entity}
    },
    math::{Quat, Vec3},
    prelude::{debug, DespawnOnExit, Sprite, Transform}
};

use crate::{
    GameState,
    assets::{ImageSource, SpriteHandles},
    gamebox::{Anchor, MapType},
    object::ObjectId
};

pub mod create;

#[derive(Component, Default)]
pub struct Map;

pub fn spawn_map(
    oid: u32,
    tid: u32,
    m: &MapType,
    location: Vec3,
    angle: f32,
    scale: f32,
    anchor: Anchor,
    parent: Entity,
    sprite_handles: &Res<SpriteHandles>,
    commands: &mut Commands
)
{
    use std::f32::consts::PI;

    let t = Transform {
        translation: location,
        rotation: Quat::from_rotation_z(angle * PI / 180.0),
        scale: Vec3::new(scale, scale, 1.0)
    };

    let anchor: bevy::sprite::Anchor = anchor.into();

    let sh = sprite_handles.0.get(&m.image).unwrap().clone();
    let sprite = match sh {
        ImageSource::Single(handle) => Sprite::from_image(handle.clone()),
        ImageSource::Crop { handle, atlas } => Sprite::from_atlas_image(
            handle.clone(),
            atlas.clone()
        )
    };

    let id = commands.spawn((
        Map,
        ObjectId(oid),
        Name::from(m.name.as_ref()),
        sprite,
        ChildOf(parent),
        t,
        anchor,
//        Visibility::Visible,
        DespawnOnExit(GameState::Game)
    )).id();

    debug!("map {id}");
}
