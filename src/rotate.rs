use bevy::{
    ecs::{
        error::Result,
        event::EntityEvent,
        observer::On,
        prelude::Query
    },
    input::keyboard::KeyCode,
    prelude::{Entity, Resource, trace, Transform}
};
use tracing::instrument;

use crate::{
    config::KeyConfig
};

#[derive(EntityEvent)]
pub struct RotateEvent {
    pub entity: Entity,
    pub dtheta: f32
}

#[derive(Resource)]
pub struct RotateCWKey(pub KeyCode);

#[derive(Resource)]
pub struct RotateCCWKey(pub KeyCode);

impl KeyConfig for RotateCWKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for RotateCCWKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

fn do_rotation(
    entity: Entity,
    mut query: Query<&mut Transform>,
    dtheta: f32
) -> Result
{
    use std::f32::consts::PI;

    const DEG_TO_RAD: f32 = PI / 180.0;

    let mut t = query.get_mut(entity)?;

    t.rotate_local_z(dtheta * DEG_TO_RAD);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_rotate(
    evt: On<RotateEvent>,
    query: Query<&mut Transform>
) -> Result
{
    trace!("");
    let entity = evt.event().event_target();
    do_rotation(entity, query, evt.dtheta)
}
