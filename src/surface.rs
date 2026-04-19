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
    prelude::{debug, DespawnOnExit, GlobalTransform, Transform, Visibility}
};
use tracing::instrument;

use crate::{
    GameState,
    maxz::MaxZ,
    object::ObjectId,
    piece::{Piece, StackingGroup},
    stack::StackBelowQueryExt
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
        Visibility::Inherited,
        DespawnOnExit(GameState::Game)
    )).id();

    commands.entity(window)
        .insert(ForSurface(id))
        .observe(on_piece_drop);
}

#[instrument(skip_all)]
pub fn on_piece_drop(
    mut drop: On<Pointer<DragDrop>>,
    mut base_query: Query<(&ChildOf, &GlobalTransform, &mut Transform), (With<Piece>, Without<Surface>)>,
    surface_query: Query<&ForSurface>,
    dst_query: Query<&GlobalTransform>,
    a_query: Query<(Option<&ChildOf>, &StackingGroup)>,
    mut commands: Commands
) -> Result
{
    debug!("");

    let src = drop.event().dropped;
    let base = a_query.bottom(src);

    let Ok((base_parent, base_gt, mut base_t)) = base_query.get_mut(base) else {
        return Ok(());
    };

    drop.propagate(false);

    let dst = surface_query.get(drop.event().event_target())?.0;

    if base_parent.0 != dst {
        let dst_gt = dst_query.get(dst)?;

        // reparent stack base to surface
        *base_t = base_gt.reparented_to(dst_gt);
        commands.entity(dst).add_child(base);
        eprintln!("surface {}", dst);
    }

    Ok(())
}
