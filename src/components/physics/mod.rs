//! Collider types:
//! - Box
//! - Sphere
//! - Capsule
//!
//! ```
//! let mut entity = Entity::new();
//! entity.addComponent::<Transform>();
//! entity.addComponent::<Physics::Box>();
//!
//! ```

use std::cell::OnceCell;

use crate::{
    self as lunar_engine,
    ecs::{ComponentReference, SelfReferenceGuard},
};

use lunar_engine_derive::dependencies;

use crate::ecs::Component;

use super::transform::Transform;
///Colliders
pub mod colliders;

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
