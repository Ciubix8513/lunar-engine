use std::sync::Arc;

use crate::{
    UUID,
    ecs::{
        Component, ComponentReference, ComponentsModified, Entity, EntityRefence, Error,
        SelfReferenceGuard, WeakEntityRefence,
    },
};
use parking_lot::RwLock;
use vec_key_value_pair::{map::VecMap, set::VecSet};

///Manages all the entities
pub struct World {
    entities: Vec<EntityRefence>,
    pub(crate) modified: Arc<RwLock<ComponentsModified>>,
    //Gotta box it, this is so stupid
    component_cache: RwLock<VecMap<std::any::TypeId, Box<dyn std::any::Any>>>,
    entity_cache: RwLock<VecMap<std::any::TypeId, Box<dyn std::any::Any>>>,
    unique_components: Arc<RwLock<VecSet<std::any::TypeId>>>,
}

impl Drop for World {
    fn drop(&mut self) {
        self.destroy_all();
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            modified: Arc::new(RwLock::new(ComponentsModified::default())),
            component_cache: RwLock::new(VecMap::new()),
            entity_cache: RwLock::new(VecMap::new()),
            unique_components: Arc::new(RwLock::new(VecSet::new())),
        }
    }
}

impl World {
    ///Creates a new World
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    ///Destroys all entities in the world
    pub fn destroy_all(&mut self) {
        for e in &self.entities {
            //Doubt this will work
            e.write().decatify();
        }
    }

    ///Adds entity to the world, consuming it in the process
    ///
    ///# Errors
    ///Returns an error if the entity contains an instance of a unique component that already
    ///exists in the world
    pub fn add_entity(&mut self, mut entity: Entity) -> Result<WeakEntityRefence, Error> {
        entity.world_modified = Some(self.modified.clone());
        entity.unique_components = Some(self.unique_components.clone());

        //Check every component for whether or not it's unique
        for (i, c) in entity.components.iter().enumerate() {
            if c.read().unique_instanced() {
                let u = &mut self.unique_components.write();

                if u.contains(&entity.comoponent_types[i]) {
                    return Err(Error::UniqueComponentExists);
                }

                u.insert(entity.comoponent_types[i]);
            }
        }

        let rc = Arc::new(RwLock::new(entity));
        //Add a self reference

        let weak = Arc::downgrade(&rc);

        rc.write().self_reference = Some(weak.clone());

        for c in &rc.read().components {
            c.write().set_self_reference(SelfReferenceGuard {
                weak: Arc::downgrade(&rc),
            });
        }
        self.entities.push(rc);

        (*self.modified).write().entity_changed();

        Ok(weak)
    }

    ///Finds and removes the entity by its reference
    ///# Errors
    ///
    ///Returns an error if the entity doesn't exist in the world
    pub fn remove_entity_by_ref(&mut self, entity: &Entity) -> Result<(), Error> {
        let mut id = None;

        for (index, e) in self.entities.iter().enumerate() {
            if e.read().get_id() == entity.get_id() {
                id = Some(index);
                break;
            }
        }

        if let Some(id) = id {
            Arc::into_inner(self.entities.remove(id))
                .unwrap()
                .into_inner()
                .decatify();
            (*self.modified).write().entity_changed();

            Ok(())
        } else {
            Err(Error::EntityDoesNotExist)
        }
    }

    ///Finds and removes the entity by its id
    ///# Errors
    ///
    ///Returns an error if the entity with the `entity_id` doesn't exist in the world
    pub fn remove_entity_by_id(&mut self, entity_id: UUID) -> Result<(), Error> {
        let mut id = None;
        for (index, e) in self.entities.iter().enumerate() {
            if e.read().get_id() == entity_id {
                id = Some(index);
                break;
            }
        }

        if let Some(id) = id {
            self.entities.remove(id).write().decatify();
            (*self.modified).write().entity_changed();
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
    pub const fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    ///Returns the entity with the requested id
    #[must_use]
    pub fn get_entity_by_id(&self, id: UUID) -> Option<EntityRefence> {
        self.entities
            .iter()
            .find(|e| e.read().get_id() == id)
            .cloned()
    }
    ///Checks the modified data and deletes all modified caches;
    fn upate_caches(&self) {
        let mut modified = (*self.modified).write();
        if modified.entity_modified {
            modified.reset();
            self.component_cache.write().clear();
            self.entity_cache.write().clear();
            return;
        }

        if !modified.modified_components.is_empty() {
            let mut c_cache = self.component_cache.write();
            let mut e_cache = self.entity_cache.write();
            //Remove caches for all modified components
            for i in &modified.modified_components {
                c_cache.remove(i);
                e_cache.remove(i);
            }
            modified.reset();
        }
    }

    /// Returns a vector of all components of type T
    ///
    /// Will return None if no components are found
    #[allow(clippy::missing_panics_doc, clippy::coerce_container_to_any)]
    #[must_use]
    pub fn get_all_components<T>(&self) -> Vec<ComponentReference<T>>
    where
        T: 'static + Component,
    {
        self.upate_caches();

        let mut binding = self.component_cache.write();
        let entry = binding
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| {
                log::warn!("Cache miss");
                Box::new(
                    self.entities
                        .iter()
                        .filter_map(|e| e.read().get_component::<T>())
                        .collect::<Vec<_>>(),
                )
            });

        let vec = entry.downcast_ref::<Vec<ComponentReference<T>>>().unwrap();

        vec.clone()
    }

    /// Returns a vector of all components of type T
    ///
    /// Will return None, if no entities are found
    #[allow(clippy::missing_panics_doc, clippy::coerce_container_to_any)]
    #[must_use]
    pub fn get_all_entities_with_component<T>(&self) -> Vec<Arc<RwLock<Entity>>>
    where
        T: 'static + Component,
    {
        self.upate_caches();

        let mut entry = self.entity_cache.write();
        let entry = entry.entry(std::any::TypeId::of::<T>()).or_insert_with(|| {
            log::warn!("Cache miss");
            Box::new(
                self.entities
                    .iter()
                    .filter(|e| e.read().has_component::<T>())
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        });
        let vec = entry.downcast_ref::<Vec<Arc<RwLock<Entity>>>>().unwrap();

        vec.clone()
    }

    ///Returns a reference to the unique component
    ///
    ///Always returns none if the component is not unique
    #[allow(clippy::coerce_container_to_any)]
    pub fn get_unique_component<T>(&self) -> Option<ComponentReference<T>>
    where
        T: 'static + Component,
    {
        if !T::unique() {
            return None;
        }

        self.upate_caches();

        let mut binding = self.component_cache.write();
        let entry = binding
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| {
                Box::new(
                    self.entities
                        .iter()
                        .filter_map(|e| e.read().get_component::<T>())
                        .collect::<Vec<_>>(),
                )
            });

        let vec = entry.downcast_ref::<Vec<ComponentReference<T>>>().unwrap();

        vec.first().cloned()
    }

    ///Calls update on all containing entities
    pub fn update(&self) {
        for e in &self.entities {
            e.write().update();
        }
    }
}
