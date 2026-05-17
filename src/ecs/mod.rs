//! The ecs module of the library
//!
//! Implements a simple ECS(like) system, heavily inspired by the Unity component system
//! implementation
mod entity;
mod entity_builder;
#[cfg(test)]
mod tests;
mod world;

pub use entity::Entity;
pub use entity_builder::EntityBuilder;
pub use world::World;

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

use lock_api::MappedRwLockReadGuard;
use parking_lot::{MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use rand::Rng;
use std::any::{Any, TypeId};
use std::sync::{Arc, Weak};

///A reference to an [Entity] in a world intended for uses with short lifetimes
pub type EntityRefence = Arc<RwLock<Entity>>;
///A weak reference to an [Entity] in a world intended for use with longer lifetimes
pub type WeakEntityRefence = Weak<RwLock<Entity>>;

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

    ///Calls `has_component` on the entity
    pub fn has_component<T: 'static>(&self) -> bool {
        self.weak.upgrade().unwrap().read().has_component::<T>()
    }

    ///Returns the uuid of the entity
    pub fn get_id(&self) -> UUID {
        self.weak.upgrade().unwrap().read().get_id()
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
