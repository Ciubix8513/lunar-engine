//! Collider types:
//! - Box
//! - Sphere
//! - Capsule
//!
//! ```
//! let mut entity = Entity::new();
//! entity.add_component::<Transform>();
//! entity.add_existing_component(Collider:new(Shapes::Sphere { radius: 2.0 }));
//!
//! ```

use std::cell::OnceCell;

use crate::{
    self as lunar_engine,
    ecs::{ComponentReference, SelfReferenceGuard},
    math::Vec3,
};

use lunar_engine_derive::dependencies;

use crate::ecs::Component;

use super::transform::Transform;

///Designates this entity as a physics object
pub struct PhysObject {
    entity: OnceCell<SelfReferenceGuard>,
    transform: OnceCell<ComponentReference<Transform>>,
}

impl Component for PhysObject {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            entity: OnceCell::new(),
            transform: OnceCell::new(),
        }
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.entity.set(reference.clone()).unwrap();

        self.transform
            .set(reference.get_component().unwrap())
            .unwrap();
    }
}

///A collider used for physics simulation
pub struct Collider {
    ///Transform component of the collider
    pub transform: OnceCell<ComponentReference<Transform>>,
    ///Shape of the collider
    pub shape: Shape,
}

impl Component for Collider {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            transform: OnceCell::new(),
            shape: Shape::Sphere { radius: 1.0 },
        }
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transform
            .set(reference.get_component().unwrap())
            .unwrap()
    }
}

impl Collider {
    ///Creates a new collider with a given shape
    pub fn new(shape: Shape) -> Self {
        Self {
            transform: OnceCell::new(),
            shape,
        }
    }
}

#[non_exhaustive]
///Shapes for colliders
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum Shape {
    Box { dimensions: Vec3 },
    Sphere { radius: f32 },
    Capsule,
}
