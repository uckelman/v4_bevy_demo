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
use tracing::instrument;

use crate::{
    context_menu::ContextMenuState,
    log::{OpenGroupEvent, CloseGroupEvent},
    maxz::MaxZ,
    piece::{
        StackingGroup,
        r#move::DoMoveEvent
    },
    raise::raise_piece,
    select::Selected,
    stack::StackBelowQueryExt,
    util::AsOrthographicProjection
};

#[derive(Clone, Component, Copy, Debug,  Default)]
pub struct Draggable;

#[derive(Component)]
pub struct DragAnchor {
    parent: Entity,
    pos: Vec3
}

#[instrument(skip_all)]
pub fn on_piece_drag_start(
    mut drag: On<Pointer<DragStart>>,
    query: Query<(Entity, &ChildOf, &GlobalTransform, &mut Transform), (With<Draggable>, With<Selected>)>,
    parent_query: Query<&ChildOf>,
    a_query: Query<(Option<&ChildOf>, &StackingGroup)>,
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

    // find all the stack bottoms, highest point for the selection
    let mut bottoms = vec![];
    let mut sel_min_z = f32::INFINITY;
    let mut sel_max_z = f32::NEG_INFINITY;

    for (e, p, gt, t) in query {
        if a_query.iter_below(e).next().is_none() {
            bottoms.push((e, p, t));
        }

        let z = gt.translation().z;

        if z < sel_min_z {
            sel_min_z = z;
        }

        if z > sel_max_z {
            sel_max_z = z;
        }

        // make sure drop events do not hit the selection
        commands.entity(e).insert(Pickable::IGNORE);
    }

    // selection must be on one surface; get it from the first selected item
    let Some(first) = bottoms.first() else { return Ok(()); };
    let root = parent_query.root_ancestor(first.0);

    // prevent the event from bubbling up to the world
    drag.propagate(false);

    // raise entire selection by amount the lowest member is below max z
    let mut max_z = maxz_query.get_mut(root)?;
    let dz = max_z.0 - sel_min_z + 1.0;
    max_z.0 = sel_max_z + dz;

    for (entity, parent, mut transform) in bottoms {
        let z = transform.translation.z + dz;
        raise_piece(&mut transform, z);

        // set the drag anchor, prevent picking from hitting piece
        commands.entity(entity)
            .insert(DragAnchor {
                parent: parent.0,
                pos: transform.translation
            });
    }

    Ok(())
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

    // prevent the event from bubbling up to the world
    drag.propagate(false);

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

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag_end(
    mut drag: On<Pointer<DragEnd>>,
    query: Query<(Entity, &ChildOf, &Transform, &DragAnchor)>,
    s_query: Query<Entity, (With<Draggable>, With<Selected>)>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut commands: Commands
)
{
    trace!("");

    if *context_menu_state == ContextMenuState::Open {
        return;
    }

    drag.propagate(false);

    if drag.button == PointerButton::Primary {
        for e in s_query {
            commands.entity(e).insert(Pickable::default());
        }

        // remove the drag anchor, make piece pickable again, move it
        let mut eptai = query.iter();
        match eptai.len() {
            0 => {},
            1 => {
                let (e, p, t, a) = eptai.next().expect("len is 1");
                unanchor_and_move(e, p.0, t, a, &mut commands);
            },
            _ => {
                commands.trigger(OpenGroupEvent);
                eptai.for_each(|(e, p, t, a)| unanchor_and_move(e, p.0, t, a, &mut commands));
                commands.trigger(CloseGroupEvent);
            }
        }
    }
}

fn unanchor_and_move(
    entity: Entity,
    p: Entity,
    t: &Transform,
    anchor: &DragAnchor,
    commands: &mut Commands
)
{
    commands.entity(entity)
        .remove::<DragAnchor>()
        .insert(Pickable::default());

    commands.trigger(DoMoveEvent {
        entity,
        src_parent: anchor.parent,
        src: anchor.pos,
        dst_parent: p,
        dst: t.translation
    });
}
