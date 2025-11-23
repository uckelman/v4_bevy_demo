use bevy::prelude::*;
use bevy::{
    asset::LoadedFolder,
    image::ImageSamplerDescriptor,
    input::mouse::{MouseScrollUnit, MouseWheel},
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
        .add_systems(OnEnter(GameState::Splash), (display_title, load_assets))
        .add_systems(Update, switch_to_game.run_if(in_state(GameState::Splash)));
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
                Text::new("May 2025"),
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

fn assets_loaded(
    mut asset_events: MessageReader<AssetEvent<LoadedFolder>>,
    sprite_handles: Res<SpriteHandles>
) -> bool
{
    asset_events.read()
        .any(|e| e.is_loaded_with_dependencies(&sprite_handles.0))
}

fn assets_loaded_x(
    mut asset_events: MessageReader<AssetEvent<Image>>,
)
{
    for e in asset_events.read() {
        eprint!(".");
    }
}

fn switch_to_game(
    mut next: ResMut<NextState<GameState>>,
    asset_events: MessageReader<AssetEvent<LoadedFolder>>,
    asset_events_x: MessageReader<AssetEvent<Image>>,
    sprite_handles: Res<SpriteHandles>,
    mut timer: ResMut<SplashScreenTimer>,
    time: Res<Time>
) {
    assets_loaded_x(asset_events_x);

    if timer.0.tick(time.delta()).is_finished()
        && assets_loaded(asset_events, sprite_handles)
    {
        next.set(GameState::Game);
    }
}

fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), display_game)
        .add_systems(
            Update,
            control_input.run_if(in_state(GameState::Game)),
        );
}

#[derive(Resource)]
struct Surface {
    max_z: f32
}

#[derive(Component)]
struct Map;

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
                Sprite::from_image(handle),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Map,
                DespawnOnExit(GameState::Game)
            ));
        }
        else {
            let x = rng.random_range(-500.0..=500.0);
            let y = rng.random_range(-500.0..=500.0);

            commands.spawn((
                Sprite::from_image(handle),
                Transform::from_xyz(x, y, surface.max_z),
                Pickable::default(),
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

fn on_camera_drag(
    drag: On<Pointer<Drag>>,
    c_query: Single<(&Camera, &GlobalTransform, &mut Transform)>
) -> Result
{
    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    let (camera, global_transform, mut transform) = c_query.into_inner();

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
    mut transforms: Query<&mut Transform>,
    mut surface: ResMut<Surface>
) -> Result
{
    if press.button != PointerButton::Primary {
        return Ok(());
    }

    let mut transform = transforms.get_mut(press.event().event_target())?;

    surface.max_z = surface.max_z.next_up();
    transform.translation.z = surface.max_z;

    // prevent the event from bubbling up to the world
    press.propagate(false);

    Ok(())
}

fn on_piece_drag(
    mut drag: On<Pointer<Drag>>,
    mut transforms: Query<&mut Transform, Without<Camera>>,
    tp_query: Query<(&Transform, &Projection), With<Camera>>
) -> Result
{
    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    let mut transform = transforms.get_mut(drag.event().event_target())?;
    let (camera_transform, camera_projection) = tp_query.single()?;
    let Projection::Orthographic(camera_projection) = camera_projection else {
        panic!("Projection is not orthographic!");
    };

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

fn control_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut tp_query: Query<(&mut Transform, &mut Projection)>,
    time: Res<Time>
) -> Result {

    let (mut transform, mut projection) = tp_query.single_mut()?;

    let Projection::Orthographic(ref mut projection) = *projection else {
        panic!("Projection is not orthographic!");
    };

    // zoom

    let wheel_scale_step = 0.1;
    let key_scale_step = 0.1;

    let mut ds = 0.0;

    ds += mouse_wheel_events
        .read()
        .map(|e| match e.unit {
            MouseScrollUnit::Line => e.y * wheel_scale_step,
            MouseScrollUnit::Pixel => e.y
        })
        .sum::<f32>();

    if keyboard_input.pressed(KeyCode::Equal) { // actually Plus
        ds += key_scale_step / (1.0 / (60.0 * time.delta_secs()));
    }

    if keyboard_input.pressed(KeyCode::Minus) {
        ds -= key_scale_step / (1.0 / (60.0 * time.delta_secs()));
    }

    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) && keyboard_input.pressed(KeyCode::Digit0) {
        projection.scale = 1.0;
    }

    if ds != 0.0 {
        projection.scale *= (-ds).exp();
    }

    // pan

    let mut pan_delta = Vec2::ZERO;

    let key_pan_step = 5.0;

    if keyboard_input.pressed(KeyCode::KeyA) {
        pan_delta.x -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        pan_delta.x += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyW) {
        pan_delta.y += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        pan_delta.y -= 1.0;
    }

    if pan_delta != Vec2::ZERO {
        pan_delta *= key_pan_step / (1.0 / (60.0 * time.delta_secs()));
    }

    if pan_delta != Vec2::ZERO {
        let mut pan_delta = pan_delta.extend(0.0);

        // apply current scale to the pan
        pan_delta *= projection.scale;

        // apply current rotation to the pan
        pan_delta = transform.rotation * pan_delta;

        transform.translation += pan_delta;
    }

    // rotate

    let rotate_step = 1.0f32.to_radians();

    let mut dt = 0.0;

    if keyboard_input.pressed(KeyCode::KeyZ) {
        dt -= rotate_step; // ccw
    }

    if keyboard_input.pressed(KeyCode::KeyX) {
        dt += rotate_step; // cw
    }

    if dt != 0.0 {
        dt /= 1.0 / (60.0 * time.delta_secs());
        transform.rotate_local_z(dt);
    }

    Ok(())
}
