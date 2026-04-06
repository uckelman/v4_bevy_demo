use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        observer::On,
        prelude::{ChildOf, Commands, Entity, Query, With, Without}
    },
    math::Vec3,
    picking::{
        Pickable,
        events::{Drag, DragEnd, DragStart, Pointer},
        pointer::PointerButton
    },
    prelude::{Camera, GlobalTransform, Projection, State, trace, Transform}
};
use itertools::Itertools;
use tracing::instrument;

use crate::{
    context_menu::ContextMenuState,
    log::{OpenGroupEvent, CloseGroupEvent},
    maxz::MaxZ,
    piece::r#move::DoMoveEvent,
    raise::raise_piece,
    select::Selected,
    util::AsOrthographicProjection
};

#[derive(Clone, Component, Copy, Debug,  Default)]
pub struct Draggable;

#[derive(Component)]
pub struct DragAnchor {
    parent: Entity,
    pos: Vec3
}

// TODO: surface needs a MaxZ covering all descendants, otherwise we
// can't raise a drag properly

#[instrument(skip_all)]
pub fn on_piece_drag_start(
    drag: On<Pointer<DragStart>>,
    query: Query<(Entity, &ChildOf, &GlobalTransform, &mut Transform), (With<Draggable>, With<Selected>)>,
    parent_query: Query<&ChildOf>,
    mut maxz_query: Query<&mut MaxZ>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut commands: Commands
) -> Result
{
    trace!("");

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    if *context_menu_state == ContextMenuState::Open {
        return Ok(());
    }

    // find the min, max depth of the selection
    let Some((sel_min_z, sel_max_z)) = query.iter()
        .map(|(_, _, t, _)| t.translation().z)
        .minmax()
        .into_option()
    else {
        return Ok(());
    };

    // selection must be on one surface; get it from the first selected item
    let (entity, ..) = query.iter().next().expect("query is nonempty");
    let root = parent_query.root_ancestor(entity);

    // raise entire selection by amount the lowest member is below max z
    let mut max_z = maxz_query.get_mut(root)?;

    let dz = max_z.0 - sel_min_z + 1.0;

    max_z.0 = sel_max_z + dz;

    for (entity, parent, _, mut transform) in query {
        let z = transform.translation.z + dz;
        raise_piece(&mut transform, z);

        // set the drag anchor, prevent picking from hitting piece
        commands.entity(entity)
            .insert((
                DragAnchor {
                    parent: parent.0,
                    pos: transform.translation
                },
                Pickable::IGNORE
            ));
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag(
    drag: On<Pointer<Drag>>,
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
    query: Query<(Entity, &ChildOf, &Transform, &DragAnchor)>,
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
        let mut eptai = query.iter();
        match eptai.len() {
            0 => {},
            1 => {
                let (e, p, t, a) = eptai.next().expect("len is 1");
                move_and_deselect(e, p.0, t, a, &mut commands);
            },
            _ => {
                commands.trigger(OpenGroupEvent);
                eptai.for_each(|(e, p, t, a)| move_and_deselect(e, p.0, t, a, &mut commands));
                commands.trigger(CloseGroupEvent);
            }
        }
    }
}

fn move_and_deselect(
    entity: Entity,
    p: Entity,
    t: &Transform,
    anchor: &DragAnchor,
    commands: &mut Commands
)
{
    commands.trigger(DoMoveEvent {
        entity,
        src_parent: anchor.parent,
        src: anchor.pos,
        dst_parent: p,
        dst: t.translation
    });

    commands.entity(entity)
        .remove::<DragAnchor>()
        .insert(Pickable::default());
}
