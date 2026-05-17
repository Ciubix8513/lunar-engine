use std::{any::Any, sync::Arc};

use crate::ecs::{Component, Entity, Error};
use parking_lot::RwLock;
use rand::Rng;

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
