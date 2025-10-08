//! Collider types: - Box
//! - Sphere
//! - Capsule
use std::sync::OnceLock;

use crate::{
    self as lunar_engine, UUID,
    ecs::{ComponentReference, SelfReferenceGuard},
    math::Vec3,
};

use lunar_engine_derive::dependencies;

use crate::ecs::Component;

use super::transform::Transform;

///Designates this entity as a physics object
pub struct PhysObject {
    entity: OnceLock<SelfReferenceGuard>,
    transform: OnceLock<ComponentReference<Transform>>,
    uuid: UUID,
}

impl Component for PhysObject {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            entity: OnceLock::new(),
            transform: OnceLock::new(),
            uuid: 0,
        }
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.entity.set(reference.clone()).unwrap();

        self.transform
            .set(reference.get_component().unwrap())
            .unwrap();

        self.uuid = reference.get_id();
    }
}

impl PhysObject {
    ///Returns a reference to the transform component on this entity
    pub fn transform(&self) -> ComponentReference<Transform> {
        self.transform.get().unwrap().clone()
    }

    ///Returns the id of the entity
    pub fn get_id(&self) -> UUID {
        self.uuid
    }
}

///A collider used for physics simulation
pub struct Collider {
    ///Transform component of the collider
    pub transform: OnceLock<ComponentReference<Transform>>,
    ///Shape of the collider
    pub shape: Shape,
    ///The material of this collider
    pub material: PhysMaterial,

    uuid: UUID,
}

impl Component for Collider {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            transform: OnceLock::new(),
            shape: Shape::Sphere { radius: 1.0 },
            uuid: 0,
            material: PhysMaterial::default(),
        }
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transform
            .set(reference.get_component().unwrap())
            .unwrap();
        self.uuid = reference.get_id();
    }
}

impl Collider {
    ///Creates a new collider with a given shape
    pub fn new(shape: Shape) -> Self {
        Self {
            transform: OnceLock::new(),
            shape,
            uuid: 0,
            material: PhysMaterial::default(),
        }
    }

    ///Returns a reference to the transform component on this entity
    pub fn transform(&self) -> ComponentReference<Transform> {
        self.transform.get().unwrap().clone()
    }

    ///Returns the id of the entity
    pub fn get_id(&self) -> UUID {
        self.uuid
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

#[derive(Clone, Copy, Debug)]
///Describes the properties of a collider
pub struct PhysMaterial {
    pub(crate) friction: f32,
    pub(crate) bounciness: f32,
}

impl PhysMaterial {
    ///Creates a new physics  material
    ///
    ///# Panics
    ///
    ///will panic if the values are smaller than 0 or bigger than 1
    pub fn new(friction: f32, bounciness: f32) -> Self {
        assert!(friction <= 1.0 && friction >= 0.0);
        assert!(bounciness <= 1.0 && bounciness >= 0.0);
        Self {
            friction,
            bounciness,
        }
    }

    ///Returns the friction coefficient of the material
    pub fn friction(&self) -> f32 {
        self.friction
    }

    ///Returns the bounciness of the material
    pub fn bounciness(&self) -> f32 {
        self.bounciness
    }

    ///Sets the friction coefficient of the material
    ///# Panics
    ///
    ///will panic if the value is smaller than 0 or bigger than 1
    pub fn set_friction(&mut self, friction: f32) {
        assert!(friction <= 1.0 && friction >= 0.0);
        self.friction = friction;
    }

    ///Sets the bounciness of the material
    ///# Panics
    ///
    ///will panic if the value is smaller than 0 or bigger than 1
    pub fn set_bounciness(&mut self, bounciness: f32) {
        assert!(bounciness <= 1.0 && bounciness >= 0.0);
        self.bounciness = bounciness;
    }
}

impl Default for PhysMaterial {
    fn default() -> Self {
        Self {
            friction: 0.5,
            bounciness: 0.5,
        }
    }
}
