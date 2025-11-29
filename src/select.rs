use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        error::Result,
        event::EntityEvent,
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
    prelude::{debug, Entity, trace}
};
use tracing::instrument;

#[derive(Component, Default)]
pub struct Selectable;

#[derive(Component, Default)]
pub struct Selected;

#[derive(EntityEvent)]
pub struct SelectEvent {
    entity: Entity
}

#[derive(EntityEvent)]
pub struct DeselectEvent {
    entity: Entity
}

#[instrument(skip_all)]
pub fn on_piece_selection(
    select: On<SelectEvent>,
    mut commands: Commands
)
{
    trace!("");

    let entity = select.event().event_target();
    commands.entity(entity).insert(Selected);

    debug!("selected {}", entity);
}

#[instrument(skip_all)]
pub fn on_piece_deselection(
    select: On<DeselectEvent>,
    mut commands: Commands
)
{
    trace!("");

    let entity = select.event().event_target();
    commands.entity(entity).remove::<Selected>();

    debug!("deselected {}", entity);
}

fn shift_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

fn ctrl_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
}

fn toggle_selection(
    entity: Entity,
    query: &Query<Entity, With<Selected>>,
    commands: &mut Commands
)
{
    match query.contains(entity) {
        true => commands.trigger(DeselectEvent { entity }),
        false => commands.trigger(SelectEvent { entity })
    }
}

fn add_to_selection(
    entity: Entity,
    query: &Query<Entity, With<Selected>>,
    commands: &mut Commands
)
{
    if !query.contains(entity) {
        commands.trigger(SelectEvent { entity });
    }
}

fn clear_selection(
    query: &Query<Entity, With<Selected>>,
    commands: &mut Commands
)
{
    query.iter()
        .for_each(|entity| commands.trigger(DeselectEvent { entity }));
}

fn set_selection_if_not_selected(
    entity: Entity,
    query: &Query<Entity, With<Selected>>,
    commands: &mut Commands
)
{
    if !query.contains(entity) {
        clear_selection(query, commands);
        commands.trigger(SelectEvent { entity });
    }
}

#[instrument(skip_all)]
pub fn on_piece_pressed(
    mut press: On<Pointer<Press>>,
    modifiers: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
) -> Result
{
    trace!("");

    if press.button != PointerButton::Primary {
        return Ok(());
    }

    let entity = press.event().event_target();

    if ctrl_pressed(&modifiers) {
        // ctrl toggles
        trace!("ctrl");
        toggle_selection(entity, &query, &mut commands);
    }
    else if shift_pressed(&modifiers) {
        // shift adds
        trace!("shift");
        add_to_selection(entity, &query, &mut commands);
    }
    else {
        // unmodified sets if not selected
        trace!("unmodified");
        set_selection_if_not_selected(entity, &query, &mut commands);
    }

    // prevent the event from bubbling up to the world
    press.propagate(false);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_nonpiece_pressed(
    press: On<Pointer<Press>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
) -> Result
{
    trace!("");

    if press.button != PointerButton::Primary {
        return Ok(());
    }

    // clear selection
    clear_selection(&query, &mut commands);

    Ok(())
}
