use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        entity::Entity,
    },
    prelude::{Real, Resource, Time},
    time::Stopwatch
};
use std::time::Duration;

#[derive(Resource)]
pub struct DoubleClickThreshold(pub Duration);

#[derive(Resource)]
pub struct DoubleClickTimer {
    pub target: Entity,
    pub timer: Stopwatch
}

impl Default for DoubleClickTimer {
    fn default() -> Self {
        DoubleClickTimer {
            target: Entity::PLACEHOLDER,
            timer: Stopwatch::default()
        }
    }
}

pub fn tick_double_click_timer(
    time: Res<Time<Real>>,
    mut dct: ResMut<DoubleClickTimer>
)
{
    dct.timer.tick(time.delta());
}
