use bevy::{
    ecs::{
        bundle::Bundle,
        change_detection::Res,
        component::Component,
        event::EntityEvent,
        name::Name,
        observer::On,
        prelude::{Commands, Query}
    },
    picking::Pickable,
    prelude::{Color, DespawnOnExit, Sprite, trace, Transform}
};

use crate::{
    actions::{add_action_observer},
    assets::{ImageSource, SpriteHandles},
    drag::{Draggable, on_piece_drag_start, on_piece_drag, on_piece_drag_end},
    gamebox::PieceType,
    raise,
    select::{on_selection, on_deselection, Selectable, SelectEvent, DeselectEvent},
    state::GameState,
    view::handle_piece_pressed
};

#[derive(Component, Default)]
pub struct Piece;

// TODO: should this reference a piece type?
#[derive(Component, Default)]
pub struct Faces(pub Vec<ImageSource>);

// TODO: should this be a cyclic iterator?
#[derive(Component, Default)]
pub struct FaceUp(pub usize);

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

pub fn spawn_piece(
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
