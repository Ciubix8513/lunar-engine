#![allow(dead_code)]
use std::cell::RefCell;

use super::entity::Entity;

///Manages all the entities
pub struct World {
    entities: Vec<RefCell<Entity>>,
}
#[derive(Debug)]
pub enum WorldError {
    EntityDoesNotExist,
}

impl World {
    ///Creates a new World
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    ///Adds entity to the world, consuming it in the process
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(RefCell::new(entity));
    }

    ///Finds and removes the entity by its reference
    pub fn remove_entity_by_ref(&mut self, entity: &Entity) -> Result<(), WorldError> {
        let mut id = None;
        for (index, e) in self.entities.iter().enumerate() {
            if e.borrow().get_id() == entity.get_id() {
                id = Some(index);
                break;
            }
        }

        if let Some(id) = id {
            self.entities.remove(id).into_inner().decatify();
            Ok(())
        } else {
            Err(WorldError::EntityDoesNotExist)
        }
    }

    ///Finds and removes the entity by its id
    pub fn remove_entity_by_id(
        &mut self,
        entity_id: super::entity::UUID,
    ) -> Result<(), WorldError> {
        let mut id = None;
        for (index, e) in self.entities.iter().enumerate() {
            if e.borrow().get_id() == entity_id {
                id = Some(index);
                break;
            }
        }

        if let Some(id) = id {
            self.entities.remove(id).into_inner().decatify();
            Ok(())
        } else {
            Err(WorldError::EntityDoesNotExist)
        }
    }

    ///Returns the total number of entities
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn get_entity_by_id<'a>(
        &'a self,
        id: super::entity::UUID,
    ) -> Result<&'a RefCell<Entity>, WorldError> {
        for e in &self.entities {
            if e.borrow().get_id() == id {
                return Ok(e);
            }
        }
        Err(WorldError::EntityDoesNotExist)
    }
}

#[cfg(test)]
mod world_tests {
    use crate::ecs::entity::Entity;

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
        let e = Entity::new();
        let id = e.get_id();
        let mut w = World::new();
        w.add_entity(e);

        let e = w.get_entity_by_id(id).unwrap();

        let e1 = e.borrow();
        drop(e1);

        w.remove_entity_by_id(id).unwrap();
    }
}
