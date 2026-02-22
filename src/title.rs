use bevy::{
    camera::Camera2d,
    ecs::children,
    prelude::{AlignItems, Commands, DespawnOnExit, FlexDirection, JustifyContent, Node, OrthographicProjection, Projection, Resource, Text, TextFont, Timer, TimerMode, Val}
};

use crate::state::GameState;

#[derive(Resource)]
pub struct SplashScreenTimer(pub Timer);

pub fn display_title(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection::default_2d())
    ));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        children![
            (
                Text::new("V4 Bevy Demo"),
                TextFont {
                    font_size: 130.0,
                    ..Default::default()
                },
            ),
            (
                Text::new("December 2025"),
                TextFont {
                    font_size: 100.0,
                    ..Default::default()
                },
            )
        ],
        DespawnOnExit(GameState::Splash),
    ));

    commands.insert_resource(
        SplashScreenTimer(Timer::from_seconds(2.0, TimerMode::Once))
    );
}
