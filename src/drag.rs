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
        events::{Drag, DragStart, Pointer},
        pointer::PointerButton
    },
    prelude::{Camera, Projection, trace, Transform}
};
use tracing::instrument;

use crate::{Piece, Surface};
use crate::raise::raise_piece_to_top;
use crate::util::AsOrthographicProjection;

#[derive(Default, Resource)]
pub struct DragAnchor {
    pos: Vec3
}

#[instrument(skip_all)]
pub fn on_piece_drag_start(
    drag: On<Pointer<DragStart>>,
    mut query: Query<&mut Transform, With<Piece>>,
    mut anchor: ResMut<DragAnchor>,
    mut surface: ResMut<Surface>
) -> Result
{
    trace!("");

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    if let Some(pos) = drag.hit.position {
        let mut transform = query.get_mut(drag.event().event_target())?;
        raise_piece_to_top(&mut transform, &mut surface);

// FIXME: anchor doesn't respect offset from piece center
        anchor.pos = pos.with_z(surface.max_z);
//        trace!("{}", anchor.pos);

    }

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag(
    mut drag: On<Pointer<Drag>>,
    mut p_query: Query<&mut Transform, (With<Piece>, Without<Camera>)>,
    tp_query: Query<(&Transform, &Projection), With<Camera>>,
    anchor: Res<DragAnchor>
) -> Result
{
    trace!("");

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    let mut transform = p_query.get_mut(drag.event().event_target())?;
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
