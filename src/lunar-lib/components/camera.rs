use std::num::NonZeroU64;

use proc_macros::alias;

use crate::{
    ecs::{self, Component, ComponentReference},
    grimoire::{CAMERA_BIND_GROUP_INDEX, CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR},
    math::{mat4x4::Mat4x4, vec4::Vec4},
    DEVICE, RESOLUTION, STAGING_BELT,
};

use super::transform::Transform;

#[derive(Debug, Default)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    transorm_reference: Option<ComponentReference<Transform>>,
    buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
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
            ..Default::default()
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

    pub fn initialize_gpu(&mut self) {
        let device = DEVICE.get().unwrap();
        let buf = crate::helpers::create_uniform_matrix(Some("Camera"));

        let bind_layout_group_descriptor =
            device.create_bind_group_layout(&CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera"),
            layout: &bind_layout_group_descriptor,
            entries: &[wgpu::BindGroupEntry {
                binding: CAMERA_BIND_GROUP_INDEX,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buf,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        self.buffer = Some(buf);
        self.bind_group = Some(bind_group);
    }

    pub fn update_gpu(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut staging_belt = STAGING_BELT.get().unwrap().write().unwrap();

        staging_belt
            .write_buffer(
                encoder,
                self.buffer.as_ref().unwrap(),
                0,
                NonZeroU64::new(std::mem::size_of::<Mat4x4>() as u64).unwrap(),
                DEVICE.get().unwrap(),
            )
            .copy_from_slice(bytemuck::bytes_of(&self.matrix()));
    }

    pub fn set_bindgroup<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_bind_group(
            CAMERA_BIND_GROUP_INDEX,
            self.bind_group.as_ref().unwrap(),
            &[],
        )
    }
}

#[alias(Camera)]
#[derive(Debug)]
pub struct MainCamera;
