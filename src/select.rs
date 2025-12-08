use bevy::{
    camera::Camera,
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::{Commands, Query, Single, With}
    },
    gizmos::{
        config::{DefaultGizmoConfigGroup, GizmoConfigStore, GizmoLineJoint},
        gizmos::Gizmos
    },
    input::{
        ButtonInput,
        keyboard::KeyCode
    },
    math::Rect,
    picking::{
        events::{Drag, DragEnd, DragStart, Pointer, Press},
        pointer::PointerButton
    },
    prelude::{Color, debug, Entity, GlobalTransform, trace, Transform, Resource, Vec3Swizzles, Window}
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

#[derive(Resource, Default)]
pub struct SelectionRect {
    pub rect: Rect,
    pub active: bool
}

#[instrument(skip_all)]
pub fn on_selection(
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
pub fn on_deselection(
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

fn toggle(
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

fn select(entity: Entity, commands: &mut Commands)
{
    commands.trigger(SelectEvent { entity });
}

fn deselect(entity: Entity, commands: &mut Commands)
{
    commands.trigger(DeselectEvent { entity });
}

fn deselect_all(
    query: &Query<Entity, With<Selected>>,
    commands: &mut Commands
)
{
    query.iter().for_each(|entity| deselect(entity, commands));
}

fn set_selection_if_not_selected(
    entity: Entity,
    query: &Query<Entity, With<Selected>>,
    commands: &mut Commands
)
{
    if !query.contains(entity) {
        deselect_all(query, commands);
        select(entity, commands);
    }
}

// TODO: check for Selectable?

#[instrument(skip_all)]
pub fn selectable_pressed(
    mut press: On<Pointer<Press>>,
    modifiers: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
)
{
    trace!("");

    if press.button != PointerButton::Primary &&
        press.button != PointerButton::Secondary
    {
        return;
    }

    let entity = press.event().event_target();

    if ctrl_pressed(&modifiers) {
        // ctrl toggles
        trace!("ctrl");
        toggle(entity, &query, &mut commands);
    }
    else if shift_pressed(&modifiers) {
        // shift adds
        trace!("shift");
        select(entity, &mut commands);
    }
    else {
        // unmodified sets if not selected
        trace!("unmodified");
        set_selection_if_not_selected(entity, &query, &mut commands);
    }

    // prevent the event from bubbling up to the world
    press.propagate(false);
}

#[instrument(skip_all)]
pub fn clear_selection(
    press: On<Pointer<Press>>,
    modifiers: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
)
{
    trace!("");

    match press.button {
        PointerButton::Primary |
        PointerButton::Middle if !ctrl_pressed(&modifiers)
            && !shift_pressed(&modifiers) =>
        {
            deselect_all(&query, &mut commands);
        },
        _ => {}
    }
}

#[instrument(skip_all)]
pub fn selection_rect_drag_start(
    drag: On<Pointer<DragStart>>,
    mut selection: ResMut<SelectionRect>
)
{
    trace!("");

    if drag.button == PointerButton::Middle &&
        let Some(pos) = drag.hit.position
    {
        let pos = pos.xy();
        selection.rect = Rect::from_corners(pos, pos);
        selection.active = true;
    }
}

#[instrument(skip_all)]
pub fn selection_rect_drag(
    drag: On<Pointer<Drag>>,
    query: Single<(&Camera, &GlobalTransform)>,
    mut selection: ResMut<SelectionRect>
) -> Result
{
    trace!("");

    if drag.button != PointerButton::Middle {
        return Ok(());
    }

    let (camera, global_transform) = *query;

    let start = camera.viewport_to_world_2d(
        global_transform,
        drag.pointer_location.position - drag.distance
    )?;

    let end = camera.viewport_to_world_2d(
        global_transform,
        drag.pointer_location.position
    )?;

    selection.rect = Rect::from_corners(start, end);

    Ok(())
}

#[instrument(skip_all)]
pub fn selection_rect_drag_end(
    drag: On<Pointer<DragEnd>>,
    modifiers: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform), With<Selectable>>,
    s_query: Query<Entity, With<Selected>>,
    mut selection: ResMut<SelectionRect>,
    mut commands: Commands
)
{
    trace!("");

    if drag.button == PointerButton::Middle {
        selection.active = false;

        // TODO: checking all Selectables is probably slow?
        let qi = query.iter()
            .filter(|(_, transform)|
                selection.rect.contains(transform.translation.xy())
            )
            .map(|(entity, _)| entity);

        if ctrl_pressed(&modifiers) {
            // toggle selection
            qi.for_each(|entity| toggle(entity, &s_query, &mut commands));
        }
        else if shift_pressed(&modifiers) {
            // add to selection
            qi.for_each(|entity| select(entity, &mut commands));
        }
        else {
            // set selection
            deselect_all(&s_query, &mut commands);
            qi.for_each(|entity| select(entity, &mut commands));
        }
    }
}

#[instrument(skip_all)]
pub fn draw_selection_rect(
    selection: Res<SelectionRect>,
    mut gizmos: Gizmos
)
{
    trace!("");
    gizmos.rect_2d(
        selection.rect.center(),
        selection.rect.size(),
        Color::srgb_u8(0xFF, 0, 0)
    );
}

// FIXME: selection box has bad corner joint

pub fn setup_selection_box(
    mut config_store: ResMut<GizmoConfigStore>
)
{
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = 5.0;
    config.line.joints = GizmoLineJoint::Miter;
}
