use bevy::prelude::*;
use bevy::{
    asset::LoadedFolder,
    image::ImageSamplerDescriptor,
    input::{
        common_conditions::input_pressed,
        mouse::AccumulatedMouseScroll
    }
};
use rand::Rng;

mod assets;
mod config;
mod drag;
mod view_adjust;
mod raise;
mod select;
mod state;
mod title;
mod util;

use crate::assets::{
   SpriteHandles, load_assets, is_folder_loaded, log_images_loaded
};
use crate::config::KeyConfig;
use crate::drag::{Draggable, on_piece_drag_start, on_piece_drag, on_piece_drag_end};
use crate::view_adjust::{
    handle_pan_left, handle_pan_right, handle_pan_up, handle_pan_down, handle_pan_drag,
    handle_rotate_ccw, handle_rotate_cw,
    handle_zoom_reset, handle_zoom_in, handle_zoom_out, handle_zoom_scroll,
    KeyPanStep, KeyRotateStep, KeyScaleStep,
    PanLeftKey, PanRightKey, PanUpKey, PanDownKey,
    RotateCCWKey, RotateCWKey,
    ZoomInKey, ZoomOutKey,
    WheelScaleStep
};
use crate::raise::RaiseAnchor;
use crate::select::{clear_selection, draw_selection_rect, on_selection, on_deselection, on_piece_pressed, selection_rect_drag_start, selection_rect_drag, selection_rect_drag_end, Selectable, SelectEvent, DeselectEvent, SelectionRect, setup_selection_box};
use crate::state::GameState;
use crate::title::{SplashScreenTimer, display_title};

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

fn splash_plugin(app: &mut App) {
    app
        .add_systems(
            OnEnter(GameState::Splash),
            (
                display_title,
                load_assets,
                load_input_settings,
                setup_selection_box
            )
        )
        .add_systems(
            Update,
            switch_to_game.run_if(in_state(GameState::Splash))
        );
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

fn load_input_settings(mut commands: Commands) {
    commands.insert_resource(KeyPanStep(5.0));
    commands.insert_resource(KeyRotateStep(1.0f32.to_radians()));
    commands.insert_resource(KeyScaleStep(0.1));
    commands.insert_resource(WheelScaleStep(0.1));

    commands.insert_resource(PanLeftKey(KeyCode::KeyA));
    commands.insert_resource(PanRightKey(KeyCode::KeyD));
    commands.insert_resource(PanUpKey(KeyCode::KeyW));
    commands.insert_resource(PanDownKey(KeyCode::KeyS));

    commands.insert_resource(ZoomInKey(KeyCode::Equal));
    commands.insert_resource(ZoomOutKey(KeyCode::Minus));

    commands.insert_resource(RotateCCWKey(KeyCode::KeyZ));
    commands.insert_resource(RotateCWKey(KeyCode::KeyX));
}

fn cfg_input_pressed<T>(
    key: Res<T>,
    inputs: Res<ButtonInput<KeyCode>>
) -> bool
where
    T: Resource + KeyConfig
{
    inputs.pressed(key.code())
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
                handle_pan_left.run_if(cfg_input_pressed::<PanLeftKey>),
                handle_pan_right.run_if(cfg_input_pressed::<PanRightKey>),
                handle_pan_up.run_if(cfg_input_pressed::<PanUpKey>),
                handle_pan_down.run_if(cfg_input_pressed::<PanDownKey>),

                handle_rotate_ccw.run_if(cfg_input_pressed::<RotateCCWKey>),
                handle_rotate_cw.run_if(cfg_input_pressed::<RotateCWKey>),

// TODO: switch to configurable input
                handle_zoom_reset.run_if(
                    input_pressed(KeyCode::Digit0).and(
                        input_pressed(KeyCode::ControlLeft).or(
                            input_pressed(KeyCode::ControlRight)
                        )
                    )
                ),
                handle_zoom_in.run_if(cfg_input_pressed::<ZoomInKey>),
                handle_zoom_out.run_if(cfg_input_pressed::<ZoomOutKey>),
                handle_zoom_scroll.run_if(
                    resource_changed::<AccumulatedMouseScroll>.and(
                        not(resource_equals(AccumulatedMouseScroll::default()))
                    )
                ),

                draw_selection_rect.run_if(
                    resource_exists::<SelectionRect>
                        .and(|r: Res<SelectionRect>| r.active)
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
    pickable: Pickable,
    selectable: Selectable,
    draggable: Draggable
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
        .observe(handle_pan_drag)
        .observe(clear_selection)
        .observe(selection_rect_drag_start)
        .observe(selection_rect_drag)
        .observe(selection_rect_drag_end);

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
            .observe(recolor_on::<Pointer<Over>>(Color::hsl(0.0, 0.9, 0.7)))
            .observe(recolor_on::<Pointer<Out>>(Color::WHITE))
            .observe(recolor_on::<Pointer<Press>>(Color::srgb(0.0, 0.0, 1.0)))
            .observe(recolor_on::<Pointer<Release>>(Color::BLACK))
*/
            .observe(recolor_on::<SelectEvent>(Color::hsl(0.0, 0.9, 0.7)))
            .observe(recolor_on::<DeselectEvent>(Color::WHITE))
            .observe(on_piece_pressed)
            .observe(raise::on_piece_pressed)
            .observe(raise::on_piece_released)
            .observe(on_piece_drag_start)
            .observe(on_piece_drag)
            .observe(on_piece_drag_end)
            .observe(on_selection)
            .observe(on_deselection);
        }
    }

    commands.insert_resource(surface);
    commands.insert_resource(RaiseAnchor::default());
    commands.insert_resource(SelectionRect::default());

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

fn recolor_on<E: EntityEvent>(color: Color) -> impl Fn(On<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        if let Ok(mut sprite) = sprites.get_mut(ev.event().event_target()) {
            sprite.color = color;
        }
    }
}

// TODO: context menus
// TODO: selection

// TODO: try turning off vsync to fix drag lag

/*
store base positions and always calculate delta on that. translation = translation_start + screen_to_local(pointer_current - pointer_start)
but im suspecting that the drag event is either being fired less often than you think, or the system itself is on a wrong schedule
*/
