use bevy::{
    ecs::{
        prelude::{ChildOf, Children, Entity, Query},
        query::{QueryData, QueryFilter},
        relationship::{Relationship, RelationshipTarget}
    }
};
use std::{
    collections::VecDeque,
    iter
};

use crate::piece::StackingGroup;

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
    entity: Entity,
    parent_query: &'w Query<'w, 's, DR, FR>,
    children_query: &'w Query<'w, 's, DS, FS>
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
