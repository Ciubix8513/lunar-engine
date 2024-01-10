#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc};

use super::{
    component::Component,
    entity::{self, Entity},
};

///Manages all the entities
#[derive(Default)]
pub struct World {
    entities: Vec<Rc<RefCell<Entity>>>,
}
#[derive(Debug)]
pub enum Error {
    EntityDoesNotExist,
}

impl World {
    ///Creates a new World
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    ///Adds entity to the world, consuming it in the process
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(Rc::new(RefCell::new(entity)));
    }

    ///Finds and removes the entity by its reference
    ///# Errors
    ///
    ///Returns an error if the enity doesn't exist in the world
    pub fn remove_entity_by_ref(&mut self, entity: &Entity) -> Result<(), Error> {
        let mut id = None;
        for (index, e) in self.entities.iter().enumerate() {
            if e.borrow().get_id() == entity.get_id() {
                id = Some(index);
                break;
            }
        }

        if let Some(id) = id {
            self.entities.remove(id).take().decatify();
            Ok(())
        } else {
            Err(Error::EntityDoesNotExist)
        }
    }

    ///Finds and removes the entity by its id
    ///# Errors
    ///
    ///Returns an error if the entity with the `enity_id` doesn't exist in the world
    pub fn remove_entity_by_id(&mut self, entity_id: super::entity::UUID) -> Result<(), Error> {
        let mut id = None;
        for (index, e) in self.entities.iter().enumerate() {
            if e.borrow().get_id() == entity_id {
                id = Some(index);
                break;
            }
        }

        if let Some(id) = id {
            self.entities.remove(id).take().decatify();
            Ok(())
        } else {
            Err(Error::EntityDoesNotExist)
        }
    }

    ///Returns the total number of entities
    ///# Errors
    ///
    ///Returns an error if the entity with a given id doesn't exist
    #[must_use]
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    #[must_use]
    pub fn get_entity_by_id(&self, id: super::entity::UUID) -> Option<Rc<RefCell<Entity>>> {
        self.entities
            .iter()
            .find(|e| e.borrow().get_id() == id)
            .cloned()
    }
    /// Returns a vector of all components of type T
    /// The vector may be empty if there are no entities that have component T
    #[must_use]
    pub fn get_all_components<T>(&self) -> Option<Vec<entity::ComponentReference<T>>>
    where
        T: 'static + Component,
    {
        let o = self
            .entities
            .iter()
            .filter_map(|e| e.borrow().get_component::<T>())
            .collect::<Vec<_>>();
        if o.len() > 1 {
            Some(o)
        } else {
            None
        }
    }
    /// Returns a vector of all components of type T
    /// The vector may be empty if there are no entities that have component T
    #[must_use]
    pub fn get_all_entities_with_component<T>(&self) -> Option<Vec<Rc<RefCell<Entity>>>>
    where
        T: 'static + Component,
    {
        let o = self
            .entities
            .iter()
            .filter(|e| e.borrow().has_component::<T>())
            .cloned()
            .collect::<Vec<_>>();
        if o.len() > 1 {
            Some(o)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod world_tests {
    use crate::ecs::{components::transform::Transform, entity::Entity};

    use super::World;

    #[test]
    fn add_enitity_test() {
        let e = Entity::new();
        let mut w = World::new();

        w.add_entity(e);

        assert_eq!(w.get_entity_count(), 1);
    }

    #[test]
    fn remove_enitity_test() {
        let e = Entity::new();
        let id = e.get_id();
        let mut w = World::new();

        assert!(w.remove_entity_by_id(id).is_err());

        w.add_entity(e);

        w.remove_entity_by_id(id).unwrap();

        assert_eq!(w.get_entity_count(), 0);
    }

    #[test]
    fn get_entity_test() {
        let mut e = Entity::new();
        e.add_component::<Transform>().unwrap();
        let id = e.get_id();
        let mut w = World::new();
        w.add_entity(e);

        let e = w.get_entity_by_id(id).unwrap();

        let e1 = e.borrow();
        let c = e1.get_component::<Transform>().unwrap();
        drop(e1);
        _ = c.borrow();

        w.remove_entity_by_id(id).unwrap();
    }

    #[test]
    fn get_all_componenents_test() {
        let mut w = World::new();
        let o = w.get_all_components::<Transform>();
        assert!(o.is_none());

        let mut e = Entity::new();
        e.add_component::<Transform>().unwrap();
        w.add_entity(e);
        let mut e = Entity::new();
        e.add_component::<Transform>().unwrap();
        w.add_entity(e);
        let mut e = Entity::new();
        e.add_component::<Transform>().unwrap();
        w.add_entity(e);

        let o = w.get_all_components::<Transform>();
        // let o = w.get_all_components::<(Transform, i32)>();
        assert!(o.is_some());
        assert_eq!(o.unwrap().len(), 3)
        // w.remove_entity_by_id().unwrap();
    }
}
