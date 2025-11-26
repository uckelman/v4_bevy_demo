use bevy::prelude::*;
use bevy::{
    asset::LoadedFolder,
    image::ImageSamplerDescriptor,
    input::{
        common_conditions::input_pressed,
        mouse::{AccumulatedMouseScroll, MouseScrollUnit}
    }
};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "V4 Bevy Demo".into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    anisotropy_clamp: 16,
                    ..ImageSamplerDescriptor::linear()
                }
            })
        )
        .init_state::<GameState>()
        .add_plugins((
            splash_plugin,
            game_plugin
        ))
        .run();
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
enum GameState {
    #[default]
    Splash,
    Game
}

fn splash_plugin(app: &mut App) {
    app
        .add_systems(
            OnEnter(GameState::Splash),
            (display_title, load_assets)
        )
        .add_systems(
            Update,
            switch_to_game.run_if(in_state(GameState::Splash))
        );
}

#[derive(Resource)]
struct SplashScreenTimer(Timer);

fn display_title(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection::default_2d())
    ));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (
                Text::new("V4 Bevy Demo"),
                TextFont {
                    font_size: 130.0,
                    ..default()
                },
            ),
            (
                Text::new("November 2025"),
                TextFont {
                    font_size: 100.0,
                    ..default()
                },
            )
        ],
        DespawnOnExit(GameState::Splash),
    ));

    commands.insert_resource(
        SplashScreenTimer(Timer::from_seconds(2.0, TimerMode::Once))
    );
}

#[derive(Resource)]
struct SpriteHandles(Handle<LoadedFolder>);

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>
)
{
    commands.insert_resource(SpriteHandles(asset_server.load_folder(".")));
}

fn is_folder_loaded(
    mut asset_events: MessageReader<AssetEvent<LoadedFolder>>,
    sprite_handles: Res<SpriteHandles>
) -> bool
{
    asset_events.read()
        .any(|e| e.is_loaded_with_dependencies(&sprite_handles.0))
}

fn log_images_loaded(
    mut asset_events: MessageReader<AssetEvent<Image>>,
)
{
    for e in asset_events.read() {
        eprint!(".");
    }
}

fn switch_to_game(
    mut next: ResMut<NextState<GameState>>,
    asset_events_folder: MessageReader<AssetEvent<LoadedFolder>>,
    asset_events_image: MessageReader<AssetEvent<Image>>,
    sprite_handles: Res<SpriteHandles>,
    mut timer: ResMut<SplashScreenTimer>,
    time: Res<Time>
) {
    log_images_loaded(asset_events_image);

    if timer.0.tick(time.delta()).is_finished()
        && is_folder_loaded(asset_events_folder, sprite_handles)
    {
        next.set(GameState::Game);
    }
}

fn game_plugin(app: &mut App) {
    app
        .add_systems(
            OnEnter(GameState::Game),
            display_game
        )
        .add_systems(
            Update,
            (
                handle_pan_left.run_if(input_pressed(KeyCode::KeyA)),
                handle_pan_right.run_if(input_pressed(KeyCode::KeyD)),
                handle_pan_up.run_if(input_pressed(KeyCode::KeyW)),
                handle_pan_down.run_if(input_pressed(KeyCode::KeyS)),

                handle_rotate_ccw.run_if(input_pressed(KeyCode::KeyZ)),
                handle_rotate_cw.run_if(input_pressed(KeyCode::KeyX)),

                handle_zoom_reset.run_if(
                    input_pressed(KeyCode::Digit0).and(
                        input_pressed(KeyCode::ControlLeft).or(
                            input_pressed(KeyCode::ControlRight)
                        )
                    )
                ),
                handle_zoom_in.run_if(input_pressed(KeyCode::Equal)),
                handle_zoom_out.run_if(input_pressed(KeyCode::Minus)),
                handle_zoom_scroll.run_if(
                    resource_changed::<AccumulatedMouseScroll>.and(
                        not(resource_equals(AccumulatedMouseScroll::default()))
                    )
                )
            )
            .run_if(in_state(GameState::Game))
        );
}

#[derive(Resource)]
struct Surface {
    max_z: f32
}

#[derive(Component, Default)]
struct Map;

#[derive(Bundle, Default)]
struct MapBundle {
    marker: Map,
    sprite: Sprite,
    transform: Transform
}

#[derive(Component, Default)]
struct Piece;

#[derive(Bundle, Default)]
struct PieceBundle {
    marker: Piece,
    sprite: Sprite,
    transform: Transform,
    pickable: Pickable
}

fn display_game(
    mut commands: Commands,
    window: Single<Entity, With<Window>>,
    sprite_handles: Res<SpriteHandles>,
    loaded_folders: Res<Assets<LoadedFolder>>,
) -> Result
{
    let Some(loaded_folder) = loaded_folders.get(&sprite_handles.0) else {
        return Ok(());
    };

    commands.entity(*window)
        .observe(on_camera_drag);

    let mut surface = Surface { max_z: 0.0 };

    let mut rng = rand::rng();

    for handle in loaded_folder.handles.iter() {
        let handle = handle.clone().try_typed::<Image>()?;

        surface.max_z = surface.max_z.next_up();

        let Some(path) = handle.path() else {
            continue;
        };

        if path.to_string() == "map.png" {
            commands.spawn((
                MapBundle {
                    sprite: Sprite::from_image(handle),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..Default::default()
                },
                DespawnOnExit(GameState::Game)
            ));
        }
        else {
            let x = rng.random_range(-500.0..=500.0);
            let y = rng.random_range(-500.0..=500.0);

            commands.spawn((
                PieceBundle {
                    sprite: Sprite::from_image(handle),
                    transform: Transform::from_xyz(x, y, surface.max_z),
                    ..Default::default()
                },
                DespawnOnExit(GameState::Game)
            ))
/*
            .observe(recolor_on::<Pointer<Over>>(Color::srgb(0.0, 1.0, 0.0)))
            .observe(recolor_on::<Pointer<Out>>(Color::BLACK))
            .observe(recolor_on::<Pointer<Pressed>>(Color::srgb(0.0, 0.0, 1.0)))
            .observe(recolor_on::<Pointer<Released>>(Color::BLACK))
*/
            .observe(on_piece_pressed)
            .observe(on_piece_drag);
        }
    }

    commands.insert_resource(surface);

    Ok(())
}

trait AsOrthographicProjection {
    fn as_ortho(&self) -> &OrthographicProjection;

    fn as_ortho_mut(&mut self) -> &mut OrthographicProjection;
}

impl AsOrthographicProjection for Projection {
    fn as_ortho(&self) -> &OrthographicProjection {
        match *self {
            Projection::Orthographic(ref p) => p,
            _ => panic!("Projection is not orthographic!")
        }
    }

    fn as_ortho_mut(&mut self) -> &mut OrthographicProjection {
        match *self {
            Projection::Orthographic(ref mut p) => p,
            _ => panic!("Projection is not orthographic!")
        }
    }
}

fn on_camera_drag(
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

/*
fn on_camera_scroll(
    scroll: Trigger<Pointer<Scroll>>,
    proj: Single<&mut Projection, With<Camera>>)
{
    if let Projection::Orthographic(ref mut proj) = *proj.into_inner() {
        proj.scale *= 1. - (scroll.y / 5.);
    }
}
*/

fn recolor_on<E: Clone + EntityEvent + Reflect>(color: Color) -> impl Fn(On<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        let Ok(mut sprite) = sprites.get_mut(ev.event().event_target()) else {
            return;
        };
        sprite.color = color;
    }
}

fn on_piece_pressed(
    mut press: On<Pointer<Press>>,
    mut query: Query<&mut Transform, With<Piece>>,
    mut surface: ResMut<Surface>
) -> Result
{
    if press.button != PointerButton::Primary {
        return Ok(());
    }

    let mut transform = query.get_mut(press.event().event_target())?;

    surface.max_z = surface.max_z.next_up();
    transform.translation.z = surface.max_z;

    // prevent the event from bubbling up to the world
    press.propagate(false);

    Ok(())
}

fn on_piece_drag(
    mut drag: On<Pointer<Drag>>,
    mut p_query: Query<&mut Transform, (With<Piece>, Without<Camera>)>,
    tp_query: Query<(&Transform, &Projection), With<Camera>>
) -> Result
{
    trace!("on_piece_drag");

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    let mut transform = p_query.get_mut(drag.event().event_target())?;
    let (camera_transform, camera_projection) = tp_query.single()?;

    let camera_projection = camera_projection.as_ortho();

    let mut drag_delta = drag.delta.extend(0.0);
    drag_delta.y = -drag_delta.y;

    // apply current scale to the drag
    drag_delta *= camera_projection.scale;

    // apply current rotation to the drag
    drag_delta = camera_transform.rotation * drag_delta;

    transform.translation += drag_delta;

    // prevent the event from bubbling up to the world
    drag.propagate(false);

    Ok(())
}

fn pan_view(
    transform: &mut Transform,
    projection: &mut Projection,
    mut pan_delta: Vec2
)
{
    let mut pan_delta = pan_delta.extend(0.0);

    let mut projection = projection.as_ortho_mut();

    // apply current scale to the pan
    pan_delta *= projection.scale;

    // apply current rotation to the pan
    pan_delta = transform.rotation * pan_delta;

    transform.translation += pan_delta;
}

fn handle_pan(
    mut query: Query<(&mut Transform, &mut Projection), With<Camera>>,
    time: Res<Time>,
    mut pan_delta: Vec2
) -> Result
{
    let (mut transform, mut projection) = query.single_mut()?;

    let key_pan_step = 5.0;
    pan_delta *= key_pan_step / (1.0 / (60.0 * time.delta_secs()));

    pan_view(&mut transform, &mut projection, pan_delta);

    Ok(())
}

fn handle_pan_left(
    query: Query<(&mut Transform, &mut Projection), With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_pan_left");
    handle_pan(query, time, Vec2::NEG_X)
}

fn handle_pan_right(
    query: Query<(&mut Transform, &mut Projection), With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_pan_right");
    handle_pan(query, time, Vec2::X)
}

fn handle_pan_up(
    query: Query<(&mut Transform, &mut Projection), With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_pan_up");
    handle_pan(query, time, Vec2::Y)
}

fn handle_pan_down(
    query: Query<(&mut Transform, &mut Projection), With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_pan_down");
    handle_pan(query, time, Vec2::NEG_Y)
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
    time: Res<Time>,
    mut dt: f32
) -> Result
{
    let mut transform = query.single_mut()?;

    let rotate_step = 1.0f32.to_radians();
    dt *= rotate_step;

    if dt != 0.0 {
        dt /= 1.0 / (60.0 * time.delta_secs());
        rotate_view(&mut transform, dt);
    }

    Ok(())
}

fn handle_rotate_ccw(
    mut query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_rotate_ccw");
    handle_rotate(query, time, -1.0)
}

fn handle_rotate_cw(
    mut query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_rotate_cw");
    handle_rotate(query, time, 1.0)
}

fn zoom_view_set(
    projection: &mut Projection,
    s: f32
)
{
    let mut projection = projection.as_ortho_mut();
    projection.scale = s;
}

fn handle_zoom_reset(
    mut query: Query<&mut Projection, With<Camera>>
) -> Result
{
    trace!("handle_zoom_reset");
    let mut projection = query.single_mut()?;
    zoom_view_set(&mut projection, 1.0);
    Ok(())
}

fn zoom_view(
    projection: &mut Projection,
    mut ds: f32
)
{
    if ds != 0.0 {
        let mut projection = projection.as_ortho_mut();
        projection.scale *= (-ds).exp();
    }
}

fn handle_zoom(
    mut query: Query<&mut Projection, With<Camera>>,
    time: Res<Time>,
    mut ds: f32
) -> Result
{
    let mut projection = query.single_mut()?;

    let key_scale_step = 0.1;

    ds *= key_scale_step / (1.0 / (60.0 * time.delta_secs()));

    if ds != 0.0 {
        zoom_view(&mut projection, ds);
    }

    Ok(())
}

fn handle_zoom_in(
    query: Query<&mut Projection, With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_zoom_in");
    handle_zoom(query, time, 1.0)
}

fn handle_zoom_out(
    query: Query<&mut Projection, With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_zoom_out");
    handle_zoom(query, time, -1.0)
}

fn handle_zoom_scroll(
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut Projection, With<Camera>>,
    time: Res<Time>
) -> Result
{
    trace!("handle_mouse_scroll");

    let wheel_scale_step = 0.1;

    let ds = match mouse_scroll.unit {
        MouseScrollUnit::Line => mouse_scroll.delta.y * wheel_scale_step,
        MouseScrollUnit::Pixel => mouse_scroll.delta.y
    };

    if ds != 0.0 {
        let mut projection = query.single_mut()?;
        zoom_view(&mut projection, ds);
    }

    Ok(())
}
