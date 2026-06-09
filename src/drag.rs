use bevy::{
    asset::Assets,
    camera::Camera,
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        error::Result,
        name::Name,
        observer::On,
        prelude::{ChildOf, Commands, Entity, Or, Query, Resource, With, Without}
    },
    image::Image,
    input::{
        ButtonInput,
        keyboard::KeyCode
    },
    math::{Dir3, Ray3d, Rect, Vec2, Vec3},
    picking::{
        Pickable,
        events::{Drag, DragDrop, DragEnd, DragStart, Pointer},
        mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
        pointer::PointerButton
    },
    prelude::{Color, GlobalTransform, Projection, Sprite, State, trace, Transform},
    sprite::Anchor
};
use tracing::instrument;

use crate::{
    context_menu::ContextMenuState,
    grid::{HexGridCell, RectGridCell},
    keys::{ctrl_pressed, shift_pressed},
    log::{OpenGroupEvent, CloseGroupEvent},
    maxz::MaxZ,
    piece::{
        Above, Below, Location, StackingGroup,
        r#move::DoMoveEvent,
        splice::DoSpliceEvent
    },
    select::Selected,
    stack::{Expanded, StackAboveQueryExt, StackBelowQueryExt},
    util::AsOrthographicProjection
};

#[derive(Clone, Component, Copy, Debug,  Default)]
pub struct Draggable;

#[derive(Default, Resource)]
pub struct DragOrigin(pub Vec2);

#[derive(Clone, Component, Copy, Debug)]
pub struct GhostStart(pub Vec3);

#[derive(Clone, Component, Copy, Debug,  Default)]
pub struct DragGhost;

#[instrument(skip_all)]
pub fn on_piece_drag_start(
    mut drag: On<Pointer<DragStart>>,
    modifiers: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &GlobalTransform, &Sprite), (With<Draggable>, With<Selected>)>,
    parent_query: Query<&ChildOf>,
    maxz_query: Query<&MaxZ>,
    mut drag_origin: ResMut<DragOrigin>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut commands: Commands
) -> Result
{
    trace!("");

    // prevent the event from bubbling up to the parent
    drag.propagate(false);

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    if *context_menu_state == ContextMenuState::Open {
        return Ok(());
    }

    if ctrl_pressed(&modifiers) || shift_pressed(&modifiers) {
        return Ok(());
    }

    // set the drag anchor to the start of the drag
    drag_origin.0 = drag.hit.position.unwrap().truncate();

    // selection must be on one surface; get it from the first selected item
    let Some(first) = query.iter().next() else { return Ok(()); };
    let root = parent_query.root_ancestor(first.0);

    let max_z = maxz_query.get(root)?;

    let color = Color::srgba(1.0, 1.0, 1.0, 0.5);

    for (_, gt, sprite) in query {
        let mut ghost_sprite = sprite.clone();
        ghost_sprite.color = color;

        let mut t = gt.compute_transform();
        t.translation.z += max_z.0;

        commands.spawn((
            DragGhost,
            ghost_sprite,
            t,
            GhostStart(t.translation),
            Pickable::IGNORE
        ));
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag(
    mut drag: On<Pointer<Drag>>,
    mut g_query: Query<(&mut Transform, &GhostStart), (With<DragGhost>, Without<Camera>)>,
    tp_query: Query<(&Transform, &Projection), With<Camera>>,
    context_menu_state: Res<State<ContextMenuState>>
) -> Result
{
    trace!("");

    // prevent the event from bubbling up to the parent
    drag.propagate(false);

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    if *context_menu_state == ContextMenuState::Open {
        return Ok(());
    }

    let (camera_transform, camera_projection) = tp_query.single()?;
    let camera_projection = camera_projection.as_ortho()?;

    let mut drag_dist = drag.distance.extend(0.0);
    drag_dist.y = -drag_dist.y;

    // apply current scale to the drag
    drag_dist *= camera_projection.scale;

    // apply current rotation to the drag
    drag_dist = camera_transform.rotation * drag_dist;

    g_query.iter_mut().for_each(|(mut transform, start)| {
        transform.translation = start.0 + drag_dist;
    });

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drag_end(
    mut drag: On<Pointer<DragEnd>>,
    g_query: Query<Entity, With<DragGhost>>,
    context_menu_state: Res<State<ContextMenuState>>,
    mut commands: Commands
) -> Result
{
    trace!("");

    // prevent the event from bubbling up to the parent
    drag.propagate(false);

    if *context_menu_state == ContextMenuState::Open {
        return Ok(());
    }

    if drag.button != PointerButton::Primary {
        return Ok(());
    }

    for g in g_query {
        commands.entity(g).despawn();
    }

    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum DropTargetType {
    Surface,
    Piece,
    Grid
}

fn find_hit(
    esrc: Entity,
    parent: Entity,
    src_gt: &GlobalTransform,
    src_t: &Transform,
    drag_dist: Vec3,
    max_z: f32,
    dz: f32,
    bboxes: &[(Entity, f32, Entity, Rect)],
    ray_cast: &mut MeshRayCast,
    mrcs: &MeshRayCastSettings,
    root: Entity,
    gt_query: Query<&GlobalTransform>,
    sg_query: Query<&StackingGroup>,
) -> Result<(Entity, Vec3)>
{
    let b_drop_pos = (src_gt.translation() + drag_dist).truncate();

    // find piece hits
    let pieces = bboxes.iter()
        .filter(|(_, _, base, bb)| esrc != *base && bb.contains(b_drop_pos))
        .map(|(e, z, ..)| (*e, *z, DropTargetType::Piece));

    // find grid cell hits
    let ray = Ray3d::new(b_drop_pos.extend(max_z + 1.0), Dir3::NEG_Z);
    let cells = ray_cast.cast_ray(ray, mrcs)
        .iter()
        .map(|(e, h)| (*e, h.point.z, DropTargetType::Grid));

    // surface is always a hit
    let surf = (root, f32::NEG_INFINITY, DropTargetType::Surface);

    // find top hit
    let (ehit, htype) = std::iter::once(surf)
        .chain(pieces)
        .chain(cells)
        .max_by(|(_, za, _), (_, zb, _)| za.partial_cmp(zb).expect("NaN"))
        .map(|(e, _, t)| (e, t))
        .expect("Surface will be hit if nothing else is");

    let dst_t = match htype {
        DropTargetType::Surface => {
            if parent == ehit {
                *src_t
            }
            else {
                let dst_gt = gt_query.get(ehit)?;
                src_gt.reparented_to(dst_gt)
            }.translation + drag_dist
        },
        DropTargetType::Piece => {
            let src_sg = sg_query.get(esrc)?;
            let dst_sg = sg_query.get(ehit)?;

            if src_sg == dst_sg {
                // stack src onto dst if they are in the same stacking group
                // give src a stacking offset
                Vec3::new(2.0, 2.0, 1.0)
            }
            else {
                // otherwise keep the same global transform if reparenting
                let dst_gt = gt_query.get(ehit)?;
                src_gt.reparented_to(dst_gt).translation + drag_dist
            }
        },
        DropTargetType::Grid => {
            // snap piece to center of grid cell
            Vec3::new(0.0, 0.0, dz)
        }
    };

    Ok((ehit, dst_t))
}

#[instrument(skip_all)]
pub fn handle_drop(
    mut drop: On<Pointer<DragDrop>>,
    selection_query: Query<(Entity, &Above, &GlobalTransform, &Transform, &Location, Option<&Expanded>), (With<Draggable>, With<Selected>)>,
    a_query: Query<(Option<&Above>, &StackingGroup)>,
    d_query: Query<(Option<&Below>, &StackingGroup)>,
    parent_query: Query<&ChildOf>,
    mut maxz_query: Query<&mut MaxZ>,
    drag_origin: Res<DragOrigin>,
    gt_query: Query<&GlobalTransform>,
    sg_query: Query<&StackingGroup>,
    sprite_collision_query: Query<(Entity, &GlobalTransform, &Anchor, &Sprite, &Name)>,
    mut ray_cast: MeshRayCast,
    cell_query: Query<(), Or<(With<HexGridCell>, With<RectGridCell>)>>,
//    mesh_collision_query: Query<(Entity, &GlobalTransform)>,
    assets: Res<Assets<Image>>,
    mut commands: Commands
) -> Result
where
{
    trace!("");

    drop.propagate(false);

    // pieces from one stack may be dropped onto and stack with another stack
    // pieces from multiple stacks do not restack on drop

    // separate stack bottoms, expanded pieces;
    // compute lowest and highest points for the selection
    let mut bottoms = vec![];
    let mut expandeds = vec![];
    let mut sel_min_z = f32::INFINITY;
    let mut sel_max_z = f32::NEG_INFINITY;

    for (e, p, gt, t, l, exp) in selection_query {
        if exp.is_some() {
            expandeds.push((e, p.0, gt, t, l));
        }
        else if a_query.iter_below(e).next().is_none() {
            bottoms.push((e, p.0, gt, t, l));
        }

        let z = gt.translation().z;

        if z < sel_min_z {
            sel_min_z = z;
        }

        if z > sel_max_z {
            sel_max_z = z;
        }
    }

    let root = if let Some(p) = bottoms.first().or(expandeds.first()) {
        parent_query.root_ancestor(p.0)
    }
    else {
        // TODO: impossible?
        return Ok(());
    };

    // raise entire selection by amount the lowest member is below max z
    let mut max_z = maxz_query.get_mut(root)?;
    let dz = max_z.0 - sel_min_z + 1.0;
    max_z.0 = sel_max_z + dz;

    // get the drop position
    let drop_pos = drop.hit.position.unwrap().truncate();

    // find the drag vector
    let drag_dist = (drop_pos - drag_origin.0).extend(dz);

// TODO: check against sprite picking backend
// TODO: kdtree or quadtree?
    // collect bounding boxes for sprites
    let bboxes = sprite_collision_query.iter()
        .map(|(e, gt, a, s, n)| {
            let image_size = assets.get(&s.image).unwrap().size_f32();
            let scaled = image_size * gt.scale().truncate();
            let pos = gt.translation();
            let mut bbox = Rect::from_center_size(pos.truncate(), scaled);

            let ashift = match *a {
                Anchor::BOTTOM_LEFT => bbox.half_size(),
                Anchor::BOTTOM_CENTER => bbox.half_size().with_x(0.0),
                Anchor::BOTTOM_RIGHT => bbox.half_size() * Vec2::new(-1.0, 1.0),
                Anchor::CENTER_LEFT => bbox.half_size().with_y(0.0),
                Anchor::CENTER => Vec2::ZERO,
                Anchor::CENTER_RIGHT => -bbox.half_size().with_y(0.0),
                Anchor::TOP_LEFT => bbox.half_size() * Vec2::new(1.0, -1.0),
                Anchor::TOP_CENTER => -bbox.half_size().with_x(0.0),
                Anchor::TOP_RIGHT => -bbox.half_size(),
                _ => todo!()
            };

            bbox.min += ashift;
            bbox.max += ashift;

            let b = a_query.bottom(e);

            (e, pos.z, b, bbox)
        })
        .collect::<Vec<_>>();

    // set up ray caster
    let cell_filter = |entity| cell_query.contains(entity);

    let mrcs = MeshRayCastSettings::default()
        .with_filter(&cell_filter)
        .always_early_exit();

    if bottoms.len() + expandeds.len() > 1 {
        commands.trigger(OpenGroupEvent);
    }

    for (esrc, parent, src_gt, src_t, src_l) in &bottoms {
        let (ehit, dst_t) = find_hit(
            *esrc,
            *parent,
            src_gt,
            src_t,
            drag_dist,
            max_z.0,
            dz,
            &bboxes,
            &mut ray_cast,
            &mrcs,
            root,
            gt_query,
            sg_query
        )?;

        commands.trigger(DoMoveEvent {
            entity: *esrc,
            src_parent: *parent,
            src: src_l.0,
            dst_parent: ehit,
            dst: dst_t
        });
    }

    for (esrc, parent, src_gt, src_t, src_l) in &expandeds {
        let (ehit, dst_t) = find_hit(
            *esrc,
            *parent,
            src_gt,
            src_t,
            drag_dist,
            max_z.0,
            dz,
            &bboxes,
            &mut ray_cast,
            &mrcs,
            root,
            gt_query,
            sg_query
        )?;

        // no move if hit is self or parent in same stack
        if *esrc == ehit || (*parent == ehit && a_query.iter_below(ehit).next() == Some(*parent)){
            continue;
        }

        commands.entity(*esrc).remove::<Expanded>();

        let src_child = d_query.iter_above(*esrc).next();
        let dst_child = d_query.iter_above(ehit).next();

        commands.trigger(DoSpliceEvent {
            entity: *esrc,
            src_parent: *parent,
            src_child,
            src: src_l.0,
            dst_parent: ehit,
            dst_child,
            dst: dst_t
        });
    }

    if bottoms.len() + expandeds.len() > 1 {
        commands.trigger(CloseGroupEvent);
    }

    Ok(())
}
