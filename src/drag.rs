use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Query, Resource, With, Without}
    },
    math::Vec3,
    picking::{
        events::{Drag, DragEnd, DragStart, Pointer},
        pointer::PointerButton
    },
    prelude::{Camera, Commands, Component, Entity, Projection, trace, Transform}
};
use tracing::instrument;

use crate::{Piece, Surface};
use crate::raise::raise_piece_to_top;
use crate::util::AsOrthographicProjection;

#[derive(Component, Default)]
pub struct Draggable;

#[derive(Component, Default)]
pub struct DragAnchor {
    pos: Vec3
}

#[instrument(skip_all)]
pub fn on_piece_drag_start(
    drag: On<Pointer<DragStart>>,
    mut query: Query<&mut Transform, With<Draggable>>,
    mut surface: ResMut<Surface>,
    mut commands: Commands
) -> Result
{
    trace!("");

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    let entity = drag.event().event_target();

    let mut transform = query.get_mut(entity)?;
    raise_piece_to_top(&mut transform, &mut surface);

    commands.entity(entity)
        .insert(DragAnchor { pos: transform.translation });

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag(
    mut drag: On<Pointer<Drag>>,
    mut d_query: Query<(&mut Transform, &DragAnchor), Without<Camera>>,
    tp_query: Query<(&Transform, &Projection), With<Camera>>
) -> Result
{
    trace!("");

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    let entity = drag.event().event_target();

    let (mut transform, anchor) = d_query.get_mut(entity)?;
    let (camera_transform, camera_projection) = tp_query.single()?;

    let camera_projection = camera_projection.as_ortho()?;

//    trace!("d {}, t {}", drag.delta, drag.distance);

    let mut drag_dist = drag.distance.extend(0.0);
    drag_dist.y = -drag_dist.y;

    // apply current scale to the drag
    drag_dist *= camera_projection.scale;

    // apply current rotation to the drag
    drag_dist = camera_transform.rotation * drag_dist;

    transform.translation = anchor.pos + drag_dist;

    // prevent the event from bubbling up to the world
    drag.propagate(false);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag_end(
    _drag: On<Pointer<DragEnd>>,
    mut query: Query<Entity, With<DragAnchor>>,
    mut commands: Commands
) -> Result
{
    trace!("");

    query.iter()
        .for_each(|entity| { commands.entity(entity).remove::<DragAnchor>(); });

    Ok(())
}
