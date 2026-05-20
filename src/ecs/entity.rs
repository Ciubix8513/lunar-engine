use std::{
    any::{Any, TypeId},
    sync::{Arc, Weak},
};

use crate::{
    UUID,
    ecs::{Component, ComponentReference, ComponentsModified, Error, SelfReferenceGuard},
};
use parking_lot::RwLock;
use rand::Rng;
use vec_key_value_pair::set::VecSet;

///A container for components
#[derive(Default)]
pub struct Entity {
    pub(super) id: UUID,
    //Store type ids separately to allow for working with components while a component is borrowed
    pub(super) comoponent_types: Vec<std::any::TypeId>,
    //It makes total sense i swear, you need an RC to share the refcell and a refcell to borrow the
    //stuff, I SWEAR IT MAKES SENSE
    pub(super) components: Vec<Arc<RwLock<dyn Component + 'static>>>,
    pub(super) self_reference: Option<Weak<RwLock<Self>>>,
    pub(crate) world_modified: Option<Arc<RwLock<ComponentsModified>>>,
    pub(crate) unique_components: Option<Arc<RwLock<VecSet<TypeId>>>>,
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

    ///Adds component of type T to the entity, and returns a reference to that component
    ///# Errors
    ///
    ///Returns an error if the entity already has the component of type `T`
    pub fn add_component<T: 'static + Component>(
        &mut self,
    ) -> Result<ComponentReference<T>, Error> {
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
        let c = self.components.last().unwrap();

        if let Some(w) = &self.world_modified {
            w.write().component_changed::<T>();
        }

        Ok(ComponentReference {
            cell: Arc::downgrade(c),
            phantom: std::marker::PhantomData,
        })
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
