use bevy::{
    ecs::{
        event::EntityEvent,
        observer::On,
        prelude::Commands
    },
    input::keyboard::KeyCode,
    prelude::{Entity, Resource, trace}
};

use crate::{
    config::KeyConfig,
};

use tracing::instrument;

#[derive(Resource)]
pub struct DeleteKey(pub KeyCode);

impl KeyConfig for DeleteKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

#[derive(EntityEvent)]
pub struct DeleteEvent {
    pub entity: Entity
}

fn do_delete(
    entity: Entity,
    commands: &mut Commands
)
{
    commands.entity(entity).despawn();
}

#[instrument(skip_all)]
pub fn on_delete(
    del: On<DeleteEvent>,
    mut commands: Commands
)
{
    trace!("");
    let entity = del.event().event_target();
    do_delete(entity, &mut commands);
}
