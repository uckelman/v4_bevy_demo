use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup, Update},
    asset::{
        AssetApp,
        io::AssetSourceBuilder
    },
    ecs::{
        change_detection::{Res, ResMut},
        entity::Entity,
        error::Result,
        prelude::{any_with_component, Commands, not, resource_changed, resource_equals, resource_exists, Single, SystemCondition, With}
    },
    image::{ImagePlugin, ImageSamplerDescriptor},
    input::{
        ButtonInput,
        common_conditions::input_just_pressed,
        keyboard::KeyCode,
        mouse::AccumulatedMouseScroll
    },
    picking::mesh_picking::MeshPickingPlugin,
    prelude::{AppExtStates, IntoScheduleConfigs, in_state, NextState, OnEnter, Resource, Time, trace, Window, WindowPlugin}
};
use std::path::PathBuf;

mod actionfunc;
mod angle;
mod assets;
mod config;
mod context_menu;
mod debug;
mod drag;
mod edittype;
mod gamebox;
mod grid;
mod keys;
mod log;
mod log_deserialize;
mod log_serialize;
mod map;
mod maxz;
mod object;
mod piece;
mod raise;
mod select;
mod state;
mod surface;
mod title;
mod util;
mod view;
mod view_adjust;

use crate::{
    assets::{LoadingHandles, SpriteHandles, load_assets, mark_images_loaded},
    config::{Config, load_config},
    context_menu::{ContextMenuState, open_context_menu, close_context_menus, trigger_close_context_menus_key, trigger_close_context_menus_press, trigger_close_context_menus_wheel},
    debug::{cursor_events, dump_edits, pick_dbg},
    gamebox::GameBox,
    keys::{cfg_input_pressed, cfg_input_just_pressed, KeyBinding},
    log::{handle_redo_over, handle_undo, init_log, on_group_close, on_group_open, on_group_redo, on_group_undo, on_redo, on_redo_all, on_undo, RedoAllEvent, RedoKey, UndoKey},
    log_deserialize::deserialize_edits,
    log_serialize::serialize_edits,
    object::{NextObjectId, ObjectIdMap},
    piece::{
        clone::{on_clone_redo, on_clone_undo},
        create::{on_create, on_create_redo, on_create_undo},
        delete::{on_delete_redo, on_delete_undo},
        flip::{on_flip_redo, on_flip_undo},
        r#move::{on_move_redo, on_move_undo},
        rotate::{on_rotate_redo, on_rotate_undo},
    },
    view_adjust::{
        handle_pan_left, handle_pan_right, handle_pan_up, handle_pan_down, handle_pan_drag,
        handle_rotate_ccw, handle_rotate_cw,
        handle_zoom_reset, handle_zoom_in, handle_zoom_out, handle_zoom_scroll,
        KeyPanStep, KeyRotateStep, KeyScaleStep,
        PanLeftKey, PanRightKey, PanUpKey, PanDownKey,
        RotateCCWKey, RotateCWKey,
        ZoomInKey, ZoomOutKey, ZoomResetKey,
        WheelScaleStep
    },
    raise::RaiseAnchor,
    select::{clear_selection, draw_selection_rect, selection_rect_drag_start, selection_rect_drag, selection_rect_drag_end, Selected, SelectionRect, setup_selection_box, handle_key_selection},
    state::GameState,
    title::{SplashScreenTimer, display_title}
};

#[derive(Resource)]
pub struct GameBoxPath(pub PathBuf);

#[derive(Resource)]
pub struct LogPath(pub Option<PathBuf>);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);

// FIXME: unwrap
    let gamebox_path = GameBoxPath(PathBuf::from(args.next().unwrap()));

    let log_path = LogPath(args.next().map(PathBuf::from));

// FIXME: unwrap
    let base_path = gamebox_path.0
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    App::new()
        .insert_resource(gamebox_path)
        .insert_resource(log_path)
        .register_asset_source(
            base_path.clone(),
            AssetSourceBuilder::platform_default(&base_path, None)
        )
        .add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "V4 Bevy Demo".into(),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    anisotropy_clamp: 16,
//                    mag_filter: ImageFilterMode::Nearest,
                    ..ImageSamplerDescriptor::linear()
                }
            }),
            MeshPickingPlugin
        ))
        .init_state::<GameState>()
        .add_plugins((
            splash_plugin,
            game_plugin
        ))
        .run();

    Ok(())
}

fn splash_plugin(app: &mut App) {
    app
        .add_systems(
            OnEnter(GameState::Splash),
            (
                display_title,
                load_config,
                load_input_settings
                    .after(load_config),
                setup_selection_box,
                setup_game_resources,
                load_assets,
                init_log,
                deserialize_edits
                    .after(load_assets)
                    .after(init_log)
            )
        )
        .add_systems(
            Update,
            (
                mark_images_loaded,
                switch_to_game
            ).run_if(in_state(GameState::Splash))
        );
}

fn switch_to_game(
    mut next: ResMut<NextState<GameState>>,
    loading_handles: Res<LoadingHandles>,
    mut timer: ResMut<SplashScreenTimer>,
    time: Res<Time>
) {
    if timer.0.tick(time.delta()).is_finished() && loading_handles.0.is_empty()
    {
        next.set(GameState::Game);
    }
}

fn load_input_settings(
    config: Res<Config>,
    mut commands: Commands
)
{
    let steps = &config.steps;
    let keys = config.keys.clone();

    commands.insert_resource(KeyPanStep(steps.pan_step));
    commands.insert_resource(KeyRotateStep(steps.rotate_step.to_radians()));
    commands.insert_resource(KeyScaleStep(steps.key_scale_step));
    commands.insert_resource(WheelScaleStep(steps.wheel_scale_step));

    commands.insert_resource(PanLeftKey(keys.pan_left));
    commands.insert_resource(PanRightKey(keys.pan_right));
    commands.insert_resource(PanUpKey(keys.pan_up));
    commands.insert_resource(PanDownKey(keys.pan_down));

    commands.insert_resource(ZoomInKey(keys.zoom_in));
    commands.insert_resource(ZoomOutKey(keys.zoom_out));
    commands.insert_resource(ZoomResetKey(keys.zoom_reset));

    commands.insert_resource(RotateCCWKey(keys.rotate_ccw));
    commands.insert_resource(RotateCWKey(keys.rotate_cw));

    commands.insert_resource(UndoKey(keys.undo));
    commands.insert_resource(RedoKey(keys.redo));
}

fn setup_game_resources(mut commands: Commands) {
    commands.insert_resource(ObjectIdMap::default());
    commands.insert_resource(NextObjectId::default());
}

// TODO: check that there is no selection for view keys

fn game_plugin(app: &mut App) {
    app
        .add_systems(
            OnEnter(GameState::Game),
            display_game
        )
        .init_state::<ContextMenuState>()
        .add_systems(
            Update,
            (
                handle_pan_left.run_if(cfg_input_pressed::<PanLeftKey>),
                handle_pan_right.run_if(cfg_input_pressed::<PanRightKey>),
                handle_pan_up.run_if(cfg_input_pressed::<PanUpKey>),
                handle_pan_down.run_if(cfg_input_pressed::<PanDownKey>),

                handle_rotate_ccw.run_if(cfg_input_pressed::<RotateCCWKey>),
                handle_rotate_cw.run_if(cfg_input_pressed::<RotateCWKey>),

                handle_zoom_in.run_if(cfg_input_pressed::<ZoomInKey>),
                handle_zoom_out.run_if(cfg_input_pressed::<ZoomOutKey>),
                handle_zoom_reset.run_if(cfg_input_just_pressed::<ZoomResetKey>),
                handle_zoom_scroll.run_if(
                    resource_changed::<AccumulatedMouseScroll>.and(
                        not(resource_equals(AccumulatedMouseScroll::default()))
                    )
                ),

                draw_selection_rect.run_if(
                    resource_exists::<SelectionRect>
                        .and(|r: Res<SelectionRect>| r.active)
                ),

                trigger_close_context_menus_wheel.run_if(
                    in_state(ContextMenuState::Open).and(
                        resource_changed::<AccumulatedMouseScroll>.and(
                            not(resource_equals(AccumulatedMouseScroll::default()))
                        )
                    )
                ),

                trigger_close_context_menus_key.run_if(
                    in_state(ContextMenuState::Open).and(
                        resource_changed::<ButtonInput<KeyCode>>
                    )
                ),

                handle_key_selection.run_if(any_with_component::<Selected>),

                handle_undo.run_if(cfg_input_just_pressed::<UndoKey>),
                handle_redo_over.run_if(cfg_input_just_pressed::<RedoKey>),

                serialize_edits.run_if(input_just_pressed(KeyCode::KeyL))
            )
            .run_if(in_state(GameState::Game))
        )
        .add_observer(open_context_menu)
        .add_observer(close_context_menus)
        .add_observer(on_undo)
        .add_observer(on_redo)
        .add_observer(on_redo_all)
        .add_observer(surface::create::on_create)
        .add_observer(surface::create::on_create_undo)
        .add_observer(surface::create::on_create_redo)
        .add_observer(map::create::on_create)
        .add_observer(map::create::on_create_undo)
        .add_observer(map::create::on_create_redo)
        .add_observer(grid::create::on_create)
        .add_observer(grid::create::on_create_undo)
        .add_observer(grid::create::on_create_redo)
        .add_observer(on_clone_undo)
        .add_observer(on_clone_redo)
        .add_observer(on_create)
        .add_observer(on_create_undo)
        .add_observer(on_create_redo)
        .add_observer(on_delete_undo)
        .add_observer(on_delete_redo)
        .add_observer(on_flip_undo)
        .add_observer(on_flip_redo)
        .add_observer(on_group_open)
        .add_observer(on_group_close)
        .add_observer(on_group_undo)
        .add_observer(on_group_redo)
        .add_observer(on_move_undo)
        .add_observer(on_move_redo)
        .add_observer(on_rotate_undo)
        .add_observer(on_rotate_redo)
        .add_observer(dump_edits)
        .add_observer(pick_dbg);
}

fn display_game(
    window: Single<Entity, With<Window>>,
    mut commands: Commands
) -> Result
{
    commands.entity(*window)
        .observe(handle_pan_drag)
        .observe(clear_selection)
        .observe(selection_rect_drag_start)
        .observe(selection_rect_drag)
        .observe(selection_rect_drag_end)
        .observe(trigger_close_context_menus_press);

    commands.insert_resource(RaiseAnchor::default());
    commands.insert_resource(SelectionRect::default());

    commands.trigger(RedoAllEvent);

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

/*
Drag lag: im suspecting that the drag event is either being fired less often than you think, or the system itself is on a wrong schedule

TODO: try turning off vsync to fix drag lag
*/

// TODO: load svg
// TODO: load pdf
// TODO: load avif
// TODO: grid
// TODO: stacking

// TODO: make states for lasso select, piece drag, etc
//
