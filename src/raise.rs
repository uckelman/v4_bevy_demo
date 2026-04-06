use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{ChildOf, Query, Resource, With, Without}
    },
    picking::{
        events::{Pointer, Press, Release},
        pointer::PointerButton
    },
    prelude::{GlobalTransform, trace, Transform}
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

pub fn raise_piece(
    transform: &mut Transform,
    z: f32,
)
{
    transform.translation.z = z;
}

pub fn raise_piece_anchored(
    transform: &mut Transform,
    z: f32,
    anchor: &mut ResMut<RaiseAnchor>
)
{
    anchor.z = transform.translation.z;
    raise_piece(transform, z);
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
    mut query: Query<(&ChildOf, &mut Transform), With<Piece>>,
    root_query: Query<&ChildOf>,
    mut maxz_query: Query<&mut MaxZ>,
    global_transform_query: Query<&GlobalTransform>,
    mut anchor: ResMut<RaiseAnchor>
) -> Result
{
    trace!("");

    if press.button != PointerButton::Secondary {
        return Ok(());
    }

    let entity = press.event().event_target();
    let (parent, mut transform) = query.get_mut(entity)?;

    // update max z
    let root = root_query.root_ancestor(entity);
    let mut max_z = maxz_query.get_mut(root)?;

    max_z.0 += 1.0;

    let parent_t = global_transform_query.get(parent.0)?;
    let local_z = max_z.0 - parent_t.translation().z;

// TODO: maybe we don't want this?
    raise_piece_anchored(&mut transform, local_z, &mut anchor);

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
