use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        event::EntityEvent,
        error::Result,
        observer::On,
        prelude::{Commands, Query}
    },
    prelude::{Entity, trace}
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    actionfunc::add_action_observers,
    edittype::EditType,
    log::{EditIndex, Edits, handle_do},
    object::{NextObjectId, ObjectId, ObjectIdMap},
    piece::{Actions, add_observers}
};

#[derive(Clone, EntityEvent)]
pub struct DoCloneEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct UndoCloneEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoCloneEvent {
    pub entity: Entity
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

#[derive(Component, Debug, Deserialize, Serialize)]
#[serde(rename = "clone", tag = "type")]
pub struct CloneEdit {
    pub object_id: u32,
    pub source_id: u32
}

#[instrument(skip_all)]
pub fn on_clone(
    evt: On<DoCloneEvent>,
    piece_query: Query<&ObjectId>,
    mut next_object_id: ResMut<NextObjectId>,
    edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    commands: Commands
) -> Result
{
    trace!("");

    let entity = evt.event().event_target();
    let source_id = piece_query.get(entity)?;

    let object_id = next_object_id.0;
    next_object_id.0 += 1;

    handle_do(
        edit_query,
        EditType::Clone,
        CloneEdit { object_id, source_id: source_id.0 },
        commands
    )
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
