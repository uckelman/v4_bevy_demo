use bevy::{
    ecs::{
        component::Component,
        event::EntityEvent,
        error::Result,
        name::Name,
        observer::On,
        prelude::{ChildOf, Commands, Entity, Query, With, Without}
    },
    picking::{
        Pickable,
        backend::HitData,
        events::{DragDrop, Pointer},
        pointer::Location
    },
    prelude::{Camera, debug, DespawnOnExit, GlobalTransform, Transform, Visibility}
};
use tracing::instrument;

use crate::{
    GameState,
    drag::handle_drop,
    maxz::MaxZ,
    object::ObjectId
};

pub mod create;

#[derive(Component, Default)]
pub struct Surface;

#[derive(Component)]
pub struct ForSurface(pub Entity);

pub fn spawn_surface(
    oid: u32,
    window: Entity,
    commands: &mut Commands
)
{
    let id = commands.spawn((
        Surface,
        ObjectId(oid),
//        Name::from(m.name.as_ref()),
        Name::from("surface"),
        Transform::IDENTITY,
//        Transform::from_xyz(0.0, 0.0, f32::NEG_INFINITY),
        MaxZ(0.0),
        Pickable::IGNORE,
        Visibility::Inherited,
        DespawnOnExit(GameState::Game)
    ))
    .observe(handle_drop)
    .id();

    commands.entity(window)
        .insert(ForSurface(id))
        .observe(forward_dragdrop_to_surface);
}

fn forward_dragdrop_to_surface(
    mut drop: On<Pointer<DragDrop>>,
    surface_query: Query<&ForSurface>,
    c_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands
) -> Result
{
    drop.propagate(false);

    let dst = drop.event().event_target();
    let src = drop.event().dropped;

    if src == dst {
        return Ok(());
    }

    let dst = surface_query.get(dst)?.0;

    let drop_pos = drop.hit.position.unwrap().truncate();

    let (camera, c_gt) = c_query.single()?;
    let drop_pos = camera.viewport_to_world_2d(c_gt, drop_pos)?;

    let position = Some(drop_pos.extend(0.0));

    commands.trigger(Pointer::new(
        drop.pointer_id,
        drop.pointer_location.clone(),
        DragDrop {
            hit: HitData {
                position,
                ..drop.event.hit
            },
            ..drop.event
        },
        dst
    ));

    Ok(())
}
