use bevy::{
    ecs::{
        change_detection::Res,
        prelude::Commands
    },
    math::Vec3
};

use crate::{
    clone::CloneEvent,
    delete::{CreateEvent, DeleteEvent},
    flip::FlipEvent,
    object::ObjectIdMap,
    rotate::RotateEvent,
};

#[derive(Clone, Debug)]
pub struct PieceData {
    pub piece_id: u32,
//    pub location: Vec3,
//    pub angle: f32,
//    pub face_up: i32,
}

pub enum Action {
    Flip(u32, i32),
    Rotate(u32, f32),
    Create(PieceData),
    Delete(PieceData),
    Clone(u32, u32),
    Group(Vec<Action>)
}

impl Action {
    pub fn flatten(self) -> Self {
        match self {
            Self::Group(mut v) if v.len() == 1 => v.pop().unwrap(),
            _ => self
        }
    }

    pub fn commit(
        &self,
        objmap: &Res<ObjectIdMap>,
        commands: &mut Commands
    )
    {
        match self {
            Self::Flip(piece_id, delta) => commands.trigger(FlipEvent {
                entity: *objmap.0.get(piece_id).unwrap(),
                delta: *delta
            }),
            Self::Rotate(piece_id, dtheta) => commands.trigger(RotateEvent {
                entity: *objmap.0.get(piece_id).unwrap(),
                dtheta: *dtheta
            }),
            Self::Create(pd) => commands.trigger(CreateEvent { pd: pd.clone() }),
            // TODO
            Self::Delete(pd) => commands.trigger(DeleteEvent {
                entity: *objmap.0.get(&pd.piece_id).unwrap()
            }),
            Self::Clone(piece_id, parent_piece_id) => commands.trigger(CloneEvent {
                entity: *objmap.0.get(parent_piece_id).unwrap(),
                clone_id: *piece_id
            }),
            Self::Group(v) => v.iter()
                .for_each(|a| a.commit(objmap, commands))
        }
    }

    pub fn revert(
        &self,
        objmap: &Res<ObjectIdMap>,
        commands: &mut Commands
    )
    {
        match self {
            Self::Flip(piece_id, delta) => commands.trigger(FlipEvent {
                entity: *objmap.0.get(piece_id).unwrap(),
                delta: -delta
            }),
            Self::Rotate(piece_id, dtheta) => commands.trigger(RotateEvent {
                entity: *objmap.0.get(piece_id).unwrap(),
                dtheta: -dtheta
            }),
            Self::Create(pd) => commands.trigger(DeleteEvent {
                entity: *objmap.0.get(&pd.piece_id).unwrap()
            }),
            Self::Delete(pd) => commands.trigger(CreateEvent { pd: pd.clone() }),
            Self::Clone(piece_id, _) => commands.trigger(DeleteEvent {
                entity: *objmap.0.get(piece_id).unwrap()
            }),
            Self::Group(v) => v.iter()
                .for_each(|a| a.revert(objmap, commands))
        }
    }
}
