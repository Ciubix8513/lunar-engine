use crate as lunar_engine;

use std::cell::OnceCell;

use lunar_engine_derive::dependencies;

use crate::{
    ecs::{Component, ComponentReference},
    math::Vec3,
};

use crate::components::transform::Transform;

///Box collider
pub struct Box {
    pub(crate) transform: OnceCell<ComponentReference<Transform>>,
    ///Dimensions of the box
    pub dimensions: Vec3,
}

///Sphere collider
pub struct Sphere {
    pub(crate) transform: OnceCell<ComponentReference<Transform>>,
    ///Radius of the sphere
    pub radius: f32,
}

///Capsule collider
pub struct Capsule {
    pub(crate) transform: OnceCell<ComponentReference<Transform>>,
}

impl Component for Sphere {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            transform: OnceCell::new(),
            radius: 0.5,
        }
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transform
            .set(reference.get_component().unwrap())
            .unwrap();
    }
}

impl Component for Capsule {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            transform: OnceCell::new(),
        }
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transform
            .set(reference.get_component().unwrap())
            .unwrap();
    }
}

impl Component for Box {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Box {
            transform: OnceCell::new(),
            dimensions: 1.into(),
        }
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transform
            .set(reference.get_component().unwrap())
            .unwrap();
    }
}
