use bevy::{
    asset::Assets,
    camera::Camera,
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        error::Result,
        name::Name,
        observer::On,
        prelude::{ChildOf, Commands, Entity, Query, Resource, With, Without}
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
    grid::HexGridCell,
    keys::{ctrl_pressed, shift_pressed},
    log::{OpenGroupEvent, CloseGroupEvent},
    maxz::MaxZ,
    piece::{
        StackingGroup,
        r#move::DoMoveEvent
    },
    raise::raise_piece,
    select::{deselect_all, select, Selected},
    stack::StackBelowQueryExt,
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

#[instrument(skip_all)]
pub fn handle_drop(
    mut drop: On<Pointer<DragDrop>>,
    selection_query: Query<(Entity, &ChildOf, &GlobalTransform, &Transform), (With<Draggable>, With<Selected>)>,
    a_query: Query<(Option<&ChildOf>, &StackingGroup)>,
    parent_query: Query<&ChildOf>,
    root_query: Query<&Name>,
    mut maxz_query: Query<&mut MaxZ>,
    drag_origin: Res<DragOrigin>,
    gt_query: Query<&GlobalTransform>,
    sg_query: Query<&StackingGroup>,
    sprite_collision_query: Query<(Entity, &GlobalTransform, &Anchor, &Sprite, &Name)>,
    mut ray_cast: MeshRayCast,
    hgc_query: Query<(), With<HexGridCell>>,
//    mesh_collision_query: Query<(Entity, &GlobalTransform)>,
    assets: Res<Assets<Image>>,
    mut commands: Commands
) -> Result
where
{
    trace!("");

    drop.propagate(false);

    // find all the stack bottoms, highest point for the selection
    let mut bottoms = vec![];
    let mut sel_min_z = f32::INFINITY;
    let mut sel_max_z = f32::NEG_INFINITY;

    for (e, p, gt, t) in selection_query {
        if a_query.iter_below(e).next().is_none() {
            bottoms.push((e, p.0, gt, t));
        }

        let z = gt.translation().z;

        if z < sel_min_z {
            sel_min_z = z;
        }

        if z > sel_max_z {
            sel_max_z = z;
        }
    }

    if bottoms.is_empty() {
        // TODO: impossible?
        return Ok(());
    }

    let root = parent_query.root_ancestor(bottoms[0].0);
    let root_name = root_query.get(root)?;

    // raise entire selection by amount the lowest member is below max z
    let mut max_z = maxz_query.get_mut(root)?;
    let dz = max_z.0 - sel_min_z + 1.0;
    max_z.0 = sel_max_z + dz;

    // get the drop position
    let drop_pos = drop.hit.position.unwrap().truncate();

    // find the drag vector
    let drag_dist = (drop_pos - drag_origin.0).extend(dz);

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
    let hex_filter = |entity| hgc_query.contains(entity);

    let mrcs = MeshRayCastSettings::default()
        .with_filter(&hex_filter)
        .always_early_exit();

    if bottoms.len() > 1 {
        commands.trigger(OpenGroupEvent);
    }

// TODO: maintain rotation on piece drop

    for (esrc, parent, src_gt, src_t) in &bottoms {

        let b_drop_pos = (src_gt.translation() + drag_dist).truncate();

        // find piece hits
        let pieces = bboxes.iter()
            .filter(|(_, _, base, bb)| esrc != base && bb.contains(b_drop_pos))
            .map(|(e, z, ..)| (*e, *z, DropTargetType::Piece));

        // find hex hits
        let ray = Ray3d::new(b_drop_pos.extend(max_z.0 + 1.0), Dir3::NEG_Z);
        let hexes = ray_cast.cast_ray(ray, &mrcs)
            .iter()
            .map(|(e, h)| (*e, h.point.z, DropTargetType::Grid));

        // surface is always a hit
        let surf = (root, f32::NEG_INFINITY, DropTargetType::Surface);

        // find top hit
        let (ehit, htype) = std::iter::once(surf)
            .chain(pieces)
            .chain(hexes)
            .max_by(|(_, za, _), (_, zb, _)| za.partial_cmp(zb).expect("NaN"))
            .map(|(e, _, t)| (e, t))
            .expect("Surface will be hit if nothing else is");

        let dst_t = match htype {
            DropTargetType::Surface => {
                if *parent == ehit {
                    **src_t
                }
                else {
                    let dst_gt = gt_query.get(ehit)?;
                    src_gt.reparented_to(dst_gt)
                }.translation + drag_dist
            },
            DropTargetType::Piece => {
                let src_sg = sg_query.get(*esrc)?;
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

        commands.trigger(DoMoveEvent {
            entity: *esrc,
            src_parent: *parent,
            src: src_t.translation,
            dst_parent: ehit,
            dst: dst_t
        });
    }

    if bottoms.len() > 1 {
        commands.trigger(CloseGroupEvent);
    }

    Ok(())
}
