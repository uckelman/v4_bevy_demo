use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        event::EntityEvent,
        error::Result,
        name::Name,
        observer::On,
        prelude::{ChildOf, Commands, Entity, Query, With, Without}
    },
    math::{Quat, Vec3},
    picking::{
        Pickable,
        events::{DragDrop, Pointer}
    },
    prelude::{debug, DespawnOnExit, Sprite, Transform}
};
use tracing::instrument;

use crate::{
    GameState,
    assets::{ImageSource, SpriteHandles},
    gamebox::{Anchor, MapType},
    object::ObjectId,
    piece::Piece
};

pub mod create;

#[derive(Component, Default)]
pub struct Map;

pub fn spawn_map(
    oid: u32,
    tid: u32,
    m: &MapType,
    parent: Entity,
    location: Vec3,
    angle: f32,
    scale: f32,
    anchor: Anchor,
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

    let id = commands
        .spawn((
            Map,
            ObjectId(oid),
            Name::from(m.name.as_ref()),
            sprite,
            ChildOf(parent),
            t,
            anchor,
            Pickable::default(),
            DespawnOnExit(GameState::Game)
        ))
        .observe(on_piece_drop)
        .id();

    debug!("map {id}");
}

#[instrument(skip_all)]
pub fn on_piece_drop(
    mut drop: On<Pointer<DragDrop>>,
    src_query: Query<&ChildOf, (With<Piece>, Without<Map>)>,
    mut commands: Commands
) -> Result
{
    debug!("");

    drop.propagate(false);

    let dst = drop.event().event_target();
    let src = drop.event().dropped;

    let parent = src_query.get(src)?;

    if parent.0 != dst {
        // reparent to map
        commands.entity(src).insert(ChildOf(dst));
        eprintln!("map {dst}");
    }

    Ok(())
}
