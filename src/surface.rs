use bevy::{
    ecs::{
        component::Component,
        event::EntityEvent,
        error::Result,
        observer::On,
        prelude::{ChildOf, Commands, Entity, Query, With, Without}
    },
    picking::{
        Pickable,
        events::{DragDrop, Pointer}
    },
    prelude::{debug, DespawnOnExit, GlobalTransform, Transform}
};
use tracing::instrument;

use crate::{
    GameState,
    maxz::MaxZ,
    object::ObjectId,
    piece::Piece
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
//    Name::from(m.name.as_ref()),
        Transform::IDENTITY,
        MaxZ(0.0),
        Pickable::IGNORE,
        DespawnOnExit(GameState::Game)
    )).id();

    commands.entity(window)
        .insert(ForSurface(id))
        .observe(on_piece_drop);
}

#[instrument(skip_all)]
pub fn on_piece_drop(
    mut drop: On<Pointer<DragDrop>>,
    mut src_query: Query<(&ChildOf, &GlobalTransform, &mut Transform), (With<Piece>, Without<Surface>)>,
    surface_query: Query<&ForSurface>,
    dst_query: Query<&GlobalTransform>,
    mut commands: Commands
) -> Result
{
    debug!("");

    drop.propagate(false);

    let src = drop.event().dropped;

    let Ok((parent, src_t, mut t)) = src_query.get_mut(src) else {
        return Ok(());
    };

    let dst = surface_query.get(drop.event().event_target())?.0;

    if parent.0 != dst {
        let dst_t = dst_query.get(dst)?;

        // reparent to surface
        *t = src_t.reparented_to(dst_t);
        commands.entity(src).insert(ChildOf(dst));
        eprintln!("surface {}", dst);
    }

    Ok(())
}
