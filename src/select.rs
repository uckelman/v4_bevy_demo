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
    math::{Rect, Vec2},
    picking::{
        events::{Drag, DragEnd, DragStart, Pointer, Press},
        pointer::PointerButton
    },
    prelude::{Color, debug, Entity, GlobalTransform, Isometry2d, trace, Transform, Resource, Vec3Swizzles}
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

    trace!("selected {}", entity);
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

    trace!("deselected {}", entity);
}

pub fn shift_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

pub fn ctrl_pressed(inputs: &Res<ButtonInput<KeyCode>>) -> bool {
    inputs.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
}

pub fn toggle(
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

pub fn select(entity: Entity, commands: &mut Commands)
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

pub fn set_selection_if_not_selected(
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

    if drag.button != PointerButton::Middle {
        return;
    }

    selection.active = false;

    // TODO: checking all Selectables is probably slow, maybe use a quadtree?
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

fn rect_inner(size: Vec2) -> [Vec2; 4] {
    let half_size = size / 2.;
    let tl = Vec2::new(-half_size.x, half_size.y);
    let tr = Vec2::new(half_size.x, half_size.y);
    let bl = Vec2::new(-half_size.x, -half_size.y);
    let br = Vec2::new(half_size.x, -half_size.y);
    [tl, tr, br, bl]
}

#[instrument(skip_all)]
pub fn draw_selection_rect(
    selection: Res<SelectionRect>,
    mut gizmos: Gizmos
)
{
    trace!("");

// TODO: Once joint drawing is fixed in Bevy, switch back to rect_2d
/*
    gizmos.rect_2d(
        selection.rect.center(),
        selection.rect.size(),
        Color::srgb_u8(0xFF, 0, 0)
    );
*/
    let color = Color::srgb_u8(0xFF, 0, 0);
    let size = selection.rect.size();
    let isometry: Isometry2d = selection.rect.center().into();
    let [tl, tr, br, bl] = rect_inner(size).map(|vec2| isometry * vec2);
    gizmos.linestrip_2d([tl, tr, br, bl, tl, tr], color);
}

// TODO: look into bevy_vector_shapes for drawing selection box

pub fn setup_selection_box(
    mut config_store: ResMut<GizmoConfigStore>
)
{
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = 5.0;
    config.line.joints = GizmoLineJoint::Miter;
}
