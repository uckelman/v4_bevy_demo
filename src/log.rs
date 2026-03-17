use bevy::{
    ecs::{
        change_detection::Res,
        component::Component,
        event::{EntityEvent, Event},
        observer::On,
        prelude::{Commands, RelationshipTarget, With, Without},
        world::DeferredWorld
    },
    input::keyboard::KeyCode,
    math::Vec3,
    prelude::{debug, Entity, EntityRef, Query, Resource, Result}
};
use std::{
    fmt,
    fs::File,
    io::{BufReader, Write}
};
use serde::{
    Serialize, Serializer,
    ser::SerializeSeq
};
use tracing::instrument;

use crate::{
    LogPath,
    clone::CloneEdit,
    create::CreateEdit,
    delete::DeleteEdit,
    flip::FlipEdit,
    r#move::MoveEdit,
    rotate::RotateEdit,
    config::KeyConfig
};

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

// the edit index is the insertion point for a new edit
#[derive(Component, Default)]
pub struct EditIndex(pub usize);

#[derive(Component)]
#[relationship(relationship_target = Edits)]
pub struct EditOf(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = EditOf, linked_spawn)]
pub struct Edits(Vec<Entity>);

#[derive(Clone, Component, Copy, Debug, Eq, PartialEq)]
pub enum EditType {
    Clone,
    Create,
    Delete,
    Flip,
    Group,
    Move,
    Rotate
}

impl EditType {
    fn dispatch_undo_event(&self, entity: Entity, commands: &mut Commands) {
        match self {
            EditType::Clone => commands.trigger(UndoCloneEvent { entity }),
            EditType::Create => commands.trigger(UndoCreateEvent { entity }),
            EditType::Delete => commands.trigger(UndoDeleteEvent { entity }),
            EditType::Flip => commands.trigger(UndoFlipEvent { entity }),
            EditType::Group => commands.trigger(UndoGroupEvent { entity }),
            EditType::Move => commands.trigger(UndoMoveEvent { entity }),
            EditType::Rotate => commands.trigger(UndoRotateEvent { entity })
        }
    }

    fn dispatch_redo_event(&self, entity: Entity, commands: &mut Commands) {
        match self {
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

#[derive(Event)]
pub struct EditsComplete;

pub fn handle_do<E>(
    mut edits_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    edit_type: EditType,
    edit: E,
    mut commands: Commands
) -> Result
where
    E: Component
{
    // there must be a unique edit cursor on an edit group
    let (edits_entity, mut edits, mut edit_index) = edits_query.single_mut()?;

    // drop the redos from the parent
    edits.0.drain(edit_index.0..)
        .for_each(|e| commands.entity(e).despawn());

    // create the event
    let ee = commands.spawn((
        EditOf(edits_entity),
        edit_type,
        edit
    )).id();

    // step forward
    commands.trigger(RedoEvent { entity: ee });
    edit_index.0 += 1;

    commands.trigger(EditsComplete);

    Ok(())
}

#[derive(Event)]
pub struct OpenGroupEvent;

pub fn on_group_open(
    _evt: On<OpenGroupEvent>,
    parent_query: Query<(Entity, &mut Edits, &EditIndex)>,
    commands: Commands
) -> Result
{
    handle_group_open(parent_query, commands)
}

pub fn handle_group_open(
    mut parent_query: Query<(Entity, &mut Edits, &EditIndex)>,
    mut commands: Commands
) -> Result
{
    debug!("handle_open_group");

    // there must be a unique edit cursor on an edit group
    let (parent_entity, mut edits, edit_index) = parent_query.single_mut()?;

    // drop the redos from the parent
    edits.0.drain(edit_index.0..)
        .for_each(|e| commands.entity(e).despawn());

    // remove edit cursor from the parent
    commands.get_entity(parent_entity)?
        .remove::<EditIndex>();

    // create the child group with new edit cursor
    commands.spawn((
        EditOf(parent_entity),
        EditType::Group,
        Edits::default(),
        EditIndex(0)
    ));

    commands.trigger(EditsComplete);
    Ok(())
}

#[derive(Event)]
pub struct CloseGroupEvent;

pub fn on_group_close(
    _evt: On<CloseGroupEvent>,
    edits_query: Query<(Entity, &EditType, &EditOf), With<EditIndex>>,
    parent_query: Query<&Edits, Without<EditIndex>>,
    commands: Commands
) -> Result
{
    handle_group_close(edits_query, parent_query, commands)
}

pub fn handle_group_close(
    edits_query: Query<(Entity, &EditType, &EditOf), With<EditIndex>>,
    parent_query: Query<&Edits, Without<EditIndex>>,
    mut commands: Commands
) -> Result
{
    debug!("handle_close_group");

    // there must be a unique edit cursor
    let (edits_entity, edit_type, parent_entity) = edits_query.single()?;

    if *edit_type != EditType::Group {
        return Ok(());
    }

    // remove edit cursor from this group
    commands.get_entity(edits_entity)?
        .remove::<EditIndex>();

    // add edit cursor to the parent group after this group
    let parent_edits = parent_query.get(parent_entity.0)?;

    let pos = parent_edits.iter()
        .position(|e| e == edits_entity)
        .expect("child must exist in parent") + 1;

    commands.get_entity(parent_entity.0)?
        .insert(EditIndex(pos));

    commands.trigger(EditsComplete);

    Ok(())
}

#[derive(EntityEvent)]
pub struct UndoEvent {
    pub entity: Entity
}

pub fn handle_undo(
    mut edits_query: Query<(Entity, &Edits, &mut EditIndex, Option<&EditOf>)>,
    parent_query: Query<&Edits, Without<EditIndex>>,
    mut commands: Commands
) -> Result
{
    debug!("handle_undo");

    let Ok((edits_entity, edits, mut edit_index, parent_entity)) = edits_query.single_mut() else {
        return Ok(());
    };

    if edit_index.0 > 0 {
        // we are not at the start of our group, just back up one
        edit_index.0 -= 1;
        commands.trigger(UndoEvent { entity: edits.0[edit_index.0] });

        commands.trigger(EditsComplete);
    }
    else if let Some(parent_entity) = parent_entity {
        // we're at the start of our group and we have a parent
        // we don't need to trigger any UndoEvents here

        // remove edit cursor from this group
        commands.get_entity(edits_entity)?
            .remove::<EditIndex>();

        // insert edit cursor before this group in parent
        let parent_edits = parent_query.get(parent_entity.0)?;

        let pos = parent_edits.iter()
            .position(|e| e == edits_entity)
            .expect("child must exist in parent");

        commands.get_entity(parent_entity.0)?
            .insert(EditIndex(pos));

        commands.trigger(EditsComplete);
    }

    Ok(())
}

pub fn handle_undo_up(
    edits_query: Query<(Entity, &Edits, &EditIndex, Option<&EditOf>)>,
    parent_query: Query<&Edits, Without<EditIndex>>,
    mut commands: Commands
) -> Result
{
    debug!("handle_undo_up");

    let Ok((edits_entity, edits, edit_index, parent_entity)) = edits_query.single() else {
        return Ok(());
    };

    let Some(parent_entity) = parent_entity else {
        return Ok(());
    };

    // undo everything in this group before the edit cursor
    edits.0[..edit_index.0]
        .iter()
        .rev()
        .for_each(|&e| commands.trigger(UndoEvent { entity: e }));

    // remove edit cursor from this group
    commands.get_entity(edits_entity)?
        .remove::<EditIndex>();

    // add edit cursor to the parent group
    let parent_edits = parent_query.get(parent_entity.0)?;

    let pos = parent_edits.iter()
        .position(|e| e == edits_entity)
        .expect("child must exist in parent");

    commands.get_entity(parent_entity.0)?
        .insert(EditIndex(pos));

    commands.trigger(EditsComplete);

    Ok(())
}

#[derive(EntityEvent)]
pub struct RedoEvent {
    pub entity: Entity
}

pub fn handle_redo_over(
    mut query: Query<(&Edits, &mut EditIndex)>,
    mut commands: Commands
) -> Result
{
    debug!("handle_redo_over");

    let (edits, mut edit_index) = query.single_mut()?;

    if edit_index.0 == edits.0.len() {
        return Ok(());
    }

    // step forward
    commands.trigger(RedoEvent { entity: edits.0[edit_index.0] });
    edit_index.0 += 1;

    commands.trigger(EditsComplete);

    Ok(())
}

pub fn handle_redo_in(
    mut parent_query: Query<(Entity, &Edits, &mut EditIndex)>,
    edit_query: Query<(Entity, &EditType)>,
    mut commands: Commands
) -> Result
{
    debug!("handle_redo_in");

    let (parent_entity, edits, mut edit_index) = parent_query.single_mut()?;

    if edit_index.0 == edits.0.len() {
        return Ok(());
    }

    let Ok((edit_entity, etype)) = edit_query.get(edits.0[edit_index.0]) else {
        return Ok(());
    };

    if *etype == EditType::Group {
        // step into this group

        // remove edit cursor from parent group
        commands.get_entity(parent_entity)?
            .remove::<EditIndex>();

        // add edit cursor to the child group
        commands.get_entity(edit_entity)?
            .insert(EditIndex(0));
    }
    else {
        // this is not a group, just step forward
        commands.trigger(RedoEvent { entity: edits.0[edit_index.0] });
        edit_index.0 += 1;
    }

    commands.trigger(EditsComplete);

    Ok(())
}

pub fn handle_redo_out(
    mut edits_query: Query<(Entity, &EditType, &Edits, &mut EditIndex, Option<&EditOf>)>,
    parent_query: Query<&Edits, Without<EditIndex>>,
    mut commands: Commands
) -> Result
{
    debug!("handle_redo_out");

    // there must be a unique edit cursor
    let (edits_entity, &edit_type, edits, mut edit_index, parent_entity) = edits_query.single_mut()?;

    if let Some(parent_entity) = parent_entity &&
        edit_type == EditType::Group
    {
        // redo everything in this group to the end
        edits.0[edit_index.0..]
        .iter()
        .for_each(|&e| commands.trigger(RedoEvent { entity: e }));

        // remove edit cursor from this group
        commands.get_entity(edits_entity)?
            .remove::<EditIndex>();

        // add edit cursor to the parent group after this group
        let parent_edits = parent_query.get(parent_entity.0)?;

        let pos = parent_edits.iter()
            .position(|e| e == edits_entity)
            .expect("child must exist in parent") + 1;

        commands.get_entity(parent_entity.0)?
            .insert(EditIndex(pos));
    }
    else {
        // this is the root or not a group, just step forward
        commands.trigger(RedoEvent { entity: edits.0[edit_index.0] });
        edit_index.0 += 1;
    }

    commands.trigger(EditsComplete);

    Ok(())
}

#[derive(Clone, EntityEvent)]
pub struct DoCloneEvent {
    pub entity: Entity
}

#[derive(Clone, Event)]
pub struct DoCreateEvent {
    pub type_id: u32,
    pub dst: Vec3
}

#[derive(Clone, EntityEvent)]
pub struct DoDeleteEvent {
    pub entity: Entity
}

#[derive(Clone, EntityEvent)]
pub struct DoFlipEvent {
    pub entity: Entity,
    pub delta: i32
}

/*
#[derive(EntityEvent)]
pub struct DoGroupEvent {
    pub entity: Entity
}
*/

#[derive(Clone, EntityEvent)]
pub struct DoMoveEvent {
    pub entity: Entity,
    pub src: Vec3,
    pub dst: Vec3
}

#[derive(Clone, EntityEvent)]
pub struct DoRotateEvent {
    pub entity: Entity,
    pub dtheta: f32
}

#[derive(EntityEvent)]
pub struct UndoCloneEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct UndoCreateEvent {
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

#[derive(EntityEvent)]
pub struct UndoGroupEvent {
    pub entity: Entity
}

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
    query.get(evt.entity)?
        .dispatch_undo_event(evt.entity, &mut commands);
    Ok(())
}

#[derive(EntityEvent)]
pub struct RedoCloneEvent {
    pub entity: Entity
}

#[derive(EntityEvent)]
pub struct RedoCreateEvent {
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

#[derive(EntityEvent)]
pub struct RedoGroupEvent {
    pub entity: Entity
}

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
    query.get(evt.entity)?
        .dispatch_redo_event(evt.entity, &mut commands);
    Ok(())
}

/*
#[instrument(skip_all)]
pub fn on_group(
    evt: On<GroupEvent>,
    mut edit_query: Query<(Entity, &mut Edits, &mut EditIndex)>,
    current_edits: Res<CurrentEdits>,
    mut commands: Commands
) -> Result
{

/*
    let (edits_entity, mut edits, mut edit_index) = edit_query.entity(current_edits.0)?;

    commands.spawn((
        EditOf(edits_entity),
        EditType::Flip,
        FlipEdit { object_id: object_id.0, delta: evt.delta }
    ));

*/


    evt.0.iter().cloned().for_each(|e| commands.trigger(e));
    Ok(())
}
*/

#[instrument(skip_all)]
pub fn on_group_undo(
    evt: On<UndoGroupEvent>,
    edit_query: Query<&Edits>,
    c_query: Query<&EditType>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(edits) = edit_query.get(evt.entity) else { return Ok(()); };

    for &entity in &edits.0 {
        c_query.get(entity)?
            .dispatch_undo_event(entity, &mut commands);
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn on_group_redo(
    evt: On<RedoGroupEvent>,
    edit_query: Query<&Edits>,
    c_query: Query<&EditType>,
    mut commands: Commands
) -> Result
{
    // get the edit
    let Ok(edits) = edit_query.get(evt.entity) else { return Ok(()); };

    for &entity in &edits.0 {
        c_query.get(entity)?
            .dispatch_redo_event(entity, &mut commands);
    }

    Ok(())
}

pub fn dump_edits(
    _evt: On<EditsComplete>,
    root_query: Query<(Entity, &Edits), Without<EditOf>>,
    edit_query: Query<&EditType>,
    edits_query: Query<&Edits>,
    edit_index_query: Query<(Entity, &EditIndex)>
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

    if edits.0.is_empty() {
        if cur_entity == entity && cur_idx == 0 {
            eprintln!("{}-->", indent);
        }
    }
    else {
        for (i, &e) in edits.0.iter().enumerate() {
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

        if cur_entity == entity && cur_idx == edits.0.len() {
            eprintln!("{}-->", indent);
        }
    }

    Ok(())
}

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
struct GroupProxyOut<'e, 's, 'w>(
    Entity,
    &'e Edits,
    &'s [(Entity, usize)],
    &'w DeferredWorld<'w>
);

impl Serialize for GroupProxyOut<'_, '_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let GroupProxyOut(entity, edits, stops, world) = &self;

        let mut edit_query = world.try_query::<(EntityRef, &EditType)>()
            .expect("no query");

        // don't go beyond the redo boundary
        let len = if let Some((stop_entity, stop_idx)) = stops.last()
            && entity == stop_entity { *stop_idx } else { edits.0.len() };

        let mut seq = serializer.serialize_seq(Some(len))?;

        for &e in edits.0.iter().take(len) {
            let (eref, etype) = edit_query.get(world, e)
                .map_err(serde::ser::Error::custom)?;

            match etype {
                EditType::Clone => seq.serialize_edit::<CloneEdit>(eref)?,
                EditType::Create => seq.serialize_edit::<CreateEdit>(eref)?,
                EditType::Delete => seq.serialize_edit::<DeleteEdit>(eref)?,
                EditType::Flip => seq.serialize_edit::<FlipEdit>(eref)?,
                EditType::Group => seq.serialize_element(
                    &GroupProxyOut(
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

        let idx = edits.0.iter()
            .position(|&ed| e == ed)
            .expect("child must exist in parent") + 1;

        stops.push((e, idx));
    }

    let g = GroupProxyOut(root_entity, root_edits, &stops, &world);

    serde_json::to_writer(&mut writer, &g)?;
    writeln!(&mut writer)?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
enum EditProxyIn {
    Clone(CloneEdit),
    Create(CreateEdit),
    Delete(DeleteEdit),
    Flip(FlipEdit),
    Move(MoveEdit),
    Rotate(RotateEdit),
    #[serde(untagged)]
    Group(Vec<EditProxyIn>)
}

fn add_edit_proxy(
    parent_entity: Entity,
    ep: EditProxyIn,
    commands: &mut Commands
) -> Entity
{
    match ep {
        EditProxyIn::Clone(ed) => commands.spawn((
            EditOf(parent_entity),
            EditType::Clone,
            ed
        )).id(),
        EditProxyIn::Create(ed) => commands.spawn((
            EditOf(parent_entity),
            EditType::Create,
            ed
        )).id(),
        EditProxyIn::Delete(ed) => commands.spawn((
            EditOf(parent_entity),
            EditType::Delete,
            ed
        )).id(),
        EditProxyIn::Flip(ed) => commands.spawn((
            EditOf(parent_entity),
            EditType::Flip,
            ed
        )).id(),
        EditProxyIn::Group(g) => {
            let group_entity = commands.spawn((
                EditOf(parent_entity),
                EditType::Group,
                Edits::default()
            )).id();

            for ed in g {
                add_edit_proxy(group_entity, ed, commands);
            }

            group_entity
        },
        EditProxyIn::Move(ed) => commands.spawn((
            EditOf(parent_entity),
            EditType::Move,
            ed
        )).id(),
        EditProxyIn::Rotate(ed) => commands.spawn((
            EditOf(parent_entity),
            EditType::Rotate,
            ed
        )).id()
    }
}

#[instrument(skip_all)]
pub fn load_log(
    log_path: Res<LogPath>,
    mut commands: Commands
) -> Result {
    debug!("");

    // create the log root
    let root_entity = commands.spawn((
        Edits::default(),
        EditIndex::default()
    )).id();

    if let Some(path) = log_path.0.as_ref() {
        // there is a log to load
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let g: Vec<EditProxyIn> = serde_json::from_reader(reader)?;

        let entities = g.into_iter()
            .map(|ep| add_edit_proxy(root_entity, ep, &mut commands))
            .collect::<Vec<_>>();

//        commands.trigger(EditsComplete);
/*
        for e in entities {
            commands.trigger(RedoEvent { entity: e });
        }
*/
    }

    Ok(())
}
