#![allow(dead_code)]

use crate::{
    asset_managment::UUID,
    ecs::{Component, ComponentReference},
    math::Mat4x4,
};

use super::transform::Transform;

#[derive(Debug)]
///Mesh component used for rendering
pub struct Mesh {
    visible: bool,
    mesh_id: Option<UUID>,

    material_id: Option<UUID>,
    transform_reference: Option<ComponentReference<Transform>>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            visible: true,
            mesh_id: None,
            material_id: None,
            transform_reference: None,
        }
    }
}

impl Component for Mesh {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    #[allow(unused_variables)]
    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transform_reference = Some(reference.get_component().unwrap());
    }
}

impl Mesh {
    #[must_use]
    ///Creates a new mesh with the given mesh and material ids
    pub const fn new(mesh: UUID, material: UUID) -> Self {
        Self {
            visible: true,
            mesh_id: Some(mesh),
            material_id: Some(material),
            transform_reference: None,
        }
    }
    ///Whether or not this mesh is rendered
    #[must_use]
    pub const fn get_visible(&self) -> bool {
        self.visible
    }
    ///Sets whether or not this mesh is rendered
    pub fn set_visible(&mut self, value: bool) {
        self.visible = value;
    }

    ///Changes the asset used by the component
    ///Does not chedk if the provided id is valid
    pub fn set_mesh(&mut self, id: UUID) {
        self.mesh_id = Some(id);
    }

    ///Returns asset id of the component
    ///
    ///Returns none if it is not set
    #[must_use]
    pub const fn get_mesh_id(&self) -> Option<UUID> {
        self.mesh_id
    }

    ///Changes the asset used by the component
    ///Does not check if the provided id is valid
    pub fn set_material(&mut self, id: UUID) {
        self.material_id = Some(id);
    }

    ///Returns asset id of the component
    ///
    ///Returns none if it is not set
    #[must_use]
    pub const fn get_material_id(&self) -> Option<UUID> {
        self.material_id
    }

    ///Returns a reference to the transform component
    #[must_use]
    pub fn get_transform(&self) -> ComponentReference<Transform> {
        self.transform_reference.clone().unwrap()
    }

    #[must_use]
    pub(crate) fn get_matrix(&self) -> Mat4x4 {
        self.transform_reference
            .as_ref()
            .unwrap()
            .borrow()
            .matrix_transposed()
    }
}
