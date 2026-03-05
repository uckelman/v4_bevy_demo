use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        component::Component,
        event::EntityEvent,
        observer::On,
        prelude::Commands
    },
    input::keyboard::KeyCode,
    prelude::{debug, Entity, Query, Resource, Result}
};
use tracing::instrument;

use crate::{
    config::KeyConfig,
    object::ObjectIdMap
};

/*
#[derive(Default, Resource)]
pub struct ActionLog {
    done: Vec<Action>,
    undone: Vec<Action>
}
*/

#[derive(Resource)]
pub struct RedoKey(pub KeyCode);

#[derive(Resource)]
pub struct UndoKey(pub KeyCode);

impl KeyConfig for RedoKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

impl KeyConfig for UndoKey {
    fn code(&self) -> KeyCode {
        self.0
    }
}

/*
#[instrument(skip_all)]
pub fn handle_do(
    mut log: &mut ResMut<ActionLog>,
    objmap: &Res<ObjectIdMap>,
    a: Action,
    commands: &mut Commands
)
{
    debug!("");

    log.undone.clear();
    a.commit(&objmap, commands);
    log.done.push(a);
}

#[instrument(skip_all)]
pub fn handle_undo(
    mut log: ResMut<ActionLog>,
    objmap: Res<ObjectIdMap>,
    mut commands: Commands
)
{
    debug!("");

    if let Some(a) = log.done.pop() {
        debug!("undoing");
        a.revert(&objmap, &mut commands);
        log.undone.push(a);
    }
}

#[instrument(skip_all)]
pub fn handle_redo(
    mut log: ResMut<ActionLog>,
    objmap: Res<ObjectIdMap>,
    mut commands: Commands
)
{
    debug!("");

    if let Some(a) = log.undone.pop() {
        debug!("redoing");
        a.commit(&objmap, &mut commands);
        log.done.push(a);
    }
}
*/

#[derive(EntityEvent)]
pub struct UndoEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoEvent {
    pub entity: Entity
}

// the edit index is the insertion point for a new edit
#[derive(Component, Default)]
pub struct EditIndex(pub usize);

#[derive(Component)]
#[relationship(relationship_target = Edits)]
pub struct EditOf(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = EditOf, linked_spawn)]
pub struct Edits(Vec<Entity>);

#[derive(Component)]
pub enum EditType {
    Clone,
    Delete,
    Flip,
//    Group,
    Move,
    Rotate
}

#[instrument(skip_all)]
pub fn handle_do(
    mut edits: &mut Edits,
    mut edit_index: &mut EditIndex,
    commands: &mut Commands
)
{
    // dump the redos
    edits.0.drain(edit_index.0..).for_each(|e| commands.entity(e).despawn());
    edit_index.0 += 1;
}

#[instrument(skip_all)]
pub fn handle_undo(
    mut query: Query<(&Edits, &mut EditIndex)>,
    mut commands: Commands
) -> Result
{
    debug!("");

    let (edits, mut edit_index) = query.single_mut()?;
    if edit_index.0 > 0 {
        edit_index.0 -= 1;
        commands.trigger(UndoEvent { entity: edits.0[edit_index.0] });
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn handle_redo(
    mut query: Query<(&Edits, &mut EditIndex)>,
    mut commands: Commands
) -> Result
{
    debug!("");

    let (edits, mut edit_index) = query.single_mut()?;
    if edit_index.0 < edits.0.len() {
        commands.trigger(RedoEvent { entity: edits.0[edit_index.0] });
        edit_index.0 += 1;
    }

    Ok(())
}

#[derive(EntityEvent)]
pub struct UndoCloneEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct UndoDeleteEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct UndoFlipEvent {
    pub entity: Entity
}

/*
#[derive(EntityEvent)]
pub struct UndoGroupEvent {
    pub entity: Entity
}
*/

#[derive(EntityEvent)]
pub struct UndoMoveEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct UndoRotateEvent {
    pub entity: Entity
}

#[instrument(skip_all)]
pub fn on_undo(
     evt: On<UndoEvent>,
     query: Query<&EditType>,
     mut commands: Commands
) -> Result
{
    let entity = evt.entity;
    match query.get(entity)? {
        EditType::Clone => commands.trigger(UndoCloneEvent { entity }),
        EditType::Delete => commands.trigger(UndoDeleteEvent { entity }),
        EditType::Flip => commands.trigger(UndoFlipEvent { entity }),
//        EditType::Group => commands.trigger(UndoGroupEvent { entity }),
        EditType::Move => commands.trigger(UndoMoveEvent { entity }),
        EditType::Rotate => commands.trigger(UndoRotateEvent { entity })
    }

    Ok(())
}

#[derive(EntityEvent)]
pub struct RedoCloneEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoDeleteEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoFlipEvent {
    pub entity: Entity
}

/*
#[derive(EntityEvent)]
pub struct RedoGroupEvent {
    pub entity: Entity
}
*/

#[derive(EntityEvent)]
pub struct RedoMoveEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoRotateEvent {
    pub entity: Entity
}

#[instrument(skip_all)]
pub fn on_redo(
     evt: On<RedoEvent>,
     query: Query<&EditType>,
     mut commands: Commands
) -> Result
{
    let entity = evt.entity;
    match query.get(entity)? {
        EditType::Clone => commands.trigger(RedoCloneEvent { entity }),
        EditType::Delete => commands.trigger(RedoDeleteEvent { entity }),
        EditType::Flip => commands.trigger(RedoFlipEvent { entity }),
//        EditType::Group => commands.trigger(RedoGroupEvent { entity }),
        EditType::Move => commands.trigger(RedoMoveEvent { entity }),
        EditType::Rotate => commands.trigger(RedoRotateEvent { entity })
    }

    Ok(())
}

/*
#[derive(Clone, Copy, Event)]
pub struct GroupEvent {
}

#[derive(Component)]
pub struct GroupEdit {
}

#[instrument(skip_all)]
pub fn on_group(
    evt: On<DeleteEvent>,
    mut edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    mut commands: Commands
) -> Result
{
}

#[instrument(skip_all)]
pub fn on_group_undo(
    evt: On<UndoGroupEvent>,
    edit: Query<&GroupEdit>,
    mut commands: Commands
) -> Result
{
}

#[instrument(skip_all)]
pub fn on_group_redo(
    evt: On<RedoGroupEvent>,
    edit: Query<&GroupEdit>,
    mut commands: Commands
) -> Result
{
}
*/
