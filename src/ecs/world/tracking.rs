use lock_api::RwLockUpgradableReadGuard;
use parking_lot::RwLock;
use rand::Rng;
use vec_key_value_pair::map::VecMap;

use crate::{
    UUID,
    ecs::{AnyComponentReference, ComponentReference, EntityRefence, WeakEntityRefence},
};
use std::{any::TypeId, sync::Arc};

///A Tracking event, indicateds that the world has changed
#[derive(Clone)]
pub enum Event {
    ///There was a component change
    Component(ComponentEvent),
    ///There was an entity change
    Entity(EntityEvent),
}

#[allow(missing_docs)]
#[derive(Clone)]
pub enum ComponentEvent {
    ///A component with the given type id and reference was added
    Addition(TypeId, AnyComponentReference),
    ///A component with the given type id was removed
    Removal(TypeId),
}

#[allow(missing_docs)]
#[derive(Clone)]
pub enum EntityEvent {
    ///An entity with uuid [`UUID`] has been added to the word
    Addition(UUID, Vec<TypeId>, WeakEntityRefence),
    ///An entity with uuid [`UUID`] has been removed from the world with
    Removal(UUID),
}

///An event filter
pub enum EventFilter {
    ///Will only get events with one specific type
    SingleType(TypeId),
    ///Will get events with any of the specified types
    MultipleTypes(Vec<TypeId>),
    ///Uses the proved function to determine whether an event should eb captured
    Complex(&'static (dyn Fn(TypeId) -> bool + Send + Sync)),
}

///A queue of component events
pub struct EventQueue {
    data: Vec<Event>,
    filter: EventFilter,
}

impl EventQueue {
    ///Creates a new event queue
    fn new(filter: EventFilter) -> Self {
        Self {
            data: Vec::new(),
            filter,
        }
    }

    ///Returns an iterator over the event queue
    pub fn iter(&self) -> std::slice::Iter<'_, Event> {
        self.data.iter()
    }

    ///Returns the length of the queue
    pub fn len(&self) -> usize {
        self.data.len()
    }

    ///Clears the events in a queue
    pub fn clear(&mut self) {
        self.data.clear();
    }

    ///Replaces the filter of an event queue with a new one
    pub fn set_filter(&mut self, filter: EventFilter) {
        self.filter = filter;
    }
}

///A collection of event queues
pub struct QueueContainer {
    queues: VecMap<UUID, Arc<RwLock<EventQueue>>>,
}

impl QueueContainer {
    ///Create a new queue contanier
    pub(crate) fn new() -> Self {
        Self {
            queues: VecMap::new(),
        }
    }

    ///Creates a new queue with the given filter, returning the id of the newly created queue
    pub fn create_event_queue(&mut self, filter: EventFilter) -> UUID {
        let id = rand::thread_rng().r#gen();
        self.queues
            .insert(id, Arc::new(RwLock::new(EventQueue::new(filter))));
        id
    }

    ///Returns a queue by its id
    pub fn get_queue(&self, id: UUID) -> Option<Arc<RwLock<EventQueue>>> {
        self.queues.get(&id).cloned()
    }

    pub(crate) fn process_add_component<T: 'static>(&self, cmp_ref: ComponentReference<T>) {
        for q in self.queues.iter() {
            let q = q.1.upgradable_read();
            let correct = match &q.filter {
                EventFilter::SingleType(type_id) => TypeId::of::<T>() == *type_id,
                EventFilter::MultipleTypes(type_ids) => type_ids.contains(&TypeId::of::<T>()),
                EventFilter::Complex(f) => f(TypeId::of::<T>()),
            };
            if correct {
                RwLockUpgradableReadGuard::upgrade(q)
                    .data
                    .push(Event::Component(ComponentEvent::Addition(
                        TypeId::of::<T>(),
                        AnyComponentReference::from_reference(cmp_ref),
                    )));
                return;
            }
        }
    }

    pub(crate) fn process_remove_component<T: 'static>(&self) {
        for q in self.queues.iter() {
            let q = q.1.upgradable_read();
            let correct = match &q.filter {
                EventFilter::SingleType(type_id) => &TypeId::of::<T>() == type_id,
                EventFilter::MultipleTypes(type_ids) => type_ids.contains(&TypeId::of::<T>()),
                EventFilter::Complex(f) => f(TypeId::of::<T>()),
            };
            if correct {
                RwLockUpgradableReadGuard::upgrade(q)
                    .data
                    .push(Event::Component(ComponentEvent::Removal(TypeId::of::<T>())));
                return;
            }
        }
    }

    pub(crate) fn process_add_entity(&self, weak: WeakEntityRefence) {
        let binding = weak.upgrade().unwrap();
        let e = binding.read();

        for c in &e.comoponent_types {
            for q in self.queues.iter() {
                let q = q.1.upgradable_read();
                let correct = match &q.filter {
                    EventFilter::SingleType(type_id) => c == type_id,
                    EventFilter::MultipleTypes(type_ids) => type_ids.contains(c),
                    EventFilter::Complex(f) => f(*c),
                };
                if correct {
                    RwLockUpgradableReadGuard::upgrade(q)
                        .data
                        .push(Event::Entity(EntityEvent::Addition(
                            e.id,
                            e.comoponent_types.clone(),
                            weak,
                        )));
                    return;
                }
            }
        }
    }

    pub(crate) fn process_remove_entity(&self, weak: WeakEntityRefence) {
        let binding = weak.upgrade().unwrap();
        let e = binding.read();

        for c in &e.comoponent_types {
            for q in self.queues.iter() {
                let q = q.1.upgradable_read();
                let correct = match &q.filter {
                    EventFilter::SingleType(type_id) => c == type_id,
                    EventFilter::MultipleTypes(type_ids) => type_ids.contains(c),
                    EventFilter::Complex(f) => f(*c),
                };
                if correct {
                    RwLockUpgradableReadGuard::upgrade(q)
                        .data
                        .push(Event::Entity(EntityEvent::Removal(e.id)));
                    return;
                }
            }
        }
    }
}
