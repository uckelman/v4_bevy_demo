use bevy::{
    ecs::{
        component::Component,
        lifecycle::HookContext,
        world::DeferredWorld
   },
   prelude::{debug, Entity, Resource}
};
use std::collections::HashMap;

#[derive(Component, Default)]
#[component(
    immutable,
    on_insert = insert_object_id,
    on_replace = discard_object_id
)]
//    on_discard = on_discard_piece_id
pub struct ObjectId(pub u32);

#[derive(Default, Resource)]
pub struct ObjectIdMap(pub HashMap<u32, Entity>);

#[derive(Default, Resource)]
pub struct NextObjectId(pub u32);

/*
fn get_oid(
    mut world: &mut DeferredWorld<'_>,
    entity: &Entity
) -> u32
{
    world.entity(entity).get::<ObjectId>()
        .expect("Insert hook guarantees ObjectId is available on entity")
        .0
}
*/

fn insert_object_id(
    mut world: DeferredWorld<'_>,
    HookContext { entity, .. }: HookContext
)
{
    let oid = world.entity(entity).get::<ObjectId>()
        .expect("Insert hook guarantees ObjectId is available on entity")
        .0;

    let mut index = world.get_resource_mut::<ObjectIdMap>()
        .expect("ObjectIdMap is created before any ObjectIds");

    index.0.insert(oid, entity);
    debug!("added {}", oid);
}

fn discard_object_id(
    mut world: DeferredWorld<'_>,
    HookContext { entity, .. }: HookContext
)
{
    let oid = world.entity(entity).get::<ObjectId>()
        .expect("Insert hook guarantees ObjectId is available on entity")
        .0;

    let mut index = world.get_resource_mut::<ObjectIdMap>()
        .expect("ObjectIdMap is created before any ObjectIds");

    index.0.remove(&oid);
    debug!("removed {}", oid);
}
