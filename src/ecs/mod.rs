//! The ecs module of the library
//!
//! Implements a simple ECS(like) system, heavily inspired by the Unity component system
//! implementation
#[cfg(test)]
mod tests;

///The trait all components that are used within the ECS must implement
pub trait Component: std::any::Any {
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
    #[allow(unused_variables)]
    fn set_self_reference(&mut self, reference: SelfReferenceGuard) {}

    //Will not be needed after stabilization of
    //Cannot be implemented automatically, well... likely can be, but i can't be bothered
    ///Converts trait object to a `std::any::Any` reference
    ///
    ///Please use [`lunar_engine_derive::as_any`] to implement this function automatically.
    ///Alternatively this function should be implemented as follows
    ///```
    /// # use lunar_engine::ecs::Component;
    /// # struct A;
    /// # impl Component for A {
    /// # fn mew() -> Self { Self }
    /// fn as_any(&self) -> &dyn std::any::Any {
    ///     self as &dyn std::any::Any
    /// }
    /// # fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    /// #   self as &mut dyn std::any::Any
    /// # }
    /// # }
    ///```
    fn as_any(&self) -> &dyn std::any::Any;
    ///Converts trait object to a mutable `std::any::Any` reference
    ///
    ///Please use [`lunar_engine_derive::as_any`] to implement this function automatically.
    ///Alternatively this function should be implemented as follows
    ///```
    /// # use lunar_engine::ecs::Component;
    /// # struct A;
    /// # impl Component for A {
    /// # fn mew() -> Self { Self }
    /// # fn as_any(&self) -> &dyn std::any::Any {
    /// #    self as &dyn std::any::Any
    /// # }
    /// fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    ///     self as &mut dyn std::any::Any
    /// }
    /// # }
    ///```
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

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
}

use rand::Rng;
use std::any::Any;
use std::cell::{Ref, RefMut};

///Id type [Entity] uses
pub type UUID = u64;

///A reference to an [Entity] in a world intended for uses with short lifetimes
pub type EntityRefence = Rc<RefCell<Entity>>;
///A weak reference to an [Entity] in a world intended for use with longer lifetimes
pub type WeakEntityRefence = Weak<RefCell<Entity>>;

///A container for components
#[derive(Default)]
pub struct Entity {
    id: UUID,
    //Store type ids separately to allow for working with components while a component is borrowed
    comoponent_types: Vec<std::any::TypeId>,
    //It makes total sense i swear, you need an RC to share the refcell and a refcell to borrow the
    //stuff, I SWEAR IT MAKES SENSE
    components: Vec<Rc<RefCell<Box<dyn Component + 'static>>>>,
    self_reference: Option<Weak<RefCell<Self>>>,
    pub(crate) world_modified: Option<Rc<RefCell<ComponentsModified>>>,
}

///A guard around the reference to the entity that contains this component
pub struct SelfReferenceGuard {
    weak: Weak<RefCell<Entity>>,
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
            |it| {
                it.borrow()
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
}

///A wrapper around the component structure of easier access
#[derive(Debug)]
pub struct ComponentReference<T> {
    phantom: std::marker::PhantomData<T>,
    cell: Weak<RefCell<Box<dyn Component + 'static>>>,
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
    pub fn borrow(&self) -> Ref<'_, T> {
        Ref::map(
            unsafe { self.cell.as_ptr().as_ref().unwrap().borrow() },
            |c| unsafe { &*(c.as_any() as *const dyn Any as *const T) },
        )
    }

    ///Mutably borrows the underlying component
    ///
    ///# Panics
    ///Will panic if the referenced component, or its entity has been dropped
    #[must_use]
    #[inline(always)]
    #[allow(clippy::ref_as_ptr, clippy::ptr_as_ptr)]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefMut::map(
            unsafe { self.cell.as_ptr().as_ref().unwrap().borrow_mut() },
            |c| unsafe { &mut *(c.as_any_mut() as *mut dyn Any as *mut T) },
        )
    }
}

impl Entity {
    ///Creates a new entity with no components
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: rand::thread_rng().gen(),
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
            if c.borrow().as_any().is::<T>() {
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
        self.components.push(Rc::new(RefCell::new(Box::new(c))));

        if let Some(w) = &self.world_modified {
            w.borrow_mut().component_changed::<T>();
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
            if c.borrow().as_any().is::<T>() {
                ind = Some(index);
                break;
            }
        }
        if let Some(ind) = ind {
            self.comoponent_types.remove(ind);
            self.components.remove(ind);

            if let Some(w) = &self.world_modified {
                w.borrow_mut().component_changed::<T>();
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
                    cell: Rc::downgrade(component),
                    phantom: std::marker::PhantomData,
                });
            }
        }
        None
    }

    ///Performs update on all components of the entity
    pub fn update(&mut self) {
        for c in &mut self.components {
            c.borrow_mut().update();
        }
    }

    ///Destroys the entity and calls decatification on all of it components
    pub fn decatify(mut self) {
        for c in &mut self.components {
            c.borrow_mut().decatification();
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
    components: Vec<Box<dyn Component>>,
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
            if i.as_any().is::<T>() {
                return self;
            }
        }
        let c = Box::new(T::mew());
        self.components.push(c);
        self.component_types.push(std::any::TypeId::of::<T>());

        self
    }

    ///Adds the component to the entity
    #[must_use]
    pub fn add_existing_component<T>(mut self, component: T) -> Self
    where
        T: Component + 'static,
    {
        let component = Box::new(component) as Box<dyn Component>;

        for i in &self.components {
            if i.as_any().type_id() == component.as_any().type_id() {
                return self;
            }
        }

        self.components.push(component);
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
        let c = Box::new(f()) as Box<dyn Component>;

        for i in &self.components {
            if i.as_any().type_id() == c.as_any().type_id() {
                return self;
            }
        }

        self.components.push(c);
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
            id: rand::thread_rng().gen(),
            ..Default::default()
        };

        for (component, comp_type) in self.components.into_iter().zip(self.component_types) {
            if let Err(e) = component.check_dependencies_instanced(&e) {
                return Err(Error::MissingDependency(e));
            }
            e.components.push(Rc::new(RefCell::new(component)));
            e.comoponent_types.push(comp_type);
        }

        for c in &e.components {
            c.borrow_mut().awawa();
        }

        Ok(e)
    }
}

use std::rc::Weak;
use std::{cell::RefCell, rc::Rc};

use vec_key_value_pair::map::VecMap;

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
    entities: Vec<EntityRefence>,
    modified: Rc<RefCell<ComponentsModified>>,
    //Gotta box it, this is so stupid
    component_cache: RefCell<VecMap<std::any::TypeId, Box<dyn std::any::Any>>>,
    entity_cache: RefCell<VecMap<std::any::TypeId, Box<dyn std::any::Any>>>,
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
            modified: Rc::new(RefCell::new(ComponentsModified::default())),
            component_cache: RefCell::new(VecMap::new()),
            entity_cache: RefCell::new(VecMap::new()),
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
            e.take().decatify();
        }
    }

    ///Adds entity to the world, consuming it in the process
    pub fn add_entity(&mut self, entity: Entity) -> WeakEntityRefence {
        let mut e = entity;
        e.world_modified = Some(self.modified.clone());

        let rc = Rc::new(RefCell::new(e));
        //Add a self reference

        rc.borrow_mut().self_reference = Some(Rc::downgrade(&rc));

        for c in &rc.borrow().components {
            c.borrow_mut().set_self_reference(SelfReferenceGuard {
                weak: Rc::downgrade(&rc),
            });
        }
        let weak = Rc::downgrade(&rc);
        self.entities.push(rc);

        (*self.modified).borrow_mut().entity_changed();

        weak
    }

    ///Finds and removes the entity by its reference
    ///# Errors
    ///
    ///Returns an error if the entity doesn't exist in the world
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
    ///Returns an error if the entity with the `entity_id` doesn't exist in the world
    pub fn remove_entity_by_id(&mut self, entity_id: UUID) -> Result<(), Error> {
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

    ///Returns the entity with the requested id
    #[must_use]
    pub fn get_entity_by_id(&self, id: UUID) -> Option<EntityRefence> {
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
    ///
    /// Will return None if no components are found
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get_all_components<T>(&self) -> Option<Vec<ComponentReference<T>>>
    where
        T: 'static + Component,
    {
        self.upate_caches();

        let mut binding = self.component_cache.borrow_mut();
        let entry = binding
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| {
                log::warn!("Cache miss");
                Box::new(
                    self.entities
                        .iter()
                        .filter_map(|e| e.borrow().get_component::<T>())
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
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get_all_entities_with_component<T>(&self) -> Option<Vec<Rc<RefCell<Entity>>>>
    where
        T: 'static + Component,
    {
        self.upate_caches();

        let mut entry = self.entity_cache.borrow_mut();
        let entry = entry.entry(std::any::TypeId::of::<T>()).or_insert_with(|| {
            log::warn!("Cache miss");
            Box::new(
                self.entities
                    .iter()
                    .filter(|e| e.borrow().has_component::<T>())
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        });
        let vec = entry.downcast_ref::<Vec<Rc<RefCell<Entity>>>>().unwrap();

        if vec.is_empty() {
            None
        } else {
            Some(vec.clone())
        }
    }

    ///Calls update on all containing entities
    pub fn update(&self) {
        for e in &self.entities {
            e.borrow_mut().update();
        }
    }
}
