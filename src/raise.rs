use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Query, Resource, With}
    },
    picking::{
        events::{Pointer, Press, Release},
        pointer::PointerButton
    },
    prelude::{trace, Transform}
};
use tracing::instrument;

use crate::{Piece, Surface};

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
    surface: &mut ResMut<Surface>
)
{
    surface.max_z = surface.max_z.next_up();
    set_piece_depth(transform, surface.max_z);
}

pub fn raise_piece_to_top_anchored(
    transform: &mut Transform,
    surface: &mut ResMut<Surface>,
    anchor: &mut ResMut<RaiseAnchor>
)
{
    anchor.z = transform.translation.z;
    raise_piece_to_top(transform, surface);
}

pub fn lower_piece_to_anchor(
    transform: &mut Transform,
    anchor: &Res<RaiseAnchor>
)
{
    transform.translation.z = anchor.z;
}

#[instrument(skip_all)]
pub fn on_piece_pressed(
    press: On<Pointer<Press>>,
    mut query: Query<&mut Transform, With<Piece>>,
    mut anchor: ResMut<RaiseAnchor>,
    mut surface: ResMut<Surface>
) -> Result
{
    trace!("");

    if press.button != PointerButton::Secondary {
        return Ok(());
    }

    let entity = press.event().event_target();

    let mut transform = query.get_mut(entity)?;

    raise_piece_to_top_anchored(&mut transform, &mut surface, &mut anchor);

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
