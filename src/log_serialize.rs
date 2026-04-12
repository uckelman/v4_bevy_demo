use bevy::{
    ecs::{
        component::Component,
        prelude::{RelationshipTarget, Without},
        world::DeferredWorld
    },
    prelude::{Entity, EntityRef, Result}
};
use serde::{
    Serialize, Serializer,
    ser::SerializeSeq
};
use std::io::Write;

use crate::{
    edittype::EditType,
    grid,
    log::{EditIndex, EditOf, Edits},
    piece::{
        clone::CloneEdit,
        create::CreateEdit,
        delete::DeleteEdit,
        flip::FlipEdit,
        r#move::MoveEdit,
        rotate::RotateEdit
    },
    surface
};

trait SerializeEdit {
    type Error: serde::ser::Error;

    fn serialize_edit<E>(
        &mut self,
        eref: EntityRef
    ) ->  Result<(), Self::Error>
    where
        E: Component + Serialize;
}

impl<S> SerializeEdit for S
where
    S: SerializeSeq
{
    type Error = S::Error;

    fn serialize_edit<E>(
        &mut self,
        eref: EntityRef
    ) ->  Result<(), Self::Error>
    where
        E: Component + Serialize
    {
        let ed = eref.get::<E>().expect("edit type mismatch");
        self.serialize_element(ed)
    }
}

// TODO: would it make sense to implement PartialOrd for the stop point?
struct GroupProxy<'e, 's, 'w>(
    Entity,
    &'e Edits,
    &'s [(Entity, usize)],
    &'w DeferredWorld<'w>
);

impl Serialize for GroupProxy<'_, '_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let GroupProxy(entity, edits, stops, world) = &self;

        let mut edit_query = world.try_query::<(EntityRef, &EditType)>()
            .expect("no query");

        // don't go beyond the redo boundary
        let len = if let Some((stop_entity, stop_idx)) = stops.last()
            && entity == stop_entity { *stop_idx } else { edits.len() };

        let mut seq = serializer.serialize_seq(Some(len))?;

        for e in edits.iter().take(len) {
            let (eref, etype) = edit_query.get(world, e)
                .map_err(serde::ser::Error::custom)?;

            match etype {
                EditType::CreateSurface => seq.serialize_edit::<surface::create::CreateEdit>(eref)?,
                EditType::CreateGrid => seq.serialize_edit::<grid::create::CreateEdit>(eref)?,
                EditType::Clone => seq.serialize_edit::<CloneEdit>(eref)?,
                EditType::Create => seq.serialize_edit::<CreateEdit>(eref)?,
                EditType::Delete => seq.serialize_edit::<DeleteEdit>(eref)?,
                EditType::Flip => seq.serialize_edit::<FlipEdit>(eref)?,
                EditType::Group => seq.serialize_element(
                    &GroupProxy(
                        e,
                        eref.get::<Edits>().expect("edit type mismatch"),
                        // peel off this level for the redo boundary
                        &stops[..stops.len().saturating_sub(1)],
                        world
                    )
                )?,
                EditType::Move => seq.serialize_edit::<MoveEdit>(eref)?,
                EditType::Rotate => seq.serialize_edit::<RotateEdit>(eref)?
            }
        };

        seq.end()
    }
}

pub fn serialize_edits(world: DeferredWorld) -> Result
{
    let mut writer = std::io::stdout();
    writeln!(&mut writer)?;

    // find the root
    let mut root_query = world.try_query_filtered::<(Entity, &Edits), Without<EditOf>>().expect("no query");

    let (root_entity, root_edits) = root_query.single(&world)?;

    // find the edit cursor
    let mut edit_index_query = world.try_query::<(Entity, &EditIndex)>()
        .expect("no query");

    let (cur_entity, cur_idx) = edit_index_query.single(&world)?;

    // find the redo boundary for the edit cursor
    let mut parent_query = world.try_query::<&EditOf>()
        .expect("no query");
    let mut parent_edits_query = world.try_query::<&Edits>()
        .expect("no query");

    let mut stops = vec![ (cur_entity, cur_idx.0) ];
    let mut e = cur_entity;
    while e != root_entity {
        e = parent_query.get(&world, e)?.0;
        let edits = parent_edits_query.get(&world, e)?;

        let idx = edits.iter()
            .position(|ed| e == ed)
            .expect("child must exist in parent") + 1;

        stops.push((e, idx));
    }

    let g = GroupProxy(root_entity, root_edits, &stops, &world);

    serde_json::to_writer(&mut writer, &g)?;
    writeln!(&mut writer)?;
    Ok(())
}
