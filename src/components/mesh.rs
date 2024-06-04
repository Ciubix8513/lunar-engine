#![allow(dead_code)]

use crate::{
    asset_managment::UUID,
    ecs::{Component, ComponentReference},
    math::mat4x4::Mat4x4,
};

use super::transform::Transform;

#[derive(Debug, Default)]
pub struct Mesh {
    mesh_id: Option<UUID>,
    material_id: Option<UUID>,
    transform_reference: Option<ComponentReference<Transform>>,
}

impl Component for Mesh {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    #[allow(unused_variables)]
    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transform_reference = Some(reference.get_component().unwrap());
    }
}

impl Mesh {
    #[must_use]
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
    pub fn get_matrix(&self) -> Mat4x4 {
        self.transform_reference
            .as_ref()
            .unwrap()
            .borrow()
            .matrix()
            .transpose()
    }
}
