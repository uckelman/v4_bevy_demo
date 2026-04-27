use bevy::{
    asset::Assets,
    camera::Camera,
    ecs::{
        change_detection::ResMut,
        component::Component,
        entity::Entity,
        error::Result,
        name::Name,
        observer::On,
        prelude::{ChildOf, Commands, Query, Single, With, Without}
    },
    math::{
        Vec2, Vec3,
        prelude::{ConvexPolygon, Polyline2d, Rectangle}
    },
    mesh::{Mesh, Mesh2d},
    picking::{
        Pickable,
        events::{Over, Out, Pointer, Move}
    },
    prelude::{Color, ColorMaterial, debug, EntityEvent, GlobalTransform, MeshMaterial2d, trace, Transform, Visibility}
};
use tracing::{enabled, instrument, Level};

use crate::{
    drag::handle_drop,
    gamebox::{Anchor, ColumnStagger, GridDefinition, HexGridDefinition, RectGridDefinition},
    object::ObjectId
};

pub mod create;

#[derive(Component, Default)]
struct RectGrid;

#[derive(Component, Default)]
struct HexGrid;

#[derive(Component, Default)]
pub struct HexGridCell;

#[derive(Component, Default)]
struct RectGridParams {
    x: f32,
    y: f32,
    cols: u32,
    rows: u32,
    cw: f32,
    rh: f32
}

fn spawn_rect_grid(
    def: &RectGridDefinition,
    mut t: Transform,
    parent: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    commands: &mut Commands
)
{
    let RectGridDefinition { x, y, anchor, cols, rows, cw, rh, .. } = def;

    let grect = Rectangle::new(*cols as f32 * cw, *rows as f32 * rh);

    // center of the grid is at x,y
    t.translation += Vec3::new(*x, *y, 0.0);
    // apply the anchor
    t.translation += match anchor {
        Anchor::BottomLeft => grect.half_size,
        Anchor::BottomCenter => grect.half_size.with_x(0.0),
        Anchor::BottomRight => grect.half_size * Vec2::new(-1.0, 1.0),
        Anchor::CenterLeft => grect.half_size.with_y(0.0),
        Anchor::Center => Vec2::ZERO,
        Anchor::CenterRight => -grect.half_size.with_y(0.0),
        Anchor::TopLeft => grect.half_size * Vec2::new(1.0, -1.0),
        Anchor::TopCenter => -grect.half_size.with_x(0.0),
        Anchor::TopRight => -grect.half_size
    }.extend(0.0);

    let tg = t;

    // center of cell 0,0 is half a grid from the center of the grid
    t.translation -= grect.half_size.extend(0.0);
    // cell 0,0 should have its lower-left corner at the grid origin
    t.translation += Vec3::new(cw / 2.0, rh / 2.0, 0.0);

    let rect = Rectangle::new(*cw, *rh);

    let omesh = meshes.add(Polyline2d::new([
       Vec2::new(-rect.half_size.x, rect.half_size.y),
       Vec2::new(rect.half_size.x, rect.half_size.y),
       Vec2::new(rect.half_size.x, -rect.half_size.y),
       Vec2::new(-rect.half_size.x, -rect.half_size.y),
       Vec2::new(-rect.half_size.x, rect.half_size.y)
    ]));

    let rmesh = meshes.add(rect);

    let grid_color = Color::srgb(0.8, 0.8, 0.8);
    let highlight_color = Color::srgba(1.0, 0.0, 0.0, 0.3);
    let unhighlight_color = Color::NONE;

    let grid_material = materials.add(grid_color);
    let highlight_material = materials.add(highlight_color);
    let unhighlight_material = materials.add(unhighlight_color);

    let gmesh = meshes.add(grect);
    commands.spawn((
        Mesh2d(gmesh),
        MeshMaterial2d(materials.add(Color::srgba(0.0, 1.0, 0.0, 0.2))),
        Pickable::default(),
        tg,
        ChildOf(parent)
    ));

    for r in 0..*rows {
        for c in 0..*cols {
            let mut ct = t;
            ct.translation += Vec3::new(c as f32 * cw, r as f32 * rh, 0.0);

            commands
                .spawn((
                    Mesh2d(rmesh.clone()),
                    MeshMaterial2d(unhighlight_material.clone()),
                    Pickable::default(),
                    ct,
                    ChildOf(parent)
                ))
                .observe(recolor_cell_on::<Pointer<Over>>(highlight_color))
                .observe(recolor_cell_on::<Pointer<Out>>(unhighlight_color));

            commands
                .spawn((
                    Mesh2d(omesh.clone()),
                    MeshMaterial2d(grid_material.clone()),
                    Pickable::default(),
                    ct,
                    ChildOf(parent)
                ));
        }
    }
}

fn anchor_to_vec3(rect: Rectangle, anchor: Anchor) -> Vec3 {
    match anchor {
        Anchor::BottomLeft => rect.half_size,
        Anchor::BottomCenter => rect.half_size.with_x(0.0),
        Anchor::BottomRight => rect.half_size * Vec2::new(-1.0, 1.0),
        Anchor::CenterLeft => rect.half_size.with_y(0.0),
        Anchor::Center => Vec2::ZERO,
        Anchor::CenterRight => -rect.half_size.with_y(0.0),
        Anchor::TopLeft => rect.half_size * Vec2::new(1.0, -1.0),
        Anchor::TopCenter => -rect.half_size.with_x(0.0),
        Anchor::TopRight => -rect.half_size
    }.extend(0.0)
}

fn spawn_hex_grid(
    def: &HexGridDefinition,
    mut t: Transform,
    parent: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    commands: &mut Commands
)
{
    let HexGridDefinition {
        id,
        x,
        y,
        anchor,
        cols,
        rows,
        hw,
        hh,
        hs,
        first,
        ..
    } = def;

// FIXME: bb is slightly too tall; why?
    // grid bounding box
    let grect = Rectangle::new(
        *cols as f32 * hw * 0.75 + (hw * 0.25),
        *rows as f32 * (hh + 0.5) + (hh * 0.5)
    );

    // anchors don't apply to meshes presently; apply the anchor manually
    t.translation += anchor_to_vec3(grect, *anchor);

    // this is the translation for the grid bounding box
    let gt = t;

    // center of cell 0,0 is half a grid from the center of the grid
    t.translation -= grect.size().extend(0.0);

    // move center of cell 0,0 be be inside the bounding box
    t.translation += Vec3::new(hw / 4.0, hh / 2.0, 0.0);

/*
    use std::f32::consts::FRAC_PI_6;

    let SQRT_3 = 3.0_f32.sqrt();
    let w = 2.0 * hs;
    let h = SQRT_3 * hs;

    let hex = ConvexPolygon::new([
        Vec2::new(-hs, 0.0),
        Vec2::new(-hs / 2.0, SQRT_3 / 2.0 * hs),
        Vec2::new(hs / 2.0, SQRT_3 / 2.0 * hs),
        Vec2::new(*hs, 0.0),
        Vec2::new(hs / 2.0, -SQRT_3 / 2.0 * hs),
        Vec2::new(-hs / 2.0, -SQRT_3 / 2.0 * hs)
    ]).unwrap();

    let v = hex.vertices(FRAC_PI_6)
        .into_iter()
        .chain(hex.vertices(FRAC_PI_6).into_iter().next());

    let omesh = meshes.add(Polyline2d::new(v));
*/

    let hex = ConvexPolygon::new([
        Vec2::new(-hw / 2.0, 0.0),
        Vec2::new(-hw / 4.0, hh / 2.0),
        Vec2::new(hw / 4.0, hh / 2.0),
        Vec2::new(hw / 2.0, 0.0),
        Vec2::new(hw / 4.0, -hh / 2.0),
        Vec2::new(-hw / 4.0, -hh / 2.0)
    ]).unwrap();

    let omesh = meshes.add(Polyline2d::new(
        hex.vertices()
            .iter()
            .cloned()
            .chain(hex.vertices()
                .first()
                .cloned()
            )
    ));

    let cmesh = meshes.add(hex);

    let grid_color = Color::srgb(0.0, 0.0, 1.0);
    let highlight_color = Color::srgba(1.0, 0.0, 0.0, 0.3);
    let unhighlight_color = Color::NONE;

    let grid_material = materials.add(grid_color);
    let highlight_material = materials.add(highlight_color);
    let unhighlight_material = materials.add(unhighlight_color);

    let gid = if enabled!(Level::DEBUG) {
        // render the bounding box
        let gmesh = meshes.add(grect);
        commands.spawn((
            ObjectId(*id),
            Mesh2d(gmesh),
            MeshMaterial2d(materials.add(Color::srgba(0.0, 1.0, 0.0, 0.2))),
            ChildOf(parent),
            gt,
            Pickable::IGNORE,
            Visibility::Inherited
        ))
    }
    else {
        // don't render the bounding box
        commands.spawn((
            ObjectId(*id),
            ChildOf(parent),
            gt,
            Pickable::IGNORE,
            Visibility::Inherited
        ))
    }.id();

    let stagger = match first {
        ColumnStagger::Low => 1,
        ColumnStagger::High => 0
    };

    for r in 0..*rows {
        for c in 0..*cols {

            let mut ct = t;
            ct.translation += Vec3::new(
                c as f32 * 0.75 * hw,
                r as f32 * hh - ((c + stagger) % 2) as f32 * hh / 2.0,
                0.0
            );

            // the cell
            commands
                .spawn((
                    HexGridCell,
                    ObjectId(id + 1 + r * cols + c),
                    Mesh2d(cmesh.clone()),
                    MeshMaterial2d(unhighlight_material.clone()),
                    ChildOf(gid),
                    ct,
                    Pickable::default(),
                    Visibility::Inherited
                ))
                .observe(recolor_cell_on::<Pointer<Over>>(highlight_color))
                .observe(recolor_cell_on::<Pointer<Out>>(unhighlight_color))
                .observe(handle_drop);

            // the outline
            commands.spawn((
                Mesh2d(omesh.clone()),
                MeshMaterial2d(grid_material.clone()),
                ChildOf(gid),
                ct,
                Pickable::IGNORE,
                Visibility::Inherited
            ));
        }
    }
}

pub fn spawn_grid(
    g: &GridDefinition,
    parent: Entity,
    t: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    commands: &mut Commands
)
{
    match g {
        GridDefinition::Rect(def) =>
            spawn_rect_grid(def, t, parent, meshes, materials, commands),
        GridDefinition::Hex(def) =>
            spawn_hex_grid(def, t, parent, meshes, materials, commands)
    }
}

fn recolor_cell_on<E: EntityEvent>(
    color: Color
) -> impl Fn(
    On<E>,
    Query<Entity, With<Mesh2d>>,
    ResMut<Assets<ColorMaterial>>,
    Commands
)
{
    move |ev, entities, mut materials, mut commands| {
        trace!("");
        if let Ok(entity) = entities.get(ev.event().event_target()) {
            commands
                .entity(entity)
                .insert(MeshMaterial2d(materials.add(color)));
        }
    }
}

/*
#[instrument(skip_all)]
pub fn handle_over_grid(
    mut over: On<Pointer<Move>>,
    grid_query: Query<&RectGridParams, With<RectGrid>>,
    query: Single<(&Camera, &GlobalTransform)>
) -> Result
{
    trace!("");

    if let Some(pos) = over.hit.position {
        let entity = over.event().event_target();
        let grid = grid_query.get(entity)?;

        let (camera, global_transform) = *query;
        let pos = camera.viewport_to_world_2d(global_transform, over.pointer_location.position)?;

        trace!("{},{}", (pos.x / grid.cw).floor(), (pos.y / grid.rh).floor());
    }

    Ok(())
}
*/

/*
#[instrument(skip_all)]
pub fn on_piece_drop(
    mut drop: On<Pointer<DragDrop>>,
    mut piece_query: Query<&mut Transform>,
    grid_query: Query<&RectGridParams, With<RectGrid>>,
    query: Single<(&Camera, &GlobalTransform)>
) -> Result
{
    debug!("");

    if let Some(pos) = drop.hit.position {
        let dst = drop.event().event_target();
        let src = drop.event().dropped;

        debug!("{} {}", src, dst);

        let grid = grid_query.get(dst)?;
        let mut src_transform = piece_query.get_mut(src)?;

        let (camera, global_transform) = *query;
        let pos = camera.viewport_to_world_2d(global_transform, drop.pointer_location.position)?;

        debug!("{},{} {},{}", pos.x, pos.y, (pos.x / grid.cw).floor(), (pos.y / grid.rh).floor());

        // snap piece to center
        let cx = ((pos.x / grid.cw).floor() + 0.5) * grid.cw;
        let cy = ((pos.y / grid.rh).floor() + 0.5) * grid.rh;

        src_transform.translation.x = cx;
        src_transform.translation.y = cy;
    }

    Ok(())
}
*/

/*
fn world_to_grid(v: Vec2) -> Vec2 {


}
*/
