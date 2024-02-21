use proc_macros::alias;

use crate::{
    ecs::{self, Component, ComponentReference},
    math::{mat4x4::Mat4x4, vec4::Vec4},
    RESOLUTION,
};

use super::transform::Transform;

#[derive(Debug, Default)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    transorm_reference: Option<ComponentReference<Transform>>,
}

impl Component for Camera {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transorm_reference = Some(reference.get_component().unwrap())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

impl Camera {
    pub fn new(fov: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            near,
            far,
            transorm_reference: None,
        }
    }

    pub fn matrix(&self) -> Mat4x4 {
        let binding = self.transorm_reference.as_ref().unwrap();
        let transform = binding.borrow();
        let rotation_matrix = Mat4x4::rotation_matrix_euler(&transform.rotation);

        let up = (rotation_matrix * Vec4::new(0.0, 1.0, 0.0, 1.0)).xyz();
        let forward = (rotation_matrix * Vec4::new(0.0, 0.0, -1.0, 1.0)).xyz();

        let camera_matrix = Mat4x4::look_at_matrix(transform.position, up, forward);

        let resolution = RESOLUTION.get().unwrap().read().unwrap();
        let aspect = resolution.width as f32 / resolution.height as f32;

        let projection_matrix =
            Mat4x4::perspercive_projection(self.fov, aspect, self.near, self.far);

        camera_matrix * projection_matrix
    }
}

#[alias(Camera)]
#[derive(Debug)]
pub struct MainCamera;
