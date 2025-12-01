use bevy::{
    ecs::{
        change_detection::ResMut,
        error::Result,
        observer::On,
        prelude::{Query, With, Without}
    },
    math::Vec3,
    picking::{
        events::{Drag, DragEnd, DragStart, Pointer},
        pointer::PointerButton
    },
    prelude::{Camera, Commands, Component, Entity, Projection, trace, Transform}
};
use itertools::Itertools;
use std::cmp::Ordering;
use tracing::instrument;

use crate::Surface;
use crate::raise::raise_piece;
use crate::select::Selected;
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
    query: Query<(Entity, &mut Transform), (With<Draggable>, With<Selected>)>,
    mut surface: ResMut<Surface>,
    mut commands: Commands
)
{
    trace!("");

    if drag.button == PointerButton::Primary {
        // find the min, max depth of the selection
        let Some((min_z, max_z)) = query.iter()
            .minmax_by(|(_, ta), (_, tb)|
                ta.translation.z.partial_cmp(&tb.translation.z)
                    .unwrap_or(Ordering::Less)
            )
            .into_option()
            .map(|((_, a), (_, b))| (a.translation.z, b.translation.z))
        else {
            return;
        };

        // raise the entire selection to be above the max
        let dz = surface.max_z.next_up() - min_z;
        surface.max_z = max_z + dz;

        for (entity, mut transform) in query {
            raise_piece(&mut transform, dz);

            commands.entity(entity)
                .insert(DragAnchor { pos: transform.translation });
        }
    }
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

    let (camera_transform, camera_projection) = tp_query.single()?;
    let camera_projection = camera_projection.as_ortho()?;

    let mut drag_dist = drag.distance.extend(0.0);
    drag_dist.y = -drag_dist.y;

    // apply current scale to the drag
    drag_dist *= camera_projection.scale;

    // apply current rotation to the drag
    drag_dist = camera_transform.rotation * drag_dist;

    d_query.iter_mut().for_each(|(mut transform, anchor)| {
        transform.translation = anchor.pos + drag_dist;
    });

    // prevent the event from bubbling up to the world
    drag.propagate(false);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag_end(
    drag: On<Pointer<DragEnd>>,
    query: Query<Entity, With<DragAnchor>>,
    mut commands: Commands
)
{
    trace!("");

    if drag.button == PointerButton::Primary {
        query.iter()
            .for_each(|entity| { commands.entity(entity).remove::<DragAnchor>();});
    }
}
