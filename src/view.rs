use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        event::EntityEvent,
        error::Result,
        observer::On,
        prelude::{ChildOf, Children, Commands, Entity, Has, Query, With}
    },
    input::{
        ButtonInput,
        keyboard::KeyCode
    },
    picking::{
        events::{Click, Pointer, Press},
        pointer::PointerButton
    },
    prelude::{GlobalTransform, State, trace, Transform}
};
use tracing::instrument;

use crate::{
    context_menu::{CloseContextMenus, ContextMenuState, OpenContextMenu},
    double_click::{DoubleClickThreshold, DoubleClickTimer},
    keys::{ctrl_pressed, shift_pressed},
    maxz::MaxZ,
    piece::StackingGroup,
    stack::{self, collapse_stack, Expanded, expand_stack, StackAboveQueryExt},
    select::{deselect_all, Selected, toggle, select}
};

#[derive(Component)]
pub struct RaiseAnchor {
    z: f32
}

#[instrument(skip_all)]
pub fn handle_pressed(
    mut press: On<Pointer<Press>>,
    mut drop_query: Query<(Entity, &mut Transform, &RaiseAnchor)>,
    mut commands: Commands
)
{
    trace!("");

    // prevent the event from bubbling up to parent
    press.propagate(false);

    let entity = press.event().event_target();

    drop_query.iter_mut()
        .filter(|(e, ..)| *e != entity)
        .for_each(|(e, mut t, a)| {
            t.translation.z = a.z;
            commands.entity(e).remove::<RaiseAnchor>();
        });
}

#[instrument(skip_all)]
pub fn handle_piece_pressed(
    mut press: On<Pointer<Press>>,
    modifiers: Res<ButtonInput<KeyCode>>,
    selection_query: Query<Entity, With<Selected>>,
    d_query: Query<(Option<&Children>, &StackingGroup)>,
    a_query: Query<(Option<&ChildOf>, &StackingGroup)>,
    expanded_query: Query<(), With<Expanded>>,
    root_query: Query<&ChildOf>,
    mut maxz_query: Query<&mut MaxZ>,
    mut t_query: Query<(&mut Transform, &GlobalTransform, Has<RaiseAnchor>)>,
    mut commands: Commands
) -> Result
{
    trace!("");

    // prevent the event from bubbling up to parent
    press.propagate(false);

    let entity = press.event().event_target();

    // primary buttons selects
    match press.button {
        PointerButton::Primary if ctrl_pressed(&modifiers) => {
            // ctrl toggles
            trace!("ctrl");
            stack::iter(&a_query, &d_query, entity)
                .for_each(|e| toggle(e, &selection_query, &mut commands));
        },
        PointerButton::Primary if shift_pressed(&modifiers) => {
            // shift adds
            trace!("shift");
            stack::iter(&a_query, &d_query, entity)
                .for_each(|e| select(e, &mut commands));
        },
        PointerButton::Primary | PointerButton::Secondary => {
            if expanded_query.contains(entity) {
                // pressed piece is in an expanded stack

                let (mut t, gt, ra) = t_query.get_mut(entity)?;

                if !ra {
                    // pressed piece is not already raised
                    let root = root_query.root_ancestor(entity);
                    let mut max_z = maxz_query.get_mut(root)?;
                    max_z.0 += 1.0;

                    commands.entity(entity)
                        .insert(RaiseAnchor { z: t.translation.z });

                    t.translation.z += max_z.0 - gt.translation().z;
                }
            }
            else if selection_query.contains(entity) {
                // pressed piece is not in an expanded stack

                // if the target is selected, don't change the selection
                return Ok(());
            }

            // if the target is not selected, select only the target

            // unmodified sets if not selected
            trace!("unmodified");
            deselect_all(&selection_query, &mut commands);

            stack::iter(&a_query, &d_query, entity)
                .for_each(|e| select(e, &mut commands));
        },
        _ => {}
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn handle_piece_clicked(
    mut click: On<Pointer<Click>>,
    mut dct: ResMut<DoubleClickTimer>,
    dc_threshold: Res<DoubleClickThreshold>,
    expanded_query: Query<(), With<Expanded>>,
    selection_query: Query<Entity, With<Selected>>,
    d_query: Query<(Option<&Children>, &StackingGroup)>,
    mut commands: Commands
)
{
    let entity = click.event().event_target();

    click.propagate(false);

    if entity == dct.target &&
        click.button == PointerButton::Primary &&
        dct.timer.elapsed() <= dc_threshold.0
    {
        deselect_all(&selection_query, &mut commands);
        select(d_query.top(entity), &mut commands);

        if expanded_query.contains(entity) {
            collapse_stack(entity, &mut commands);
        }
        else {
            expand_stack(entity, &mut commands);
        }
    }

    dct.target = entity;
    dct.timer.reset();
}

#[instrument(skip_all)]
pub fn handle_context_menu(
    mut press: On<Pointer<Press>>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut commands: Commands
)
{
    trace!("");

    // prevent the event from bubbling up to parent
    press.propagate(false);

    // all buttons close context menus
    if *context_menu_state == ContextMenuState::Open {
        commands.trigger(CloseContextMenus);
    }

    // secondary button opens a context menu
    if press.button == PointerButton::Secondary {
        let entity = press.event().event_target();

        commands.trigger(OpenContextMenu {
            entity,
            pos: press.pointer_location.position
        });
    }
}
