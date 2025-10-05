//! The ecs module of the library
//!
//! Implements a simple ECS(like) system, heavily inspired by the Unity component system
//! implementation
#[cfg(test)]
mod tests;

///The trait all components that are used within the ECS must implement
pub trait Component: std::any::Any + Send + Sync {
    ///Creates a new instance of the component
    fn mew() -> Self
    where
        Self: Sized;
    ///Called every frame
    fn update(&mut self) {}
    ///Called after the component is created
    fn awawa(&mut self) {}
    ///Called upon component deletion
    fn decatification(&mut self) {}

    ///Called when the entity containing this component is added to a world
    ///
    ///May be used to get a weak reference to the entity for acquring other components on this
    ///entity
    ///
    ///If the entity is in a world, this function will be called when the component is added,
    ///otherwise it will be called when the entity is added to the world
    ///
    ///Please consider using [`std::cell::OnceCell`] for storing references acquired using this
    ///function
    #[allow(unused_variables)]
    fn set_self_reference(&mut self, reference: SelfReferenceGuard) {}

    #[allow(clippy::missing_errors_doc)]
    ///Checks if the specified entity contains all the dependencies of this `Component`
    ///
    ///# Returns:
    ///
    ///`Ok` if all dependencies are satisfied
    ///Name of the missing component as `&'static str`
    ///
    ///# Note
    ///
    ///This function is not meant to be implemented manually, use [`lunar_engine_derive::dependencies`]
    ///macro instead.
    #[allow(unused_variables)]
    fn check_dependencies(entity: &Entity) -> Result<(), &'static str>
    where
        Self: Sized,
    {
        Ok(())
    }

    #[allow(clippy::missing_errors_doc, unused_variables)]
    ///See [`Component::check_dependencies`]
    fn check_dependencies_instanced(&self, entity: &Entity) -> Result<(), &'static str> {
        Ok(())
    }

    ///Returns whether the component is unique or not, by default a component is not unique
    ///
    ///If a component is unique, then only one instance of that component can exist in a `World`.
    ///
    ///# Note
    ///
    ///This function is not meant to be implemented manually, use [`lunar_engine_derive::unique`]
    ///macro instead
    #[must_use]
    fn unique() -> bool
    where
        Self: Sized,
    {
        false
    }

    ///See [`Component::unique`]
    fn unique_instanced(&self) -> bool {
        false
    }
}

use lock_api::{MappedRwLockReadGuard, RawRwLock};
use parking_lot::{MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use rand::Rng;
use std::any::{Any, TypeId};
use std::cell::{Ref, RefMut};
use std::sync::{Arc, Weak};
use vec_key_value_pair::set::VecSet;

///A reference to an [Entity] in a world intended for uses with short lifetimes
pub type EntityRefence = Arc<RwLock<Entity>>;
///A weak reference to an [Entity] in a world intended for use with longer lifetimes
pub type WeakEntityRefence = Weak<RwLock<Entity>>;

///A container for components
#[derive(Default)]
pub struct Entity {
    id: UUID,
    //Store type ids separately to allow for working with components while a component is borrowed
    comoponent_types: Vec<std::any::TypeId>,
    //It makes total sense i swear, you need an RC to share the refcell and a refcell to borrow the
    //stuff, I SWEAR IT MAKES SENSE
    components: Vec<Arc<RwLock<dyn Component + 'static>>>,
    self_reference: Option<Weak<RwLock<Self>>>,
    pub(crate) world_modified: Option<Arc<RwLock<ComponentsModified>>>,
    pub(crate) unique_components: Option<Arc<RwLock<VecSet<TypeId>>>>,
}

///A guard around the reference to the entity that contains this component
#[derive(Debug, Clone)]
pub struct SelfReferenceGuard {
    weak: Weak<RwLock<Entity>>,
}

impl SelfReferenceGuard {
    ///Calls `get_component` on this entity
    ///
    ///# Errors
    ///Returns an error if the entity has been deleted or if the requested component doesn't exist
    pub fn get_component<T>(&self) -> Result<ComponentReference<T>, Error>
    where
        T: Component + 'static,
    {
        self.weak.upgrade().map_or_else(
            || Err(Error::EntityDoesNotExist),
            //The problematic borrow
            |it| {
                //Circumvent the borrow check of the RefCell and get the value even if it's already
                //mutably borrowed
                //
                //This SHOULD be fine, bc this call only happens when you add a component, or add
                //the entity to the world, so it SHOULDN'T cause any problems
                unsafe { it.data_ptr().as_ref().unwrap() }
                    .get_component::<T>()
                    .map_or_else(|| Err(Error::ComponentDoesNotExist), Ok)
            },
        )
    }
}

///ECS errors
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ///Entity does not contain the requested component
    ComponentDoesNotExist,
    ///Entity already contains the component
    ComponentAlreadyExists,
    ///Entity is not part of the world
    EntityDoesNotExist,
    ///Entity does not contain a dependency of a component
    MissingDependency(&'static str),
    ///An instance of the component already exists
    UniqueComponentExists,
}

///A wrapper around the component structure of easier access
#[derive(Debug)]
pub struct ComponentReference<T> {
    phantom: std::marker::PhantomData<T>,
    cell: Weak<RwLock<dyn Component + 'static>>,
}

//Have to use the manual implementation, so that it doesn't require T to implement clone
//bc it literally doesn't need clone
impl<T> Clone for ComponentReference<T> {
    fn clone(&self) -> Self {
        Self {
            phantom: self.phantom,
            cell: self.cell.clone(),
        }
    }
}

impl<T: 'static> ComponentReference<T> {
    ///Borrows the underlying component
    ///
    ///# Panics
    ///Will panic if the referenced component, or its entity has been dropped
    #[must_use]
    #[inline(always)]
    #[allow(clippy::ref_as_ptr, clippy::ptr_as_ptr)]
    pub fn borrow(&self) -> MappedRwLockReadGuard<'_, parking_lot::RawRwLock, T> {
        RwLockReadGuard::map(
            unsafe { self.cell.as_ptr().as_ref().unwrap().read() },
            |c| unsafe { &*(c as *const dyn Any as *const T) },
        )
    }

    ///Mutably borrows the underlying component
    ///
    ///# Panics
    ///Will panic if the referenced component, or its entity has been dropped
    #[must_use]
    #[inline(always)]
    #[allow(clippy::ref_as_ptr, clippy::ptr_as_ptr)]
    pub fn borrow_mut(&self) -> MappedRwLockWriteGuard<'_, T> {
        RwLockWriteGuard::map(
            unsafe { self.cell.as_ptr().as_ref().unwrap().write() },
            |c| unsafe { &mut *(c as *mut dyn Any as *mut T) },
        )
    }
}

impl Entity {
    ///Creates a new entity with no components
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: rand::thread_rng().r#gen(),
            ..Default::default()
        }
    }

    ///Returns internal entity id
    #[must_use]
    pub const fn get_id(&self) -> UUID {
        self.id
    }

    ///Checks if the entity has component of type T
    #[must_use]
    pub fn has_component<T: 'static>(&self) -> bool {
        for c in &self.components {
            let ptr = c.data_ptr() as *mut dyn Any;
            if unsafe { ptr.as_ref().unwrap().is::<T>() } {
                return true;
            }
        }
        false
    }

    ///Adds component of type T to the entity
    ///# Errors
    ///
    ///Returns an error if the entity already has the component of type `T`
    pub fn add_component<T: 'static + Component>(&mut self) -> Result<(), Error> {
        //Check if already have that component
        if self.has_component::<T>() {
            return Err(Error::ComponentAlreadyExists);
        }

        //Check if component is unique
        if T::unique()
            && let Some(u) = &self.unique_components
        {
            // let map = &mut u.write();
            let map = u.read();

            //Returns an error if there already is a instance of a component
            if map.contains(&TypeId::of::<T>()) {
                return Err(Error::UniqueComponentExists);
            }

            drop(map);

            u.write().insert(TypeId::of::<T>());
        }

        if let Err(e) = T::check_dependencies(self) {
            return Err(Error::MissingDependency(e));
        }

        let mut c = T::mew();
        c.awawa();

        if let Some(w) = &self.self_reference {
            c.set_self_reference(SelfReferenceGuard { weak: w.clone() });
        }

        //Add component type ID
        self.comoponent_types.push(std::any::TypeId::of::<T>());
        self.components.push(Arc::new(RwLock::new(c)));

        if let Some(w) = &self.world_modified {
            w.write().component_changed::<T>();
        }

        Ok(())
    }

    ///Removes component of type T from the entity
    ///# Errors
    ///
    ///Returns an error if the entity doesn't have the component of type `T`
    pub fn remove_component<T: 'static + Component>(&mut self) -> Result<(), Error> {
        let mut ind = None;
        for (index, c) in self.components.iter().enumerate() {
            if unsafe { (c.data_ptr() as *mut dyn Any).as_ref().unwrap().is::<T>() } {
                ind = Some(index);
                break;
            }
        }
        if let Some(ind) = ind {
            self.comoponent_types.remove(ind);
            self.components.remove(ind);

            if let Some(w) = &self.world_modified {
                w.write().component_changed::<T>();
            }

            Ok(())
        } else {
            Err(Error::ComponentDoesNotExist)
        }
    }

    ///Acquires a reference to the component of type T
    #[must_use]
    pub fn get_component<T: 'static>(&self) -> Option<ComponentReference<T>> {
        for (component, comp_type) in self.components.iter().zip(self.comoponent_types.iter()) {
            if &std::any::TypeId::of::<T>() == comp_type {
                return Some(ComponentReference {
                    cell: Arc::downgrade(component),
                    phantom: std::marker::PhantomData,
                });
            }
        }
        None
    }

    ///Performs update on all components of the entity
    pub fn update(&mut self) {
        for c in &mut self.components {
            c.write().update();
        }
    }

    ///Destroys the entity and calls decatification on all of it components
    pub fn decatify(&mut self) {
        for (i, c) in self.components.iter_mut().enumerate() {
            let mut c = c.write();

            if c.unique_instanced()
                && let Some(u) = &self.unique_components
            {
                let u = &mut u.write();
                let type_id = self.comoponent_types[i];

                u.remove(&type_id);
            }

            c.decatification();
        }
    }
}

///Builder struct for easier [Entity] creation
///
///Note: Component addition order matters when using an `EntityBuilder` to create an entity,
///dependencies must be added first
#[derive(Default)]
#[allow(clippy::module_name_repetitions)]
pub struct EntityBuilder {
    components: Vec<Arc<RwLock<dyn Component + 'static>>>,
    component_types: Vec<std::any::TypeId>,
}

impl EntityBuilder {
    ///Creates a new [Self]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    ///Creates a component of type `T` and adds is to the entity
    #[must_use]
    pub fn add_component<T>(mut self) -> Self
    where
        T: 'static + Component,
    {
        for i in &self.components {
            if unsafe { (i.data_ptr() as *mut dyn Any).as_ref().unwrap().is::<T>() } {
                return self;
            }
        }
        let c = T::mew();
        self.components.push(Arc::new(RwLock::new(c)));
        self.component_types.push(std::any::TypeId::of::<T>());

        self
    }

    ///Adds the component to the entity
    #[must_use]
    pub fn add_existing_component<T>(mut self, component: T) -> Self
    where
        T: Component + 'static,
    {
        for i in &self.components {
            if unsafe { (i.data_ptr() as *mut dyn Any).as_ref().unwrap() }.is::<T>() {
                return self;
            }
        }

        self.components.push(Arc::new(RwLock::new(component)));
        self.component_types.push(std::any::TypeId::of::<T>());

        self
    }

    ///Creates a new component, using the provided closure and adds it to the entity
    #[must_use]
    pub fn create_component<F, T>(mut self, f: F) -> Self
    where
        F: FnOnce() -> T,
        T: Component + 'static,
    {
        let c = f();

        for i in &self.components {
            if unsafe { (i.data_ptr() as *mut dyn Any).as_ref().unwrap() }.is::<T>() {
                return self;
            }
        }

        self.components.push(Arc::new(RwLock::new(c)));
        self.component_types.push(std::any::TypeId::of::<T>());

        self
    }

    ///Creates the entity
    ///
    ///# Errors
    ///May return an error if a dependency is not satisfied
    ///
    ///Note: component addition order matters in the builder, dependencies MUST be added first
    pub fn create(self) -> Result<Entity, Error> {
        let mut e = Entity {
            id: rand::thread_rng().r#gen(),
            ..Default::default()
        };

        for (component, comp_type) in self.components.into_iter().zip(self.component_types) {
            if let Err(e) = component.read().check_dependencies_instanced(&e) {
                return Err(Error::MissingDependency(e));
            }
            e.components.push(component);
            e.comoponent_types.push(comp_type);
        }

        for c in &e.components {
            c.write().awawa();
        }

        Ok(e)
    }
}

use vec_key_value_pair::map::VecMap;

use crate::UUID;

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
    pub const fn entity_changed(&mut self) {
        self.entity_modified = true;
    }
}

///Manages all the entities
pub struct World {
    entities: Vec<EntityRefence>,
    modified: Arc<RwLock<ComponentsModified>>,
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
    pub fn get_all_components<T>(&self) -> Option<Vec<ComponentReference<T>>>
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

        if vec.is_empty() {
            None
        } else {
            Some((*vec).clone())
        }
    }

    /// Returns a vector of all components of type T
    ///
    /// Will return None, if no entities are found
    #[allow(clippy::missing_panics_doc, clippy::coerce_container_to_any)]
    #[must_use]
    pub fn get_all_entities_with_component<T>(&self) -> Option<Vec<Arc<RwLock<Entity>>>>
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

        if vec.is_empty() {
            None
        } else {
            Some(vec.clone())
        }
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
