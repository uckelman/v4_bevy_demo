use bevy::{
    ecs::{
        event::EntityEvent,
        observer::On,
        name::Name,
        prelude::{Entity, Query, RelationshipTarget, Without},
    },
    picking::{
        events::{Click, Pointer}
    },
    prelude::{Result, trace}
};

use crate::log::{EditOf, EditIndex, EditType, Edits, EditsComplete};

pub fn pick_dbg(ev: On<Pointer<Click>>, names: Query<&Name>) {
    let name = names
        .get(ev.event_target())
        .map(|n| n.to_string())
        .unwrap_or("Unknown".to_string());

    trace!("Picked {name}({:?})", ev.event_target());
}

pub fn dump_edits(
    _evt: On<EditsComplete>,
    root_query: Query<(Entity, &Edits), Without<EditOf>>,
    edit_query: Query<&EditType>,
    edits_query: Query<&Edits>,
    edit_index_query: Query<(Entity, &EditIndex)>,
//    all: Query<Entity>,
//    world: DeferredWorld
) -> Result
{
    eprintln!();

/*
    for e in all.iter() {
        eprintln!("{e}");
        for ci in world.inspect_entity(e)? {
            eprintln!("{}", ci.name());
        }
        eprintln!("");
    }

    Ok(())
*/

/*
    for (root_entity, root_edits) in root_query.iter() {
        eprintln!("{root_entity}");
    }
    Ok(())
*/

    let (cur_entity, cur_idx) = edit_index_query.single()?;
    let (root_entity, root_edits) = root_query.single()?;
    dump_group(
        root_entity,
        root_edits,
        0,
        cur_entity,
        cur_idx.0,
        &edit_query,
        &edits_query
    )
}

fn dump_group(
    entity: Entity,
    edits: &Edits,
    level: usize,
    cur_entity: Entity,
    cur_idx: usize,
    edit_query: &Query<&EditType>,
    edits_query: &Query<&Edits>
) -> Result
{
    let indent = "  ".repeat(level);

    if edits.is_empty() {
        if cur_entity == entity && cur_idx == 0 {
            eprintln!("{}-->", indent);
        }
    }
    else {
        for (i, e) in edits.iter().enumerate() {
            if cur_entity == entity && cur_idx == i {
                eprintln!("{}-->", indent);
            }

            let etype = edit_query.get(e)?;
            eprintln!("{}{:?}", indent, etype);

            if etype == &EditType::Group {
                dump_group(
                    e,
                    edits_query.get(e)?,
                    level + 1,
                    cur_entity,
                    cur_idx,
                    edit_query,
                    edits_query
                )?;
            }
        }

        if cur_entity == entity && cur_idx == edits.len() {
            eprintln!("{}-->", indent);
        }
    }

    Ok(())
}
