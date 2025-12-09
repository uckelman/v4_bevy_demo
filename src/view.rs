use bevy::{
    ecs::{
        change_detection::Res,
        observer::On,
        prelude::{Commands, Query, With}
    },
    input::{
        ButtonInput,
        keyboard::KeyCode
    },
    picking::{
        events::{Pointer, Press},
        pointer::PointerButton
    },
    prelude::{Entity, EntityEvent, State, trace}
};
use tracing::instrument;

use crate::{
    context_menu::{CloseContextMenus, ContextMenuState, OpenContextMenu},
    select::{Selected, toggle, select, set_selection_if_not_selected, ctrl_pressed, shift_pressed}
};

#[instrument(skip_all)]
pub fn handle_piece_pressed(
    mut press: On<Pointer<Press>>,
    modifiers: Res<ButtonInput<KeyCode>>,
    selection_query: Query<Entity, With<Selected>>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut commands: Commands
)
{
    trace!("");

    // prevent the event from bubbling up to the world
    press.propagate(false);

    // all buttons close context menus
    if *context_menu_state == ContextMenuState::Open {
        commands.trigger(CloseContextMenus);
    }

    // primary, secondary buttons select
    if press.button == PointerButton::Primary ||
        press.button == PointerButton::Secondary
    {
        let entity = press.event().event_target();

        if ctrl_pressed(&modifiers) {
            // ctrl toggles
            trace!("ctrl");
            toggle(entity, &selection_query, &mut commands);
        }
        else if shift_pressed(&modifiers) {
            // shift adds
            trace!("shift");
            select(entity, &mut commands);
        }
        else {
            // unmodified sets if not selected
            trace!("unmodified");
            set_selection_if_not_selected(
                entity,
                &selection_query,
                &mut commands
            );
        }

        // secondary button opens a context menu
        if press.button == PointerButton::Secondary {
            commands.trigger(OpenContextMenu {
                entity,
                pos: press.pointer_location.position
            });
        }
    }
}
