use bevy::{
    color::{
        Color,
        palettes::tailwind::{GRAY_50, GRAY_200}
    },
    ecs::{
        bundle::Bundle,
        change_detection::Res,
        component::Component,
        event::{EntityEvent, Event},
        observer::On,
        prelude::{Commands, Query, With}
    },
    input::mouse::AccumulatedMouseScroll,
    math::Vec2,
    picking::{
        Pickable,
        events::{Out, Over, Pointer, Press},
        pointer::PointerButton
    },
    prelude::{BackgroundColor, BorderColor, BorderRadius, Button, children, Entity, FlexDirection, Node, PositionType, px, Reflect, SpawnRelated, Text, TextColor, TextFont, trace, UiRect}
};
use std::{
    collections::HashSet,
    fmt::Debug
};
use tracing::instrument;

use crate::{
    Actions,
    actions::trigger_action,
    select::Selected
};

#[derive(EntityEvent)]
pub struct OpenContextMenu {
    entity: Entity,
    pos: Vec2
}

#[derive(Event, Default)]
pub struct CloseContextMenus;

#[derive(Component, Default)]
pub struct ContextMenu;

#[derive(Component, Default)]
pub struct ContextMenuItem(String);

#[instrument(skip_all)]
pub fn open_piece_context_menu(
    mut press: On<Pointer<Press>>,
    mut commands: Commands
)
{
    trace!("");

    if press.button != PointerButton::Secondary {
        return;
    }

    commands.trigger(OpenContextMenu {
        entity: press.event().event_target(),
        pos: press.pointer_location.position
    });

    press.propagate(false);
}

#[instrument(skip_all)]
pub fn open_context_menu(
    open: On<OpenContextMenu>,
    query: Query<&Actions, With<Selected>>,
    mut commands: Commands
)
{
    trace!("");

    commands.trigger(CloseContextMenus);

    // show intersection of actions for selected entities
// FIXME: maintain order somehow
    let actions: Vec<String> = query.iter()
        .map(|a| HashSet::from_iter(a.0.iter()))
        .reduce(|acc: HashSet<&String>, s| &acc & &s)
        .unwrap_or_default()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    if actions.is_empty() {
        // there are no actions shared by all selected items
        return;
    }

    let bg = GRAY_50.into();
    let border: Color = GRAY_200.into();
    let highlight = GRAY_200.into();

    commands.spawn((
        ContextMenu,
        Node {
            position_type: PositionType::Absolute,
            left: px(open.pos.x),
            top: px(open.pos.y),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(4)),
            border: UiRect::all(px(1)),
            ..Default::default()
        },
        BorderColor::all(border),
        BorderRadius::all(px(4)),
        BackgroundColor(bg),
    ))
    .with_children(|parent|
        actions.iter()
            .for_each(|a| { parent.spawn(context_item(a, a, bg)); })
    )
    .observe(on_item_selection)
    .observe(highlight_on_hover::<Out>(bg))
    .observe(highlight_on_hover::<Over>(highlight));
}

fn on_item_selection(
    mut event: On<Pointer<Press>>,
    menu_items: Query<&ContextMenuItem>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
)
{
    let target = event.original_event_target();

    if let Ok(item) = menu_items.get(target) {
        query.iter()
            .for_each(|entity| trigger_action(entity, &item.0, &mut commands));
    }

    event.propagate(false);
}

fn context_item(key: &str, text: &str, bg: Color) -> impl Bundle {
    (
        ContextMenuItem(key.into()),
        Button,
        Node {
            padding: UiRect::all(px(5)),
            ..Default::default()
        },
        BackgroundColor(bg),
        BorderRadius::all(px(4)),
        children![(
            Pickable::IGNORE,
            Text::new(text),
            TextFont {
                font_size: 14.0,
                ..Default::default()
            },
            TextColor(Color::BLACK)
        )]
    )
}

#[instrument(skip_all)]
pub fn highlight_on_hover<T: Debug + Clone + Reflect>(
    color: Color,
) -> impl FnMut(
    On<Pointer<T>>,
    Query<&mut BackgroundColor, With<ContextMenuItem>>
)
{
    move |
        mut event: On<Pointer<T>>,
        mut bg_color: Query<&mut BackgroundColor, With<ContextMenuItem>>
    |
    {
        let Ok(mut bg) = bg_color.get_mut(event.original_event_target()) else {
            return;
        };

        bg.0 = color;

        event.propagate(false);
    }
}

#[instrument(skip_all)]
pub fn trigger_close_context_menus_press(
    _event: On<Pointer<Press>>,
    mut commands: Commands
)
{
    trace!("");
    commands.trigger(CloseContextMenus);
}

#[instrument(skip_all)]
pub fn trigger_close_context_menus_wheel(
    _mouse_scroll: Res<AccumulatedMouseScroll>,
    mut commands: Commands
)
{
    trace!("");
    commands.trigger(CloseContextMenus);
}

#[instrument(skip_all)]
pub fn close_context_menus(
    _event: On<CloseContextMenus>,
    menus: Query<Entity, With<ContextMenu>>,
    mut commands: Commands,
)
{
    trace!("");
    menus.iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
