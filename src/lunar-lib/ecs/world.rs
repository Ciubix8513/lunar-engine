#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc};

use vec_key_value_pair::VecMap;

use super::{
    component::Component,
    entity::{self, Entity},
};

//Oh god this is gonna be a mess
#[derive(Debug, Default)]
pub(crate) struct ComponentsModified {
    modified_components: Vec<std::any::TypeId>,
    entity_modified: bool,
}

impl ComponentsModified {
    ///Sets all caches modified to false
    pub fn reset(&mut self) {
        self.modified_components.clear();
        self.entity_modified = false;
    }

    ///Must be called upon component addition or removal
    pub fn component_changed<T: Component>(&mut self) {
        self.modified_components.push(std::any::TypeId::of::<T>());
    }

    ///Must be called upon new entity creation or entity delition
    pub fn entity_changed(&mut self) {
        self.entity_modified = true;
    }
}

///Manages all the entities
pub struct World {
    entities: Vec<Rc<RefCell<Entity>>>,
    modified: Rc<RefCell<ComponentsModified>>,
    //Gotta box it, this is so stupid
    component_cache: RefCell<VecMap<std::any::TypeId, Box<dyn std::any::Any>>>,
    entity_cache: RefCell<VecMap<std::any::TypeId, Box<dyn std::any::Any>>>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            modified: Rc::new(RefCell::new(ComponentsModified::default())),
            component_cache: RefCell::new(VecMap::new()),
            entity_cache: RefCell::new(VecMap::new()),
        }
    }
}
#[derive(Debug)]
pub enum Error {
    EntityDoesNotExist,
}

impl World {
    ///Creates a new World
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    ///Adds entity to the world, consuming it in the process
    pub fn add_entity(&mut self, entity: Entity) {
        let mut e = entity;
        e.world_modified = Some(self.modified.clone());

        self.entities.push(Rc::new(RefCell::new(e)));

        (*self.modified).borrow_mut().entity_changed();
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
            (*self.modified).borrow_mut().entity_changed();

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
            (*self.modified).borrow_mut().entity_changed();
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
    ///Checks the modified data and deletes all modified caches;
    fn upate_caches(&self) {
        let mut modified = (*self.modified).borrow_mut();
        if modified.entity_modified {
            modified.reset();
            self.component_cache.borrow_mut().clear();
            self.entity_cache.borrow_mut().clear();
            return;
        }
        let mut c_cache = self.component_cache.borrow_mut();
        let mut e_cache = self.entity_cache.borrow_mut();
        if !modified.modified_components.is_empty() {
            //Remove caches for all modified components
            for i in &modified.modified_components {
                c_cache.remove(i);
                e_cache.remove(i);
            }
            modified.reset();
        }
    }

    /// Returns a vector of all components of type T
    /// The vector may be empty if there are no entities that have component T
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get_all_components<T>(&self) -> Option<Vec<entity::ComponentReference<T>>>
    where
        T: 'static + Component,
    {
        self.upate_caches();

        let mut binding = self.component_cache.borrow_mut();
        let entry = binding
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| {
                Box::new(
                    self.entities
                        .iter()
                        .filter_map(|e| e.borrow().get_component::<T>())
                        .collect::<Vec<_>>(),
                )
            });

        let vec = entry
            .downcast_ref::<Vec<entity::ComponentReference<T>>>()
            .unwrap();

        if vec.len() > 1 {
            Some((*vec).clone())
        } else {
            None
        }
    }

    /// Returns a vector of all components of type T
    /// The vector may be empty if there are no entities that have component T
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get_all_entities_with_component<T>(&self) -> Option<Vec<Rc<RefCell<Entity>>>>
    where
        T: 'static + Component,
    {
        self.upate_caches();

        let mut entry = self.entity_cache.borrow_mut();
        let entry = entry.entry(std::any::TypeId::of::<T>()).or_insert_with(|| {
            Box::new(
                self.entities
                    .iter()
                    .filter(|e| e.borrow().has_component::<T>())
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        });
        let vec = entry.downcast_ref::<Vec<Rc<RefCell<Entity>>>>().unwrap();
        if vec.len() > 1 {
            Some(vec.clone())
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

        for _ in 0..200 {
            let mut e = Entity::new();
            e.add_component::<Transform>().unwrap();
            w.add_entity(e);
        }

        //To test speed
        for _ in 0..10000 {
            let o = w.get_all_components::<Transform>();
            assert!(o.is_some());
            assert_eq!(o.unwrap().len(), 200);

            let o = w.get_all_entities_with_component::<Transform>();
            assert!(o.is_some());
            assert_eq!(o.unwrap().len(), 200);
            // w.modified.borrow_mut().entity_changed();
        }

        let mut e = Entity::new();
        e.add_component::<Transform>().unwrap();
        w.add_entity(e);

        //Test cache invalidation for components
        let o = w.get_all_components::<Transform>();
        assert!(o.is_some());
        assert_eq!(o.unwrap().len(), 201);

        //Test cache invalidation for components
        let o = w.get_all_entities_with_component::<Transform>();
        assert!(o.is_some());
        let o = o.unwrap();
        assert_eq!(o.len(), 201);

        let mut e = o.get(0).unwrap().borrow_mut();
        e.remove_component::<Transform>().unwrap();
        drop(e);

        let o = w.get_all_components::<Transform>();
        assert!(o.is_some());
        assert_eq!(o.unwrap().len(), 200);

        let o = w.get_all_entities_with_component::<Transform>();
        assert!(o.is_some());
        assert_eq!(o.unwrap().len(), 200);
    }
}
