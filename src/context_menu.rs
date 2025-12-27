use bevy::{
    color::{
        Color,
        palettes::tailwind::{GRAY_50, GRAY_200, GRAY_400}
    },
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        event::{EntityEvent, Event},
        observer::On,
        prelude::{Commands, Query, With},
        relationship::RelatedSpawnerCommands
    },
    input::mouse::AccumulatedMouseScroll,
    math::Vec2,
    picking::{
        Pickable,
        events::{Out, Over, Pointer, Press},
        pointer::PointerButton
    },
    prelude::{BackgroundColor, BorderColor, BorderRadius, Button, children, ChildOf, Display, Entity, FlexDirection, JustifySelf, NextState, Node, PositionType, px, Reflect, SpawnRelated, States, Text, TextColor, TextFont, trace, UiRect, Val}
};
use itertools::Itertools;
use std::fmt::Debug;
use tracing::instrument;

use crate::{
    actionfunc::ActionFunc,
    actions::trigger_action,
    piece::{Action, Actions},
    select::Selected
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, States)]
pub enum ContextMenuState {
    #[default]
    Closed,
    Open
}

#[derive(EntityEvent)]
pub struct OpenContextMenu {
    pub entity: Entity,
    pub pos: Vec2
}

#[derive(Event, Default)]
pub struct CloseContextMenus;

#[derive(Component, Default)]
pub struct ContextMenu;

#[derive(Component)]
pub struct ContextMenuItem(ActionFunc);

#[instrument(skip_all)]
pub fn open_context_menu(
    open: On<OpenContextMenu>,
    query: Query<&Actions, With<Selected>>,
    mut next_state: ResMut<NextState<ContextMenuState>>,
    mut commands: Commands
)
{
    trace!("");

    // show intersection of actions for selected entities
    let actions: Vec<Action> = query.iter()
        .flat_map(|a| a.0.iter())
        .unique()
        .cloned()
        .collect::<Vec<_>>();

    if actions.is_empty() {
        // there are no actions shared by all selected items
        return;
    }

    next_state.set(ContextMenuState::Open);

    let bg_color = GRAY_50.into();
    let label_color = Color::BLACK;
    let key_color = GRAY_400.into();
    let border_color: Color = GRAY_200.into();
    let highlight_color = GRAY_200.into();

    let font = TextFont {
        font_size: 14.0,
        ..Default::default()
    };

    commands.spawn((
        ContextMenu,
        Node {
            position_type: PositionType::Absolute,
            left: px(open.pos.x),
            top: px(open.pos.y),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(4)),
            border: UiRect::all(px(1)),
            ..Default::default()
        },
        Pickable {
            should_block_lower: true,
            is_hoverable: false
        },
        BorderColor::all(border_color),
        BorderRadius::all(px(4)),
        BackgroundColor(bg_color),
    ))
    .with_children(|parent|
        actions.iter()
            .for_each(|a| {
                make_context_item(
                    a,
                    font.clone(),
                    bg_color,
                    label_color,
                    key_color,
                    parent
                );
            })
    )
    .observe(on_item_selection)
    .observe(highlight_on_hover::<Out>(bg_color))
    .observe(highlight_on_hover::<Over>(highlight_color));
}

fn make_context_item(
    action: &Action,
    font: TextFont,
    bg_color: Color,
    label_color: Color,
    key_color: Color,
    commands: &mut RelatedSpawnerCommands<'_, ChildOf>
)
{
    let mut item = commands.spawn((
        ContextMenuItem(action.action),
        Button,
        Node {
            padding: UiRect::all(px(5)),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(font.font_size),
            ..Default::default()
        },
        BackgroundColor(bg_color),
        BorderRadius::all(px(4)),
        Pickable::default(),
        children![(
            Node {
                justify_self: JustifySelf::Start,
                flex_grow: 1.0,
                ..Default::default()
            },
            Pickable::IGNORE,
            Text::new(action.label.clone()),
            font.clone(),
            TextColor(label_color)
        )]
    ));

    if let Some(key) = &action.key {
        item.with_children(|item| {
            item.spawn((
                Node {
                    justify_self: JustifySelf::End,
                    ..Default::default()
                },
                Pickable::IGNORE,
                Text::new(key.clone()),
                font.clone(),
                TextColor(key_color)
            ));
        });
    }
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

fn on_item_selection(
    mut press: On<Pointer<Press>>,
    menu_items: Query<&ContextMenuItem>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands
)
{
    let target = press.original_event_target();

    if let Ok(item) = menu_items.get(target)
        && press.button == PointerButton::Primary
    {
        commands.trigger(CloseContextMenus);
        query.iter()
            .for_each(|entity| trigger_action(entity, item.0, &mut commands));
    }

    press.propagate(false);
}

#[instrument(skip_all)]
pub fn trigger_close_context_menus_press(
    _press: On<Pointer<Press>>,
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
    mut next_state: ResMut<NextState<ContextMenuState>>,
    mut commands: Commands,
)
{
    trace!("");
    next_state.set(ContextMenuState::Closed);
    menus.iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
