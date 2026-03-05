use bevy::{
    ecs::{
        observer::On,
        event::{EntityEvent, Event}
    },
    prelude::{Commands, Entity, trace}
};
use tracing::instrument;

use crate::{
    actionfunc::ActionFunc,
    clone::CloneEvent,
    delete::DeleteEvent,
    flip::FlipEvent,
    r#move::MoveEvent,
    rotate::RotateEvent
};

#[derive(Clone, Event)]
pub enum EditEvent {
    Clone(CloneEvent),
    Delete(DeleteEvent),
    Flip(FlipEvent),
    Move(MoveEvent),
    Rotate(RotateEvent),
    Group(Vec<EditEvent>)
}

impl From<(Entity, ActionFunc)> for EditEvent {
    fn from((entity, afunc): (Entity, ActionFunc)) -> Self {
        match afunc {
            ActionFunc::Clone => Self::Clone(CloneEvent { entity }),
            ActionFunc::Delete => Self::Delete(DeleteEvent { entity }),
            ActionFunc::Flip(delta) => Self::Flip(FlipEvent { entity, delta }),
            ActionFunc::Rotate(dtheta) => Self::Rotate(RotateEvent { entity, dtheta: dtheta.0 })
        }
    }
}

impl EditEvent {
    pub fn collapse(self) -> Self {
        match self {
            Self::Group(mut events) if events.len() == 1 =>
                events.pop().expect("len is 1"),
            _ => self
        }
    }
}

#[instrument(skip_all)]
pub fn on_edit(
    evt: On<EditEvent>,
    mut commands: Commands
)
{
    trace!("");

    match *evt {
        EditEvent::Clone(e) => commands.trigger(e),
        EditEvent::Delete(e) => commands.trigger(e),
        EditEvent::Flip(e) => commands.trigger(e),
        EditEvent::Move(e) => commands.trigger(e),
        EditEvent::Rotate(e) => commands.trigger(e),
        EditEvent::Group(ref v) => v.iter().cloned().for_each(|e| commands.trigger(e))
    }
}
