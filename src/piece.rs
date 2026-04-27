use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EntityEvent,
        name::Name,
        observer::On,
        prelude::{ChildOf, Commands, EntityCommands, Query}
    },
    math::{Quat, Vec3},
    picking::Pickable,
    prelude::{Color, debug, Sprite, trace, Transform, Visibility}
};
use tracing::instrument;

use crate::{
    actionfunc::{ActionFunc, add_action_observers},
    assets::{ImageSource, SpriteHandles},
    drag::{Draggable, handle_drop, on_piece_drag_start, on_piece_drag, on_piece_drag_end},
    gamebox::{Anchor, PieceType},
    keys::KeyBinding,
    object::ObjectId,
    piece::r#move::on_move,
    raise,
    select::{on_selection, on_deselection, Selectable, SelectEvent, DeselectEvent},
    view::{handle_context_menu, handle_piece_pressed}
};

pub mod clone;
pub mod create;
pub mod delete;
pub mod flip;
pub mod r#move;
pub mod rotate;

#[derive(Clone, Component, Copy, Debug, Default)]
pub struct Piece;

#[derive(Clone, Component, Copy, Debug)]
pub struct PieceTypeId(pub u32);

#[derive(Clone, Component, Copy, Debug, Eq, PartialEq)]
pub struct StackingGroup(pub u32);

// TODO: should this reference a piece type?
#[derive(Clone, Component, Debug, Default)]
pub struct Faces(pub Vec<ImageSource>);

// TODO: should this be a cyclic iterator?
#[derive(Clone, Component, Copy, Debug, Default)]
pub struct FaceUp(pub usize);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Action {
    pub label: String,
    pub action: ActionFunc,
    pub key: Option<KeyBinding>
}

#[derive(Clone, Component, Debug, Default)]
pub struct Actions(pub Vec<Action>);

#[derive(Clone, Component, Copy, Debug)]
pub struct Parent(pub Entity);

#[derive(Clone, Component, Copy, Debug)]
pub struct Location(pub Vec3);

#[derive(Clone, Component, Copy, Debug)]
pub struct Angle(pub f32);

pub fn add_draggable_observers(ec: &mut EntityCommands) {
    ec
        .observe(on_piece_drag_start)
        .observe(on_piece_drag)
        .observe(on_piece_drag_end)
        .observe(on_move);
}

pub fn add_selectable_observers(ec: &mut EntityCommands) {
    ec
        .observe(recolor_on::<SelectEvent>(Color::hsl(0.0, 0.9, 0.7)))
        .observe(recolor_on::<DeselectEvent>(Color::WHITE))
        .observe(on_selection)
        .observe(on_deselection)
        .observe(raise::on_piece_pressed)
        .observe(raise::on_piece_released)
        .observe(handle_piece_pressed);
}

pub fn spawn_piece(
    oid: u32,
    pid: u32,
    p: &PieceType,
    parent: Entity,
    location: Vec3,
    angle: f32,
    anchor: Anchor,
    faceup: usize,
    sprite_handles: &SpriteHandles,
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

    use std::f32::consts::PI;

    let t = Transform {
        translation: location,
        rotation: Quat::from_rotation_z(angle * PI / 180.0),
        scale: Vec3::ONE
    };

    let anchor: bevy::sprite::Anchor = anchor.into();

    trace!("piece {}", t.translation.z);

    let mut ec = commands.spawn((
        (
            Piece,
            ObjectId(oid),
            PieceTypeId(pid),
            Name::from(p.name.as_ref()),
            StackingGroup(p.stacking_group),
            Parent(parent),
            Location(t.translation),
            Angle(angle),
            sprite,
            ChildOf(parent),
            t,
            anchor
        ),
        (
            Faces(faces),
            FaceUp(faceup),
            Actions(p.actions.iter()
                .map(|a| Action {
                    label: a.label.clone(),
                    action: a.action,
                    key: a.key.clone()
                })
                .collect::<Vec<_>>()
            ),
            Pickable::default(),
            Visibility::Inherited
        )
    ));

    ec
        .observe(handle_context_menu)
        .observe(handle_drop);

    if p.selectable {
        ec.insert(Selectable);
        add_selectable_observers(&mut ec);
    }

    if p.draggable {
        ec.insert(Draggable);
        add_draggable_observers(&mut ec);
    }

    add_action_observers(p.actions.iter().map(|a| a.action), &mut ec);

    debug!("piece {}", ec.id());
}

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
