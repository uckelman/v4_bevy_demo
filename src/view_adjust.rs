use bevy::{
    camera::Camera,
    ecs::{
        change_detection::Res,
        error::Result,
        observer::On,
        prelude::{Query, Single, With}
    },
    input::mouse::{AccumulatedMouseScroll, MouseScrollUnit},
    math::Vec2,
    picking::events::{Drag, Pointer},
    prelude::{GlobalTransform, Projection, OrthographicProjection, PointerButton, Time, trace, Transform}
};

use crate::config::{
    KeyPanStep, KeyRotateStep, KeyScaleStep, WheelScaleStep
};
use crate::util::AsOrthographicProjection;

fn pan_view(
    transform: &mut Transform,
    projection: &OrthographicProjection,
    pan_delta: Vec2
)
{
    let mut pan_delta = pan_delta.extend(0.0);

    // apply current scale to the pan
    pan_delta *= projection.scale;

    // apply current rotation to the pan
    pan_delta = transform.rotation * pan_delta;

    transform.translation += pan_delta;
}

fn handle_pan(
    mut query: Query<(&mut Transform, &Projection), With<Camera>>,
    step: Res<KeyPanStep>,
    time: Res<Time>,
    mut pan_delta: Vec2
) -> Result
{
    let (mut transform, projection) = query.single_mut()?;

    pan_delta *= step.0 / (1.0 / (60.0 * time.delta_secs()));

    pan_view(&mut transform, projection.as_ortho()?, pan_delta);

    Ok(())
}

pub fn handle_pan_left(
    query: Query<(&mut Transform, &Projection), With<Camera>>,
    step: Res<KeyPanStep>,
    time: Res<Time>
) -> Result
{
    trace!("handle_pan_left");
    handle_pan(query, step, time, Vec2::NEG_X)
}

pub fn handle_pan_right(
    query: Query<(&mut Transform, &Projection), With<Camera>>,
    step: Res<KeyPanStep>,
    time: Res<Time>
) -> Result
{
    trace!("handle_pan_right");
    handle_pan(query, step, time, Vec2::X)
}

pub fn handle_pan_up(
    query: Query<(&mut Transform, &Projection), With<Camera>>,
    step: Res<KeyPanStep>,
    time: Res<Time>
) -> Result
{
    trace!("handle_pan_up");
    handle_pan(query, step, time, Vec2::Y)
}

pub fn handle_pan_down(
    query: Query<(&mut Transform, &Projection), With<Camera>>,
    step: Res<KeyPanStep>,
    time: Res<Time>
) -> Result
{
    trace!("handle_pan_down");
    handle_pan(query, step, time, Vec2::NEG_Y)
}

pub fn handle_pan_drag(
    drag: On<Pointer<Drag>>,
    query: Single<(&Camera, &GlobalTransform, &mut Transform)>
) -> Result
{
    trace!("on_camera_drag");

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    let (camera, global_transform, mut transform) = query.into_inner();

    let mut viewport = camera.world_to_viewport(global_transform, transform.translation)?;
    viewport += drag.delta * -1.0; // inverted feels more natural
    transform.translation = camera.viewport_to_world_2d(global_transform, viewport)?.extend(0.0);

    Ok(())
}

fn rotate_view(
    transform: &mut Transform,
    dt: f32
)
{
    transform.rotate_local_z(dt);
}

fn handle_rotate(
    mut query: Query<&mut Transform, With<Camera>>,
    step: Res<KeyRotateStep>,
    time: Res<Time>,
    mut dt: f32
) -> Result
{
    if dt != 0.0 {
        let mut transform = query.single_mut()?;
        dt *= step.0;
        dt /= 1.0 / (60.0 * time.delta_secs());
        rotate_view(&mut transform, dt);
    }

    Ok(())
}

pub fn handle_rotate_ccw(
    query: Query<&mut Transform, With<Camera>>,
    step: Res<KeyRotateStep>,
    time: Res<Time>
) -> Result
{
    trace!("handle_rotate_ccw");
    handle_rotate(query, step, time, -1.0)
}

pub fn handle_rotate_cw(
    query: Query<&mut Transform, With<Camera>>,
    step: Res<KeyRotateStep>,
    time: Res<Time>
) -> Result
{
    trace!("handle_rotate_cw");
    handle_rotate(query, step, time, 1.0)
}

fn zoom_view_set(
    projection: &mut OrthographicProjection,
    s: f32
)
{
    projection.scale = s;
}

pub fn handle_zoom_reset(
    mut query: Query<&mut Projection, With<Camera>>
) -> Result
{
    trace!("handle_zoom_reset");
    let mut projection = query.single_mut()?;
    zoom_view_set(projection.as_ortho_mut()?, 1.0);
    Ok(())
}

fn zoom_view(
    projection: &mut OrthographicProjection,
    ds: f32
)
{
    if ds != 0.0 {
        projection.scale *= (-ds).exp();
    }
}

fn handle_zoom(
    mut query: Query<&mut Projection, With<Camera>>,
    step: Res<KeyScaleStep>,
    time: Res<Time>,
    mut ds: f32
) -> Result
{
    let mut projection = query.single_mut()?;

    ds *= step.0 / (1.0 / (60.0 * time.delta_secs()));

    if ds != 0.0 {
        zoom_view(projection.as_ortho_mut()?, ds);
    }

    Ok(())
}

pub fn handle_zoom_in(
    query: Query<&mut Projection, With<Camera>>,
    step: Res<KeyScaleStep>,
    time: Res<Time>
) -> Result
{
    trace!("handle_zoom_in");
    handle_zoom(query, step, time, 1.0)
}

pub fn handle_zoom_out(
    query: Query<&mut Projection, With<Camera>>,
    step: Res<KeyScaleStep>,
    time: Res<Time>
) -> Result
{
    trace!("handle_zoom_out");
    handle_zoom(query, step, time, -1.0)
}

pub fn handle_zoom_scroll(
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut Projection, With<Camera>>,
    step: Res<WheelScaleStep>
) -> Result
{
    trace!("handle_mouse_scroll");

    let ds = match mouse_scroll.unit {
        MouseScrollUnit::Line => mouse_scroll.delta.y * step.0,
        MouseScrollUnit::Pixel => mouse_scroll.delta.y
    };

    if ds != 0.0 {
        let mut projection = query.single_mut()?;
        zoom_view(projection.as_ortho_mut()?, ds);
    }

    Ok(())
}
