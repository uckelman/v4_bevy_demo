use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup, Update},
    asset::{
        AssetApp, Assets,
        io::AssetSourceBuilder
    },
    ecs::{
        bundle::Bundle,
        change_detection::{Res, ResMut},
        component::Component,
        entity::Entity,
        event::EntityEvent,
        error::Result,
        name::Name,
        observer::On,
        prelude::{Commands, not, Query, resource_changed, resource_equals, resource_exists, Single, SystemCondition, With}
    },
    image::{ImagePlugin, ImageSamplerDescriptor},
    input::{
        ButtonInput,
        common_conditions::input_pressed,
        keyboard::KeyCode,
        mouse::AccumulatedMouseScroll
    },
    math::Vec3,
    mesh::Mesh,
    picking::{
        Pickable,
        events::{Pointer, Click},
        mesh_picking::MeshPickingPlugin
    },
    prelude::{AppExtStates, Color, ColorMaterial, DespawnOnExit, IntoScheduleConfigs, in_state, NextState, OnEnter, Resource, Sprite, Time, trace, Transform, Window, WindowPlugin},
    sprite::Anchor,
};
use rand::Rng;
use std::path::Path;

mod actions;
mod assets;
mod config;
mod context_menu;
mod drag;
mod flip;
mod gamebox;
mod grid;
mod view;
mod view_adjust;
mod raise;
mod select;
mod state;
mod title;
mod util;

use crate::{
    actions::{add_action_observer},
    assets::{ImageSource, LoadingHandles, SpriteHandles, load_assets, mark_images_loaded},
    config::KeyConfig,
    context_menu::{ContextMenuState, open_context_menu, close_context_menus, trigger_close_context_menus_press, trigger_close_context_menus_wheel},
    drag::{Draggable, on_piece_drag_start, on_piece_drag, on_piece_drag_end},
    flip::{FlipForwardKey, FlipBackKey, handle_flip_forward, handle_flip_back},
    gamebox::{GameBox, MapDefinition, PieceType, SurfaceItem},
    grid::spawn_grid,
    view::handle_piece_pressed,
    view_adjust::{
        handle_pan_left, handle_pan_right, handle_pan_up, handle_pan_down, handle_pan_drag,
        handle_rotate_ccw, handle_rotate_cw,
        handle_zoom_reset, handle_zoom_in, handle_zoom_out, handle_zoom_scroll,
        KeyPanStep, KeyRotateStep, KeyScaleStep,
        PanLeftKey, PanRightKey, PanUpKey, PanDownKey,
        RotateCCWKey, RotateCWKey,
        ZoomInKey, ZoomOutKey,
        WheelScaleStep
    },
    raise::RaiseAnchor,
    select::{clear_selection, draw_selection_rect, on_selection, on_deselection, selection_rect_drag_start, selection_rect_drag, selection_rect_drag_end, Selectable, SelectEvent, DeselectEvent, SelectionRect, setup_selection_box},
    state::GameState,
    title::{SplashScreenTimer, display_title}
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();

// FIXME: unwrap
    let base = Path::new(&args[1])
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    App::new()
        .register_asset_source(
            base.clone(),
            AssetSourceBuilder::platform_default(&base, None)
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
                load_assets,
                load_input_settings,
                setup_selection_box
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

    commands.insert_resource(FlipBackKey(KeyCode::BracketLeft));
    commands.insert_resource(FlipForwardKey(KeyCode::BracketRight));
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

fn cfg_input_just_pressed<T>(
    key: Res<T>,
    inputs: Res<ButtonInput<KeyCode>>
) -> bool
where
    T: Resource + KeyConfig
{
    inputs.just_pressed(key.code())
}

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
                ),

                trigger_close_context_menus_wheel.run_if(
                    in_state(ContextMenuState::Open).and(
                        resource_changed::<AccumulatedMouseScroll>.and(
                            not(resource_equals(AccumulatedMouseScroll::default()))
                        )
                    )
                ),

                handle_flip_forward.run_if(cfg_input_just_pressed::<FlipForwardKey>),
                handle_flip_back.run_if(cfg_input_just_pressed::<FlipBackKey>)
            )
            .run_if(in_state(GameState::Game))
        )
        .add_observer(open_context_menu)
        .add_observer(close_context_menus)
        .add_observer(pick_dbg);
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
    name: Name,
    sprite: Sprite,
    anchor: Anchor,
    transform: Transform
}

#[derive(Component, Default)]
struct Piece;

// TODO: should this reference a piece type?
#[derive(Component, Default)]
struct Faces(Vec<ImageSource>);

// TODO: should this be a cyclic iterator?
#[derive(Component, Default)]
struct FaceUp(usize);

#[derive(Component, Default)]
pub struct Actions(pub Vec<String>);

#[derive(Bundle, Default)]
struct PieceBundle {
    marker: Piece,
    name: Name,
    pickable: Pickable,
    selectable: Selectable,
    draggable: Draggable,
    sprite: Sprite,
    transform: Transform,
    faces: Faces,
    up: FaceUp,
    actions: Actions
}

fn spawn_map(
    m: &MapDefinition,
    mut t: Transform,
    sprite_handles: &Res<SpriteHandles>,
    commands: &mut Commands
)
{
    let Some(src) = sprite_handles.0.get(&m.image) else { return; };
    
    let sprite = match src {
        ImageSource::Single(handle) => Sprite::from_image(handle.clone()),
        _ => todo!()
    };

    t.translation += Vec3::new(m.x, m.y, 0.0);

    trace!("map {}", t.translation.z);

    commands.spawn((
        MapBundle {
            sprite,
            anchor: m.anchor.into(),
            transform: t,
            ..Default::default()
        },
        DespawnOnExit(GameState::Game)
    ));
}

fn spawn_piece(
    p: &PieceType,
    mut t: Transform,
    sprite_handles: &Res<SpriteHandles>,
    commands: &mut Commands
)
{
// FIXME: should fail if we can't get a sprite?
    let faces = p.faces.iter()
        .filter_map(|f| sprite_handles.0.get(f))
        .cloned()
        .collect::<Vec<_>>();

    let sprite = match &faces[0] {
        ImageSource::Single(handle) => Sprite::from_image(handle.clone()),
        ImageSource::Crop { handle, atlas } => Sprite::from_atlas_image(
            handle.clone(),
            atlas.clone()
        )
    };

    trace!("piece {}", t.translation.z);

    let mut ec = commands.spawn((
        PieceBundle {
            name: Name::from(p.name.as_ref()),
            sprite,
            transform: t,
            faces: Faces(faces),
            up: FaceUp(0),
            actions: Actions(p.actions.clone()),
            ..Default::default()
        },
        DespawnOnExit(GameState::Game)
    ));

    ec
        .observe(recolor_on::<SelectEvent>(Color::hsl(0.0, 0.9, 0.7)))
        .observe(recolor_on::<DeselectEvent>(Color::WHITE))
        .observe(handle_piece_pressed)
        .observe(raise::on_piece_pressed)
        .observe(raise::on_piece_released)
        .observe(on_piece_drag_start)
        .observe(on_piece_drag)
        .observe(on_piece_drag_end)
    //    .observe(on_piece_drop)
        .observe(on_selection)
        .observe(on_deselection);

    for a in &p.actions {
        add_action_observer(a, &mut ec);
    }
}

fn pick_dbg(ev: On<Pointer<Click>>, names: Query<&Name>) {
    let name = names
        .get(ev.event_target())
        .map(|n| n.to_string())
        .unwrap_or("Unknown".to_string());

    trace!("Picked {name}({:?})", ev.event_target());
}

fn display_game(
    mut commands: Commands,
    window: Single<Entity, With<Window>>,
    sprite_handles: Res<SpriteHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    gamebox: Res<GameBox>,
) -> Result
{
    commands.entity(*window)
        .observe(handle_pan_drag)
        .observe(clear_selection)
        .observe(selection_rect_drag_start)
        .observe(selection_rect_drag)
        .observe(selection_rect_drag_end)
        .observe(trigger_close_context_menus_press);

    let mut surface = Surface { max_z: 0.0 };

    // create surface

    let mut stack = vec![(&gamebox.surface, Transform::IDENTITY)];

    loop {
        let Some((st, t)) = stack.pop() else { break; };

        match st {
            SurfaceItem::Map(m) => spawn_map(m, t, &sprite_handles, &mut commands),
            SurfaceItem::Grid(g) => spawn_grid(g, t, &mut meshes, &mut materials, &mut commands),
            SurfaceItem::Group(g) => {
                let mut t = t;
                t.translation += Vec3::new(g.x, g.y, 0.0);
                let mut z = t.translation.z;

                stack.extend(
                    g.children
                        .iter()
                        .map(|ch| {
                            let mut t = t;
                            z -= 1.0;
                            t.translation.z = z;
                            (ch, t)
                        })
                );
            }
        }
    }

    // create pieces
    let mut rng = rand::rng();

    for p in &gamebox.piece {
        let x = rng.random_range(-500.0..=500.0);
        let y = rng.random_range(-500.0..=500.0);

        surface.max_z += 1.0;

        let t = Transform::from_xyz(x, y, surface.max_z);

        spawn_piece(p, t, &sprite_handles, &mut commands);
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

fn recolor_on<E: EntityEvent>(
    color: Color
) -> impl Fn(On<E>, Query<&mut Sprite>)
{
    move |ev, mut sprites| {
        if let Ok(mut sprite) = sprites.get_mut(ev.event().event_target()) {
            sprite.color = color;
        }
    }
}

// TODO: try turning off vsync to fix drag lag

/*
Drag lag: im suspecting that the drag event is either being fired less often than you think, or the system itself is on a wrong schedule
*/

// TODO: load svg
// TODO: load pdf
// TODO: load avif
// TODO: log
// TODO: undo/redo
// TODO: grid
// TODO: stacking

// Actions
// TODO: delete piece
// TODO: clone piece
// TODO: rotate piece
// TODO: context menu text
// TODO: context menu hotkeys


// TODO: make states for lasso select, piece drag, etc
