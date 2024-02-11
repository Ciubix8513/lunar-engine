#![allow(dead_code)]
use std::num::NonZeroU64;

use crate::{asset_managment::UUID, ecs::Component, math::mat4x4::Mat4x4};

#[derive(Debug, Default)]
pub struct Mesh {
    entity_id: crate::ecs::UUID,
    asset_id: Option<UUID>,
    transform_uniform: Option<wgpu::Buffer>,
    transform_bind_group: Option<wgpu::BindGroup>,
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

    fn awawa(&mut self) {
        self.gen_gpu();
    }
}

impl Mesh {
    ///Changes the asset used by the component
    ///Does not chedk if the provided id is valid
    pub fn set_mesh(&mut self, id: UUID) {
        self.asset_id = Some(id);
    }

    ///Returns asset id of the component
    ///
    ///Returns none if it is not set
    pub const fn get_mesh_id(&self) -> Option<UUID> {
        self.asset_id
    }

    ///Creates the buffers and loads data into them
    fn gen_gpu(&mut self) {
        let device = crate::DEVICE.get().unwrap();
        let label = format!("Mesh {}", self.entity_id);
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label),
            size: std::mem::size_of::<Mat4x4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&label),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        NonZeroU64::new(std::mem::size_of::<Mat4x4>() as u64).unwrap(),
                    ),
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&label),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        self.transform_bind_group = Some(bind_group);
        self.transform_uniform = Some(uniform);
    }
}
