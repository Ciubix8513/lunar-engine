#![allow(dead_code, clippy::missing_panics_doc)]

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use rand::Rng;

use super::component::Component;
pub type UUID = u64;

#[derive(Default, Debug)]
pub struct Entity {
    id: UUID,
    components: Vec<Rc<RefCell<Box<dyn Component + 'static>>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentError {
    ComponentDoesNotExist,
    ComponentAlreadyExists,
}

///A wrapper around the component structure of easier access
pub struct ComponentReference<T> {
    phantom: std::marker::PhantomData<T>,
    cell: Rc<RefCell<Box<dyn Component + 'static>>>,
}

impl<T: 'static> ComponentReference<T> {
    ///Borrows the underlying component
    #[must_use]
    pub fn borrow(&self) -> Ref<'_, T> {
        Ref::map(self.cell.borrow(), |c| {
            c.as_any().downcast_ref::<T>().unwrap()
        })
    }

    ///Mutably borrows the underlying component
    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefMut::map(self.cell.borrow_mut(), |c| {
            c.as_any_mut().downcast_mut::<T>().unwrap()
        })
    }
}

impl Entity {
    ///Creates a new entity with no components
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: rand::thread_rng().gen(),
            components: Vec::new(),
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
            let c = c.borrow();
            let any = c.as_any().downcast_ref::<T>();
            if any.is_some() {
                return true;
            }
        }
        false
    }

    ///Adds component of type T to the entity
    ///# Errors
    ///
    ///Returns an error if the entity already has the component of type `T`
    pub fn add_component<T: 'static + Component>(&mut self) -> Result<(), ComponentError> {
        //Check if already have that component
        if self.has_component::<T>() {
            return Err(ComponentError::ComponentAlreadyExists);
        }
        let mut c = T::mew();
        c.awawa();
        self.components
            .push(Rc::new(RefCell::new(Box::new(T::mew()))));

        Ok(())
    }

    ///Removes component of type T from the entity
    ///# Errors
    ///
    ///Returns an error if the entity doesn't have the component of type `T`
    pub fn remove_component<T: 'static + Component>(&mut self) -> Result<(), ComponentError> {
        let mut ind = None;
        for (index, c) in self.components.iter().enumerate() {
            let binding = c.borrow();
            let any = binding.as_any().downcast_ref::<T>();
            if any.is_some() {
                ind = Some(index);
                break;
            }
        }
        if let Some(ind) = ind {
            self.components.remove(ind);
            Ok(())
        } else {
            Err(ComponentError::ComponentDoesNotExist)
        }
    }

    ///Acquires a reference to the component of type T
    #[must_use]
    pub fn get_component<T: 'static>(&self) -> Option<ComponentReference<T>> {
        for c in &self.components {
            let binding = c.borrow();
            if binding.as_any().downcast_ref::<T>().is_some() {
                return Some(ComponentReference {
                    cell: c.clone(),
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

#[cfg(test)]
mod entity_tests {
    use crate::ecs::components::transform::Transform;

    use super::*;

    #[derive(Debug)]
    struct TestComponent {
        pub value: i32,
    }

    impl Component for TestComponent {
        fn mew() -> Self
        where
            Self: Sized,
        {
            Self { value: 0 }
        }

        fn update(&mut self) {
            self.value += 10;
        }

        fn awawa(&mut self) {}
        fn decatification(&mut self) {}

        fn as_any(&self) -> &dyn std::any::Any {
            self as &dyn std::any::Any
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self as &mut dyn std::any::Any
        }
    }

    #[test]
    fn component_add_test() {
        let mut entity = Entity::new();

        assert!(!entity.has_component::<crate::ecs::components::transform::Transform>());

        let res = entity.add_component::<Transform>();

        assert_eq!(res, Ok(()));
        assert!(entity.has_component::<crate::ecs::components::transform::Transform>());
    }

    #[test]
    fn component_remove_test() {
        let mut entity = Entity::new();

        entity.add_component::<Transform>().unwrap();
        let e = entity.remove_component::<Transform>();

        assert_eq!(e, Ok(()));
        assert!(!entity.has_component::<crate::ecs::components::transform::Transform>());
    }

    #[test]
    fn get_component_test() {
        let mut entity = Entity::new();
        entity.add_component::<Transform>().unwrap();
        let c = entity.get_component::<Transform>();
        assert!(c.is_some());
        entity.remove_component::<Transform>().unwrap();

        let c = entity.get_component::<Transform>();
        assert!(c.is_none());
    }

    #[test]
    fn component_update_test() {
        let mut entity = Entity::new();

        entity.add_component::<TestComponent>().unwrap();
        entity.update();

        let c = entity.get_component::<TestComponent>().unwrap();
        let c = c.borrow();
        assert_eq!(c.value, 10)
    }

    #[test]
    fn component_decatification_test() {
        let mut entity = Entity::new();

        entity.add_component::<TestComponent>().unwrap();
        entity.update();

        let c = entity.get_component::<TestComponent>().unwrap();
        assert_eq!(c.borrow().value, 10);

        entity.decatify();
    }
}
