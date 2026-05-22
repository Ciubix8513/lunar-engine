use crate::{self as lunar_engine, ecs::EntityBuilder};
use std::any::TypeId;

use lunar_engine_derive::marker_component;

use crate::ecs::{Entity, World};

use super::tracking::EventFilter;

#[test]
fn queue_creation_querying() {
    let world = World::new();

    let q1 = world
        .queues
        .write()
        .create_event_queue(EventFilter::SingleType(TypeId::of::<i32>()));

    let q2 = world
        .queues
        .write()
        .create_event_queue(EventFilter::MultipleTypes(vec![
            TypeId::of::<i32>(),
            TypeId::of::<u32>(),
        ]));

    let q3 = world
        .queues
        .write()
        .create_event_queue(EventFilter::Complex(&|_| false));

    assert!(world.queues.read().get_queue(q1).is_some());
    assert!(world.queues.read().get_queue(q2).is_some());
    assert!(world.queues.read().get_queue(q3).is_some());
}

#[marker_component]
struct TestComponent;

#[test]
fn entity_events() {
    let mut world = World::new();

    let e = world.add_entity(Entity::new()).unwrap();

    let q3 = world
        .queues
        .write()
        .create_event_queue(EventFilter::SingleType(TypeId::of::<TestComponent>()));

    e.upgrade()
        .unwrap()
        .write()
        .add_component::<TestComponent>()
        .unwrap();

    e.upgrade()
        .unwrap()
        .write()
        .remove_component::<TestComponent>()
        .unwrap();

    let q = world.queues.read().get_queue(q3).unwrap().clone();

    assert_eq!(q.read().len(), 2);

    q.write().clear();

    let e = world
        .add_entity(
            EntityBuilder::new()
                .add_component::<TestComponent>()
                .create()
                .unwrap(),
        )
        .unwrap();

    let id = e.upgrade().unwrap().read().id;

    world.remove_entity_by_id(id).unwrap();

    assert_eq!(q.read().len(), 2);

    q.write().clear();
}
