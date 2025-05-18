use bevy::prelude::*;
use bevy::{
    image::ImageSamplerDescriptor,
    input::mouse::{MouseScrollUnit, MouseWheel},
};

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
        .enable_state_scoped_entities::<GameState>()
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

#[derive(Resource)]
struct Assets {
    map: Handle<Image>
}

fn splash_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Splash), (display_title, load_assets))
        .add_systems(Update, switch_to_game.run_if(in_state(GameState::Splash)));
}

fn display_title(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
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
        StateScoped(GameState::Splash),
    ));

    commands.insert_resource(
        SplashScreenTimer(Timer::from_seconds(2.0, TimerMode::Once))
    );
}

#[derive(Resource)]
struct SplashScreenTimer(Timer);

fn switch_to_game(
    mut next: ResMut<NextState<GameState>>,
    mut timer: ResMut<SplashScreenTimer>,
    time: Res<Time>
) {
    if timer.0.tick(time.delta()).just_finished() {
        next.set(GameState::Game);
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Assets {
        map: asset_server.load("map.png")
    });
}

fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), display_game)
        .add_systems(
            Update,
            control_input.run_if(in_state(GameState::Game)),
        );
}

#[derive(Component)]
struct Map;

fn display_game(mut commands: Commands, game_assets: Res<Assets>) {
    commands.spawn((
        Sprite::from_image(game_assets.map.clone()),
        Map,
        StateScoped(GameState::Game)
    ));
}

fn control_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut cursor_motion_events: EventReader<CursorMoved>,
    mut tp_query: Query<(&mut Transform, &mut Projection)>,
    mut prev_pos: Local<Option<Vec2>>,
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

    if mouse_button_input.just_released(MouseButton::Left) {
        *prev_pos = None;
    }
    else if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {

        if let Some(cm_event) = cursor_motion_events.read().last() {
            let cur_pos = cm_event.position;
            if let Some(prev_pos) = *prev_pos {
                pan_delta = cur_pos - prev_pos;
                pan_delta.x = -pan_delta.x;
            }

            *prev_pos = Some(cur_pos);
        }
    }
    else {
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
    }

    if pan_delta != Vec2::ZERO {
        // apply current scale to the pan
        pan_delta = (projection.scale * pan_delta.extend(0.0)).truncate();

        // apply current rotation to the pan
        pan_delta = (transform.rotation * pan_delta.extend(0.0)).truncate();

        transform.translation += pan_delta.extend(0.0);
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
        dt /= 1.0 / (60.0 * time.delta().as_secs_f32());
        transform.rotate_local_z(dt);
    }

    Ok(())
}
