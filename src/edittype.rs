use bevy::ecs::{
    component::Component,
    prelude::{Commands, Entity}
};

use crate::{
    grid,
    log::{RedoGroupEvent, UndoGroupEvent},
    piece::{
        clone::{RedoCloneEvent, UndoCloneEvent},
        create::{RedoCreateEvent, UndoCreateEvent},
        delete::{RedoDeleteEvent, UndoDeleteEvent},
        flip::{RedoFlipEvent, UndoFlipEvent},
        r#move::{RedoMoveEvent, UndoMoveEvent},
        rotate::{RedoRotateEvent, UndoRotateEvent}
    },
    surface
};

#[derive(Clone, Component, Copy, Debug, Eq, PartialEq)]
pub enum EditType {
    CreateSurface,
    CreateGrid,
    Clone,
    Create,
    Delete,
    Flip,
    Group,
    Move,
    Rotate
}

impl EditType {
    pub fn dispatch_undo_event(&self, entity: Entity, commands: &mut Commands) {
        match self {
            EditType::CreateSurface => commands.trigger(surface::create::UndoCreateEvent { entity }),
            EditType::CreateGrid => commands.trigger(grid::create::UndoCreateEvent { entity }),
            EditType::Clone => commands.trigger(UndoCloneEvent { entity }),
            EditType::Create => commands.trigger(UndoCreateEvent { entity }),
            EditType::Delete => commands.trigger(UndoDeleteEvent { entity }),
            EditType::Flip => commands.trigger(UndoFlipEvent { entity }),
            EditType::Group => commands.trigger(UndoGroupEvent { entity }),
            EditType::Move => commands.trigger(UndoMoveEvent { entity }),
            EditType::Rotate => commands.trigger(UndoRotateEvent { entity })
        }
    }

    pub fn dispatch_redo_event(&self, entity: Entity, commands: &mut Commands) {
        match self {
            EditType::CreateSurface => commands.trigger(surface::create::RedoCreateEvent { entity }),
            EditType::CreateGrid => commands.trigger(grid::create::RedoCreateEvent { entity }),
            EditType::Clone => commands.trigger(RedoCloneEvent { entity }),
            EditType::Create => commands.trigger(RedoCreateEvent { entity }),
            EditType::Delete => commands.trigger(RedoDeleteEvent { entity }),
            EditType::Flip => commands.trigger(RedoFlipEvent { entity }),
            EditType::Group => commands.trigger(RedoGroupEvent { entity }),
            EditType::Move => commands.trigger(RedoMoveEvent { entity }),
            EditType::Rotate => commands.trigger(RedoRotateEvent { entity })
        }
    }
}
