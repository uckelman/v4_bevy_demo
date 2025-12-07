use bevy::{
    ecs::prelude::Commands,
    prelude::{Entity, EntityCommands}
};

use crate::flip::{FlipForwardEvent, FlipBackEvent, on_flip_forward, on_flip_back};

pub fn add_action_observer(
    name: &str,
    ec: &mut EntityCommands<'_>
)
{
    match name {
        "flip_forward" => ec.observe(on_flip_forward),
        "flip_back" => ec.observe(on_flip_back),
        _ => todo!()
    };
}

pub fn trigger_action(
    entity: Entity,
    name: &str,
    commands: &mut Commands
)
{
    match name {
        "flip_forward" => commands.trigger(FlipForwardEvent { entity }),
        "flip_back" => commands.trigger(FlipBackEvent { entity }),
        _ => todo!()
    };
}
