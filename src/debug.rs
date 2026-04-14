use bevy::{
    camera::Camera,
    ecs::{
        event::EntityEvent,
        message::MessageReader,
        name::Name,
        observer::On,
        prelude::{Entity, Query, RelationshipTarget, With, Without},
        world::DeferredWorld
    },
    picking::{
        events::{Click, Pointer}
    },
    prelude::{CursorMoved, GlobalTransform, Result, trace},
    window::{PrimaryWindow, Window}
};

use crate::{
    edittype::EditType,
    log::{EditOf, EditIndex, Edits, EditsComplete}
};

pub fn pick_dbg(ev: On<Pointer<Click>>, names: Query<&Name>) {
    let name = names
        .get(ev.event_target())
        .map(|n| n.to_string())
        .unwrap_or("Unknown".to_string());

    trace!("Picked {name}({:?})", ev.event_target());
}

pub fn dump_entities(
    all: Query<Entity>,
    world: DeferredWorld
) -> Result
{
    for e in all.iter() {
        eprintln!("{e}");
        for ci in world.inspect_entity(e)? {
            eprintln!("{}", ci.name());
        }
        eprintln!();
    }

    Ok(())
}

/*
    for (root_entity, root_edits) in root_query.iter() {
        eprintln!("{root_entity}");
    }
    Ok(())
*/

pub fn dump_edits(
    _evt: On<EditsComplete>,
    root_query: Query<(Entity, &Edits), Without<EditOf>>,
    edit_query: Query<&EditType>,
    edits_query: Query<&Edits>,
    edit_index_query: Query<(Entity, &EditIndex)>,
) -> Result
{
    eprintln!();

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

pub fn cursor_position(
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
) -> Result
{
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single()?;

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single()?;

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z

    let Some(cursor) = window.cursor_position() else { return Ok(()); };
    let world_position = camera.viewport_to_world(camera_transform, cursor)
        .map(|ray| ray.origin.truncate())?;

    eprintln!("mouse @ {:.0},{:.0}", world_position.x, world_position.y);

    Ok(())
}

pub fn cursor_events(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut reader: MessageReader<CursorMoved>
) -> Result
{
    if reader.is_empty() {
        return Ok(());
    }

    let (camera, camera_transform) = q_camera.single()?;
    let window = q_window.single()?;

    for ev in reader.read() {
        let world_pos = camera.viewport_to_world(camera_transform, ev.position)
            .map(|ray| ray.origin.truncate())?;

        println!(
//            "mouse @ {:.0},{:.0} in {:?}",
            "mouse @ {},{} in {:?}",
            world_pos.x,
            world_pos.y,
            ev.window
        );
    }

    Ok(())
}
