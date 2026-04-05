use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{ChildOf, Query, Resource, With}
    },
    picking::{
        events::{Pointer, Press, Release},
        pointer::PointerButton
    },
    prelude::{trace, Transform}
};
use tracing::instrument;

use crate::{
    maxz::MaxZ,
    piece::Piece
};

#[derive(Default, Resource)]
pub struct RaiseAnchor {
    z: f32
}

pub fn set_piece_depth(
    transform: &mut Transform,
    z: f32
)
{
    transform.translation.z = z;
}

pub fn raise_piece(
    transform: &mut Transform,
    dz: f32
)
{
    set_piece_depth(transform, transform.translation.z + dz)
}

pub fn raise_piece_to_top(
    transform: &mut Transform,
    max_z: &mut MaxZ,
)
{
    max_z.0 = max_z.0.next_up();
    set_piece_depth(transform, max_z.0);
}

pub fn raise_piece_to_top_anchored(
    transform: &mut Transform,
    max_z: &mut MaxZ,
    anchor: &mut ResMut<RaiseAnchor>
)
{
    anchor.z = transform.translation.z;
    raise_piece_to_top(transform, max_z);
}

pub fn lower_piece_to_anchor(
    transform: &mut Transform,
    anchor: &Res<RaiseAnchor>
)
{
    transform.translation.z = anchor.z;
}

// TODO: add MaxZ to all things which can have children

#[instrument(skip_all)]
pub fn on_piece_pressed(
    press: On<Pointer<Press>>,
    mut query: Query<&mut Transform, With<Piece>>,
    root_query: Query<&ChildOf>,
    mut maxz_query: Query<&mut MaxZ>,
    mut anchor: ResMut<RaiseAnchor>
) -> Result
{
    trace!("");

    if press.button != PointerButton::Secondary {
        return Ok(());
    }

    let entity = press.event().event_target();
    let mut transform = query.get_mut(entity)?;

    let root = root_query.root_ancestor(entity);
    let mut max_z = maxz_query.get_mut(root)?;

// TODO: maybe we don't want this?
    raise_piece_to_top_anchored(&mut transform, &mut max_z, &mut anchor);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_released(
    press: On<Pointer<Release>>,
    mut query: Query<&mut Transform, With<Piece>>,
    anchor: Res<RaiseAnchor>
) -> Result
{
    trace!("");

    if press.button != PointerButton::Secondary {
        return Ok(());
    }

    let entity = press.event().event_target();

    let mut transform = query.get_mut(entity)?;
    lower_piece_to_anchor(&mut transform, &anchor);

    Ok(())
}
