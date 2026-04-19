use bevy::{
    ecs::{
        change_detection::Res,
        event::EntityEvent,
        observer::On,
        prelude::{ChildOf, Children, Commands, Entity, Query, With}
    },
    input::{
        ButtonInput,
        keyboard::KeyCode
    },
    picking::{
        events::{Pointer, Press},
        pointer::PointerButton
    },
    prelude::{State, trace}
};
use tracing::instrument;

use crate::{
    context_menu::{CloseContextMenus, ContextMenuState, OpenContextMenu},
    keys::{ctrl_pressed, shift_pressed},
    piece::StackingGroup,
    stack,
    select::{deselect_all, Selected, toggle, select}
};

#[instrument(skip_all)]
pub fn handle_piece_pressed(
    mut press: On<Pointer<Press>>,
    modifiers: Res<ButtonInput<KeyCode>>,
    selection_query: Query<Entity, With<Selected>>,
    sg_query: Query<&StackingGroup>,
    d_query: Query<(Option<&Children>, &StackingGroup)>,
    a_query: Query<(Option<&ChildOf>, &StackingGroup)>,
    mut commands: Commands
)
{
    trace!("");

    // prevent the event from bubbling up
    press.propagate(false);

    // primary buttons selects
    if press.button == PointerButton::Primary
    {
        let entity = press.event().event_target();

        let Ok(entity_sg) = sg_query.get(entity) else { return; };

        let stack_iter = stack::iter(&a_query, &d_query, entity);

        if ctrl_pressed(&modifiers) {
            // ctrl toggles
            trace!("ctrl");
            stack_iter.for_each(|e| toggle(e, &selection_query, &mut commands));
        }
        else if shift_pressed(&modifiers) {
            // shift adds
            trace!("shift");
            stack_iter.for_each(|e| select(e, &mut commands));
        }
        else {
            // unmodified sets if not selected
            trace!("unmodified");
            deselect_all(&selection_query, &mut commands);
            stack_iter.for_each(|e| select(e, &mut commands));
        }
    }
    else if press.button == PointerButton::Secondary &&
        !ctrl_pressed(&modifiers) &&
        !shift_pressed(&modifiers)
    {
        // if the target is selected, don't change the selection
        // if the target is not selected, select only the target

        let entity = press.event().event_target();

        let Ok(entity_sg) = sg_query.get(entity) else { return; };

        if selection_query.contains(entity) {
            return;
        }

        let stack_iter = stack::iter(&a_query, &d_query, entity);

        // unmodified sets if not selected
        trace!("unmodified");
        deselect_all(&selection_query, &mut commands);
        stack_iter.for_each(|e| select(e, &mut commands));
    }
}

#[instrument(skip_all)]
pub fn handle_context_menu(
    mut press: On<Pointer<Press>>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut commands: Commands
)
{
    trace!("");

    // prevent the event from bubbling up
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
