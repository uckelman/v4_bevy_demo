use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        event::EntityEvent,
        name::Name,
        observer::On,
        prelude::{Commands, EntityCommands, Query}
    },
    picking::Pickable,
    prelude::{Color, DespawnOnExit, Sprite, trace, Transform}
};

use crate::{
    actionkey::ActionKey,
    actionfunc::ActionFunc,
    actions::{add_action_observers},
    angle::Angle,
    assets::{ImageSource, SpriteHandles},
    drag::{Draggable, on_piece_drag_start, on_piece_drag, on_piece_drag_end},
    gamebox::PieceType,
    raise,
    select::{on_selection, on_deselection, Selectable, SelectEvent, DeselectEvent},
    state::GameState,
    view::handle_piece_pressed
};

#[derive(Clone, Component, Copy, Debug, Default)]
pub struct Piece;

// TODO: should this reference a piece type?
#[derive(Clone, Component, Debug, Default)]
pub struct Faces(pub Vec<ImageSource>);

// TODO: should this be a cyclic iterator?
#[derive(Clone, Component, Debug, Default)]
pub struct FaceUp(pub usize);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Action {
    pub label: String,
    pub action: ActionFunc,
    pub key: Option<ActionKey>
}

#[derive(Clone, Component, Debug, Default)]
pub struct Actions(pub Vec<Action>);

pub fn add_observers(commands: &mut EntityCommands<'_>) {
    commands
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
}

pub fn spawn_piece(
    p: &PieceType,
    t: Transform,
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
        Piece,
        Name::from(p.name.as_ref()),
        Pickable::default(),
        Selectable,
        Draggable,
        sprite,
        t,
        Faces(faces),
        FaceUp(0),
        Actions(p.actions.iter()
            .map(|a| Action {
                label: a.label.clone(),
                action: a.action,
                key: a.key
            })
            .collect::<Vec<_>>()
        ),
        DespawnOnExit(GameState::Game)
    ));

    add_observers(&mut ec);
    add_action_observers(p.actions.iter().map(|a| a.action), &mut ec);
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
