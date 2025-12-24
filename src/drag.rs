use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        error::Result,
        observer::On,
        prelude::{Query, With, Without}
    },
    math::Vec3,
    picking::{
        Pickable,
        events::{Drag, DragEnd, DragStart, Pointer},
        pointer::PointerButton
    },
    prelude::{Camera, Commands, Component, Entity, Projection, State, trace, Transform}
};
use itertools::Itertools;
use std::cmp::Ordering;
use tracing::instrument;

use crate::{
    Surface,
    context_menu::ContextMenuState,
    raise::raise_piece,
    select::Selected,
    util::AsOrthographicProjection
};

#[derive(Clone, Component, Copy, Debug,  Default)]
pub struct Draggable;

#[derive(Component, Default)]
pub struct DragAnchor {
    pos: Vec3
}

#[instrument(skip_all)]
pub fn on_piece_drag_start(
    drag: On<Pointer<DragStart>>,
    query: Query<(Entity, &mut Transform), (With<Draggable>, With<Selected>)>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut surface: ResMut<Surface>,
    mut commands: Commands
)
{
    trace!("");

    if drag.button != PointerButton::Primary {
        return;
    }

    if *context_menu_state == ContextMenuState::Open {
        return;
    }

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

        // set the drag anchor, prevent picking from hitting piece
        commands.entity(entity)
            .insert(DragAnchor { pos: transform.translation })
            .insert(Pickable::IGNORE);
    }
}

#[instrument(skip_all)]
pub fn on_piece_drag(
    mut drag: On<Pointer<Drag>>,
    mut d_query: Query<(&mut Transform, &DragAnchor), Without<Camera>>,
    tp_query: Query<(&Transform, &Projection), With<Camera>>,
    context_menu_state: Res<State<ContextMenuState>>
) -> Result
{
    trace!("");

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    if *context_menu_state == ContextMenuState::Open {
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
//    drag.propagate(false);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag_end(
    drag: On<Pointer<DragEnd>>,
    query: Query<Entity, With<DragAnchor>>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut commands: Commands
)
{
    trace!("");

    if *context_menu_state == ContextMenuState::Open {
        return;
    }

    if drag.button == PointerButton::Primary {
        // remove the drag anchor, make piece pickable again
        query.iter()
            .for_each(|entity| {
                commands.entity(entity)
                    .remove::<DragAnchor>()
                    .insert(Pickable::default());
            });
    }
}
