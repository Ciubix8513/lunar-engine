use lunar_engine_derive::as_any;

use crate::math::{Mat4x4, Vec3};

use crate::ecs::{Component, ComponentReference};

///Transform  component contains function and data to determine the position of the entity
///
///Note: rotation is represented as Euler angles using degrees
#[derive(Debug)]
pub struct Transform {
    ///Position of the object
    pub position: Vec3,
    ///Rotation of the object
    pub rotation: Vec3,
    ///Scale of the object
    pub scale: Vec3,
    ///Parent transform of the object
    pub parent: Option<ComponentReference<Self>>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::default(),
            rotation: Vec3::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            parent: None,
        }
    }
}

impl Component for Transform {
    #[as_any]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            rotation: Vec3::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            position: Vec3::default(),
            parent: None,
        }
    }
}

impl Transform {
    ///Create a new transform instance
    #[must_use]
    pub const fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            parent: None,
        }
    }

    ///Creates a new transform instance, with a parent
    #[must_use]
    pub const fn with_parent(
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        parent: ComponentReference<Self>,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
            parent: Some(parent),
        }
    }

    ///Returns transformation of the entity taking transform of the parent into account
    #[must_use]
    pub fn matrix(&self) -> Mat4x4 {
        self.parent.as_ref().map_or_else(
            || Mat4x4::transform_matrix_euler(&self.position, &self.scale, &self.rotation),
            |p| {
                let parent_mat = p.borrow().matrix();
                parent_mat
                    * Mat4x4::transform_matrix_euler(&self.position, &self.scale, &self.rotation)
            },
        )
    }

    ///Returns transformation of the entity taking transform of the parent into account, this
    ///matrix is transposed
    #[must_use]
    pub fn matrix_transposed(&self) -> Mat4x4 {
        self.parent.as_ref().map_or_else(
            || {
                Mat4x4::transform_matrix_euler_transposed(
                    &self.position,
                    &self.scale,
                    &self.rotation,
                )
            },
            |p| {
                let parent_mat = p.borrow().matrix_transposed();
                Mat4x4::transform_matrix_euler_transposed(
                    &self.position,
                    &self.scale,
                    &self.rotation,
                ) * parent_mat
            },
        )
    }
    ///Returns transformation matrix of the entity, without taking the parent transformation into
    ///account, this matrix is transposed
    #[must_use]
    pub fn matrix_local_transposed(&self) -> Mat4x4 {
        Mat4x4::transform_matrix_euler_transposed(&self.position, &self.scale, &self.rotation)
    }

    ///Returns transformation matrix of the entity, without taking the parent transformation into
    ///account
    #[must_use]
    pub fn matrix_local(&self) -> Mat4x4 {
        Mat4x4::transform_matrix_euler(&self.position, &self.scale, &self.rotation)
    }

    ///Sets the parent of the entity, applying all parent transformations to this entity
    pub fn set_parent(mut self, p: ComponentReference<Self>) {
        self.parent = Some(p);
    }
}
