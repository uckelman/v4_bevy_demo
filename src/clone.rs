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
    object::ObjectId,
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
    pub entity: Entity,
    pub clone_id: u32
}

#[derive(EntityEvent)]
pub struct DecloneEvent {
    pub entity: Entity
}

fn do_clone(
    entity: Entity,
    clone_id: u32,
    query: Query<&Actions>,
    commands: &mut Commands
) -> Result
{
    let mut ec = commands.entity(entity);

    // clone everything but the ObjectId
    let mut ec = ec.clone_and_spawn_with_opt_out(
        |builder| { builder.deny::<ObjectId>(); }
    );
    // assign the clone's ObjectId
    ec.insert(ObjectId(clone_id));

    add_observers(&mut ec);
    
    let actions = query.get(entity)?;
    add_action_observers(actions.0.iter().map(|a| a.action), &mut ec);

    Ok(())
}

#[instrument(skip_all)]
pub fn on_clone(
    evt: On<CloneEvent>,
    query: Query<&Actions>,
    mut commands: Commands
) -> Result
{
    trace!("");
    let entity = evt.event().event_target();
    do_clone(entity, evt.clone_id, query, &mut commands)
}

fn do_declone(
    entity: Entity,
    commands: &mut Commands
)
{
    commands.entity(entity).despawn();
}

#[instrument(skip_all)]
pub fn on_declone(
    evt: On<DecloneEvent>,
    mut commands: Commands
)
{
    trace!("");
    let entity = evt.event().event_target();
    do_declone(entity, &mut commands);
}
