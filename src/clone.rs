use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
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
    log::{EditIndex, EditOf, EditType, Edits, handle_do, RedoCloneEvent, UndoCloneEvent},
    object::{NextObjectId, ObjectId, ObjectIdMap},
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

fn do_clone(
    entity: Entity,
    object_id: u32,
    actions: &Actions,
    commands: &mut Commands
)
{
    let mut ec = commands.entity(entity);

    // clone everything but the ObjectId
    let mut ec = ec.clone_and_spawn_with_opt_out(
        |builder| { builder.deny::<ObjectId>(); }
    );
    // assign the clone's ObjectId
    ec.insert(ObjectId(object_id));

    add_observers(&mut ec);
    add_action_observers(actions.0.iter().map(|a| a.action), &mut ec);
}

#[derive(Clone, Copy, EntityEvent)]
pub struct CloneEvent {
    pub entity: Entity 
}

#[derive(Component)]
pub struct CloneEdit {
    pub object_id: u32,
    pub source_id: u32
}

#[instrument(skip_all)]
pub fn on_clone(
    evt: On<CloneEvent>,
    piece_query: Query<(&ObjectId, &Actions)>,
    mut edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    mut next_object_id: ResMut<NextObjectId>,
    mut commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let (source_id, actions) = piece_query.get(entity)?;

    let object_id = next_object_id.0;
    next_object_id.0 += 1;

    let (edits_entity, mut edits, mut edit_index) = edit_query.single_mut()?;
    handle_do(&mut edits, &mut edit_index, &mut commands);
 
    commands.spawn((
        EditOf(edits_entity),
        EditType::Clone,
        CloneEdit { object_id, source_id: source_id.0 }
    ));

    do_clone(entity, object_id, actions, &mut commands);
    Ok(())
}

#[instrument(skip_all)]
pub fn on_clone_undo(
    evt: On<UndoCloneEvent>,
    edit: Query<&CloneEdit>,
    objmap: Res<ObjectIdMap>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cl) = edit.get(evt.entity) else { return Ok(()); };
    // get the entity being edited
    let entity = *objmap.0.get(&cl.object_id).unwrap();
    // apply the change
    commands.entity(entity).despawn();
    Ok(())
}

#[instrument(skip_all)]
pub fn on_clone_redo(
    evt: On<RedoCloneEvent>,
    edit: Query<&CloneEdit>,
    objmap: Res<ObjectIdMap>,
    query: Query<&Actions>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(cl) = edit.get(evt.entity) else { return Ok(()); };
    // get the source entity
    let entity = *objmap.0.get(&cl.source_id).unwrap();
    // get the components of the source entity
    let actions = query.get(entity)?;
    // apply the change
    do_clone(entity, cl.object_id, actions, &mut commands);
    Ok(())
}
