use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EntityEvent,
        error::Result,
        observer::On,
        prelude::{Commands, Query},
        query::{QueryData, QueryFilter},
        relationship::{Relationship, RelationshipTarget}
    },
    math::Vec3,
    prelude::{trace, Transform}
};
use std::{
    collections::VecDeque,
    iter
};
use tracing::instrument;

use crate::piece::{Above, Below, Location, StackingGroup};

struct StackBelowIter<'w, 's, D: QueryData, F: QueryFilter, R: Relationship>
where
    D::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w R>, &'w StackingGroup)>
{
    parent_query: &'w Query<'w, 's, D, F>,
    stacking_group: StackingGroup,
    next: Option<Entity>
}

impl<'w, 's, D: QueryData, F: QueryFilter, R: Relationship> StackBelowIter<'w, 's, D, F, R>
where
    D::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w R>, &'w StackingGroup)>
{
    pub fn new(
        parent_query: &'w Query<'w, 's, D, F>,
        entity: Entity
    ) -> Self {
        let (next, stacking_group) = parent_query
            .get(entity)
            .map(|(p, sg)| (p.map(Relationship::get), *sg))
            .unwrap_or((None, StackingGroup(0)));

        Self {
            parent_query,
            stacking_group,
            next
        }
    }
}

impl<'w, 's, D: QueryData, F: QueryFilter, R: Relationship> Iterator
    for StackBelowIter<'w, 's, D, F, R>
where
    D::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w R>, &'w StackingGroup)>
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok((parent, &sg)) = self.parent_query.get(self.next?)
            && sg == self.stacking_group
        {
            let ret = self.next;
            self.next = parent.map(Relationship::get);
            ret
        }
        else {
            None
        }
    }
}

struct StackAboveIter<'w, 's, D: QueryData, F: QueryFilter, S: RelationshipTarget>
where
    D::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w S>, &'w StackingGroup)>
{
    children_query: &'w Query<'w, 's, D, F>,
    stacking_group: StackingGroup,
    next: VecDeque<Entity>
}

impl<'w, 's, D: QueryData, F: QueryFilter, S: RelationshipTarget> StackAboveIter<'w, 's, D, F, S>
where
    D::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w S>, &'w StackingGroup)>
{
    pub fn new(
        children_query: &'w Query<'w, 's, D, F>,
        entity: Entity
    ) -> Self {
        let (next, stacking_group) = children_query
            .get(entity)
            .map(|(ch, sg)| (
                ch.map_or_else(VecDeque::new, |c| c.iter().collect()),
                *sg
            ))
            .unwrap_or((VecDeque::new(), StackingGroup(0)));

        Self {
            children_query,
            stacking_group,
            next
        }
    }
}

impl<'w, 's, D: QueryData, F: QueryFilter, S: RelationshipTarget> Iterator
    for StackAboveIter<'w, 's, D, F, S>
where
    D::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w S>, &'w StackingGroup)>,
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entity = self.next.pop_front()?;
            if let Ok((children, &sg)) = self.children_query.get(entity)
                && sg == self.stacking_group
            {
                if let Some(children) = children {
                    self.next.extend(children.iter());
                }

                return Some(entity);
            }
        }
    }
}

pub fn iter<'w, 's, DR: QueryData, FR: QueryFilter, R: Relationship, DS: QueryData, FS: QueryFilter, S: RelationshipTarget> (
    parent_query: &'w Query<'w, 's, DR, FR>,
    children_query: &'w Query<'w, 's, DS, FS>,
    entity: Entity,
) -> impl Iterator<Item = Entity>
where
    DR::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w R>, &'w StackingGroup)>,
    DS::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w S>, &'w StackingGroup)>
{
    StackBelowIter::new(parent_query, entity)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .chain(iter::once(entity))
        .chain(StackAboveIter::new(children_query, entity))
}

pub trait StackAboveQueryExt<'w> {
    fn iter_above(
        &'w self,
        entity: Entity
    ) -> impl Iterator<Item = Entity>;

    fn top(&'w self, entity: Entity) -> Entity;
}

impl<'w, 's, D, F, S> StackAboveQueryExt<'w> for Query<'w, 's, D, F>
where
    D: QueryData,
    D::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w S>, &'w StackingGroup)>,
    F: QueryFilter,
    S: RelationshipTarget
{
    fn iter_above(
        &'w self,
        entity: Entity
    ) -> impl Iterator<Item = Entity>
    {
        StackAboveIter::new(self, entity)
    }

    fn top(&'w self, entity: Entity) -> Entity
    {
        self.iter_above(entity).last().unwrap_or(entity)
    }
}

pub trait StackBelowQueryExt<'w> {
    fn iter_below(
        &'w self,
        entity: Entity
    ) -> impl Iterator<Item = Entity>;

    fn bottom(&'w self, entity: Entity) -> Entity;
}

impl<'w, 's, D, F, R> StackBelowQueryExt<'w> for Query<'w, 's, D, F>
where
    D: QueryData,
    D::ReadOnly: QueryData<Item<'w, 's> = (Option<&'w R>, &'w StackingGroup)>,
    F: QueryFilter,
    R: Relationship
{
    fn iter_below(
        &'w self,
        entity: Entity
    ) -> impl Iterator<Item = Entity>
    {
        StackBelowIter::new(self, entity)
    }

    fn bottom(&'w self, entity: Entity) -> Entity
    {
        self.iter_below(entity).last().unwrap_or(entity)
    }
}

// FIXME: this messes with cloning because Expanded gets cloned
//#[derive(Clone, Component, Copy, Debug, Default)]
#[derive(Component, Debug, Default)]
pub struct Expanded;

#[derive(EntityEvent)]
pub struct ExpandEvent {
    entity: Entity
}

#[derive(EntityEvent)]
pub struct CollapseEvent {
    entity: Entity
}


#[derive(EntityEvent)]
pub struct RestackEvent {
    entity: Entity
}

/*
#[instrument(skip_all)]
pub fn on_expand_stack(
    expand: On<ExpandEvent>,
    a_query: Query<(Option<&Above>, &StackingGroup)>,
    d_query: Query<(Option<&Below>, &StackingGroup)>,
    parent_query: Query<&ChildOf>,
    gt_query: Query<&GlobalTransform>,
    mut t_query: Query<(&Above, &mut Transform, &GlobalTransform)>,
    mut commands: Commands
) -> Result
{
    trace!("");

    let entity = expand.event().event_target();

    let mut si = self::iter(&a_query, &d_query, entity);

    if let Some(base) = si.next() {
        commands.entity(base).insert(Expanded);

// TODO: make stack offset a gamebox setting
        let stack_offset = Vec3::new(30.0, 30.0, 0.0);

        let root = parent_query.root_ancestor(base);
        let root_gt = gt_query.get(root)?;

        // expand the stack
        for (i, e) in si.enumerate() {
            let (p, mut t, gt) = t_query.get_mut(e)?;

            *t = gt.reparented_to(root_gt);
            t.translation += ((i + 1) as f32) * stack_offset;

            commands.entity(e).insert(Expanded);
            commands.entity(root).add_child(e);
        }
    }

    Ok(())
}
*/

#[instrument(skip_all)]
pub fn on_expand_stack(
    expand: On<ExpandEvent>,
    a_query: Query<(Option<&Above>, &StackingGroup)>,
    d_query: Query<(Option<&Below>, &StackingGroup)>,
    base_query: Query<(&Above, &Location)>,
    mut t_query: Query<&mut Transform>,
    mut commands: Commands
) -> Result
{
    trace!("");

    let entity = expand.event().event_target();

    let mut si = self::iter(&a_query, &d_query, entity);

    if let Some(base) = si.next() && let Some(second) = si.next() {
        let stack_offset = Vec3::new(30.0, 30.0, 1.0);

        // reparent pieces in stack to the parent of the stack base
        // spread pieces in stack by stack offset

        let (base_par, base_loc) = base_query.get(base)?;

        for (i, e) in [base, second].into_iter().chain(si).enumerate() {
            let mut t = t_query.get_mut(e)?;
            t.translation = base_loc.0 + (i as f32) * stack_offset;
            commands.entity(base_par.0).add_child(e);
            commands.entity(e).insert(Expanded);
        }
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn on_collapse_stack(
    collapse: On<CollapseEvent>,
    a_query: Query<(Option<&Above>, &StackingGroup)>,
    d_query: Query<(Option<&Below>, &StackingGroup)>,
    mut t_query: Query<(&mut Transform, &Above, &Location)>,
    mut commands: Commands
) -> Result
{
    trace!("");

    let entity = collapse.event().event_target();

    for e in self::iter(&a_query, &d_query, entity) {
        let (mut t, p, loc) = t_query.get_mut(e)?;

        commands.entity(e).remove::<Expanded>();
        commands.entity(p.0).add_child(e);
        t.translation = loc.0;
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn on_restack(
    restack: On<RestackEvent>,
    a_query: Query<(Option<&Above>, &StackingGroup)>,
    d_query: Query<(Option<&Below>, &StackingGroup)>,
    base_query: Query<(&Above, &Location)>,
    mut t_query: Query<&mut Transform>,
    mut commands: Commands
) -> Result
{
    trace!("");

    let entity = restack.event().event_target();

    let mut si = self::iter(&a_query, &d_query, entity);

    if let Some(base) = si.next() {
        if let Some(second) = si.next() {
            // reparent pieces in stack to the parent of the stack base
            // spread pieces in stack by stack offset

            let stack_offset = Vec3::new(30.0, 30.0, 1.0);

            let (base_par, base_loc) = base_query.get(base)?;

            for (i, e) in [base, second].into_iter().chain(si).enumerate() {
                let mut t = t_query.get_mut(e)?;
                t.translation = base_loc.0 + (i as f32) * stack_offset;
                commands.entity(base_par.0).add_child(e);
                commands.entity(e).insert(Expanded);
            }
        }
        else {
            // base is the last piece in the stack, so collapse it
            let (base_par, base_loc) = base_query.get(base)?;
            let mut t = t_query.get_mut(base)?;
            t.translation = base_loc.0;
            commands.entity(base_par.0).add_child(base);
            commands.entity(base).remove::<Expanded>();
        }
    }

    Ok(())
}

pub fn expand_stack(entity: Entity, commands: &mut Commands) {
    commands.trigger(ExpandEvent { entity });
}

pub fn collapse_stack(entity: Entity, commands: &mut Commands) {
    commands.trigger(CollapseEvent { entity });
}

pub fn restack_stack(entity: Entity, commands: &mut Commands) {
    commands.trigger(RestackEvent { entity });
}
