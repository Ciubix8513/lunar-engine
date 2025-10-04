use std::cell::OnceCell;

use crate::math::{Mat4x4, Quaternion, Vec3};

use crate::ecs::{Component, ComponentReference, SelfReferenceGuard};

///Transform  component contains function and data to determine the position of the entity
///
///Note: rotation is represented as Euler angles using degrees
#[derive(Debug)]
pub struct Transform {
    ///Position of the object
    pub position: Vec3,
    ///Rotation of the object
    pub rotation: Quaternion,
    ///Scale of the object
    pub scale: Vec3,
    ///Parent transform of the object
    parent: Option<ComponentReference<Self>>,
    children: Vec<ComponentReference<Self>>,
    self_reference: OnceCell<SelfReferenceGuard>,
    self_comp_ref: OnceCell<ComponentReference<Transform>>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::default(),
            rotation: Quaternion::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            parent: None,
            children: Vec::new(),
            self_reference: OnceCell::new(),
            self_comp_ref: OnceCell::new(),
        }
    }
}

impl Component for Transform {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            rotation: Quaternion::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            position: Vec3::default(),
            parent: None,
            children: Vec::new(),
            self_reference: OnceCell::new(),
            self_comp_ref: OnceCell::new(),
        }
    }

    fn set_self_reference(&mut self, reference: SelfReferenceGuard) {
        let c_ref = reference.get_component().unwrap();

        if let Some(p) = self.parent.clone() {
            p.borrow_mut().add_child(c_ref.clone());
        }

        self.self_comp_ref.set(c_ref).unwrap();
        self.self_reference.set(reference).unwrap();
    }
}

impl Transform {
    ///Create a new transform instance
    #[must_use]
    pub const fn new(position: Vec3, rotation: Quaternion, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            parent: None,
            children: Vec::new(),
            self_reference: OnceCell::new(),
            self_comp_ref: OnceCell::new(),
        }
    }

    ///Creates a new transform instance, with a parent
    #[must_use]
    pub const fn with_parent(
        position: Vec3,
        rotation: Quaternion,
        scale: Vec3,
        parent: ComponentReference<Self>,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
            parent: Some(parent),
            children: Vec::new(),
            self_reference: OnceCell::new(),
            self_comp_ref: OnceCell::new(),
        }
    }

    fn add_child(&mut self, child: ComponentReference<Self>) {
        self.children.push(child);
    }

    ///Returns the children of this object
    pub fn get_children(&self) -> &[ComponentReference<Self>] {
        &self.children
    }

    ///Returns the parent of this object
    pub fn get_parent(&self) -> Option<ComponentReference<Self>> {
        self.parent.clone()
    }

    ///Returns a reference to the entity this component is on
    pub fn enity(&self) -> SelfReferenceGuard {
        self.self_reference.get().unwrap().clone()
    }

    ///Rotates the object using a given rotation
    pub fn rotate_quat(&mut self, rotation: Quaternion) {
        self.rotation *= rotation;
    }

    ///Rotates the object using a given rotation in euler angles
    pub fn rotate(&mut self, rotation: Vec3) {
        self.rotation *= Quaternion::from_euler(rotation);
    }

    ///Returns transformation of the entity taking transform of the parent into account
    #[must_use]
    pub fn matrix(&self) -> Mat4x4 {
        self.parent.as_ref().map_or_else(
            || Mat4x4::transform_matrix(self.position, self.scale, self.rotation),
            |p| {
                let parent_mat = p.borrow().matrix();
                parent_mat * Mat4x4::transform_matrix(self.position, self.scale, self.rotation)
            },
        )
    }

    ///Returns transformation of the entity taking transform of the parent into account, this
    ///matrix is transposed
    #[must_use]
    pub fn matrix_transposed(&self) -> Mat4x4 {
        self.parent.as_ref().map_or_else(
            || Mat4x4::transform_matrix_transposed(self.position, self.scale, self.rotation),
            |p| {
                let parent_mat = p.borrow().matrix_transposed();
                Mat4x4::transform_matrix_transposed(self.position, self.scale, self.rotation)
                    * parent_mat
            },
        )
    }
    ///Returns transformation matrix of the entity, without taking the parent transformation into
    ///account, this matrix is transposed
    #[must_use]
    pub fn matrix_local_transposed(&self) -> Mat4x4 {
        Mat4x4::transform_matrix_transposed(self.position, self.scale, self.rotation)
    }

    ///Returns transformation matrix of the entity, without taking the parent transformation into
    ///account
    #[must_use]
    pub fn matrix_local(&self) -> Mat4x4 {
        Mat4x4::transform_matrix(self.position, self.scale, self.rotation)
    }

    ///Sets the parent of the entity, applying all parent transformations to this entity
    pub fn set_parent(&mut self, p: ComponentReference<Self>) {
        if let Some(s) = self.self_comp_ref.get() {
            p.borrow_mut().add_child(s.clone());
        }

        self.parent = Some(p);
    }

    ///Returns global position of the entity
    #[must_use]
    pub fn position_global(&self) -> Vec3 {
        self.parent.as_ref().map_or(self.position, |p| {
            p.borrow().matrix().transform3(self.position)
        })
    }

    ///Returns global rotation of the entity
    //hope this  works
    #[must_use]
    pub fn rotation_global(&self) -> Quaternion {
        self.parent.as_ref().map_or(self.rotation, |p| {
            p.borrow().rotation_global() * self.rotation
        })
    }

    #[must_use]
    ///Returns global scale of the entity
    pub fn scale_global(&self) -> Vec3 {
        self.parent
            .as_ref()
            .map_or(self.scale, |p| p.borrow().scale_global() * self.scale)
    }
}
