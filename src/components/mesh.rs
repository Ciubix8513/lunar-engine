#![allow(dead_code)]

use lunar_engine_derive::as_any;

use crate::{
    asset_managment::UUID,
    ecs::{Component, ComponentReference},
    math::{Mat4x4, Vec3},
};

use super::transform::Transform;

#[derive(Debug, Default)]
///Mesh component used for rendering
pub struct Mesh {
    mesh_id: Option<UUID>,
    material_id: Option<UUID>,
    transform_reference: Option<ComponentReference<Transform>>,
}

impl Component for Mesh {
    #[as_any]

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
            mesh_id: Some(mesh),
            material_id: Some(material),
            transform_reference: None,
        }
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

    #[must_use]
    pub(crate) fn get_position(&self) -> Vec3 {
        self.transform_reference.as_ref().unwrap().borrow().position
    }

    #[must_use]
    pub(crate) fn get_matrix(&self) -> Mat4x4 {
        self.transform_reference
            .as_ref()
            .unwrap()
            .borrow()
            .matrix()
            .transpose()
    }
}
