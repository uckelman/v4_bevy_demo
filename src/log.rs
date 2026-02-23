use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        prelude::Commands
    },
    input::keyboard::KeyCode,
    prelude::{debug, Resource}
};
use tracing::instrument;

use crate::{
    action::Action,
    config::KeyConfig,
    object::ObjectIdMap
};

#[derive(Default, Resource)]
pub struct ActionLog {
    done: Vec<Action>,
    undone: Vec<Action>
}

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
