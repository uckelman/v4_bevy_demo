use bevy::{
    ecs::{
        event::EntityEvent,
        error::Result,
        observer::On,
        prelude::{Commands, Query}
    },
    input::keyboard::KeyCode,
    math::Vec2,
    prelude::{Entity, Resource, trace}
};

use crate::{
    actions::add_action_observers,
    config::KeyConfig,
    piece::{Actions, add_observers},
    select::Selected
};

use tracing::instrument;

#[derive(Resource)]
pub struct CloneKey(pub KeyCode);

impl KeyConfig for CloneKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

#[derive(EntityEvent)]
pub struct CloneEvent {
    pub entity: Entity
}

fn do_clone(
    entity: Entity,
    query: Query<&Actions>,
    commands: &mut Commands
) -> Result
{
    let mut ec = commands.entity(entity);
    let mut ec = ec.clone_and_spawn();

    let clone = ec.id();
    add_observers(&mut ec);
    
    let actions = query.get(entity)?;
    add_action_observers(actions.0.iter().map(|a| a.action), &mut ec);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_clone(
    del: On<CloneEvent>,
    query: Query<&Actions>,
    mut commands: Commands
) -> Result
{
    trace!("");
    let entity = del.event().event_target();
    do_clone(entity, query, &mut commands)
}
