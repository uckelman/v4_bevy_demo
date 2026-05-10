use bevy::{
    ecs::{
        change_detection::{Res, ResMut},
        entity::Entity,
        event::EntityEvent,
        error::Result,
        observer::On,
        prelude::{ChildOf, Children, Query},
        query::{QueryData, QueryFilter},
        relationship::{Relationship, RelationshipTarget}
    },
    math::Vec3,
    picking::events::{Click, Pointer},
    prelude::{Resource, Transform}
};
use std::{
    collections::VecDeque,
    iter
};

use crate::{
    double_click::{DoubleClickThreshold, DoubleClickTimer},
    piece::StackingGroup
};

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

#[derive(Default, Resource)]
pub struct ExpandedStack(pub Option<Entity>);

pub fn expand_stack(
    mut evt: On<Pointer<Click>>,
    d_query: Query<(Option<&Children>, &StackingGroup)>,
    a_query: Query<(Option<&ChildOf>, &StackingGroup)>,
    mut t_query: Query<&mut Transform>,
    mut dct: ResMut<DoubleClickTimer>,
    dc_threshold: Res<DoubleClickThreshold>,
    mut exp_stack: ResMut<ExpandedStack>
) -> Result
{
// TODO: should we store the stack base as the target?

    let target = evt.event().event_target();

    evt.propagate(false);

    let stack_offset = Vec3::new(30.0, 30.0, 0.0);

    if target == dct.target && dct.timer.elapsed() <= dc_threshold.0 {
        let mut si = self::iter(&a_query, &d_query, dct.target);

        exp_stack.0 = si.next();

        // expand the stack
        for e in si {
            let mut t = t_query.get_mut(e)?;
            t.translation += stack_offset;
        }
    }

    dct.target = target;
    dct.timer.reset();
    Ok(())
}

// TODO: Send expand, collapse events?

pub fn collapse_stack(
    evt: On<Pointer<Click>>,
    d_query: Query<(Option<&Children>, &StackingGroup)>,
    mut t_query: Query<&mut Transform>,
    mut exp_stack: ResMut<ExpandedStack>
) -> Result
{
    if let Some(base) = exp_stack.0 {
        let stack_offset = Vec3::new(30.0, 30.0, 0.0);

        for e in d_query.iter_above(base) {
            let mut t = t_query.get_mut(e)?;
            t.translation -= stack_offset;
        }

        exp_stack.0 = None;
    }

    Ok(())
}
