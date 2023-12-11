#![allow(clippy::cast_possible_truncation)]
use std::num::NonZeroU64;

use crate::grimoire;

use super::{transfomation::Transformation, DEVICE};
use bytemuck::bytes_of;
use renderer_lib::{math::mat4x4::Mat4x4, structrures::model::Mesh};
use wgpu::{util::DeviceExt, RenderPass};

#[derive(Debug)]
pub struct Model {
    pub transform: Transformation,
    pub mesh: Mesh,
    transform_uniform: wgpu::Buffer,
    transform_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Model {
    pub fn new(mesh: Mesh) -> Self {
        let device = DEVICE.get().unwrap();

        let v_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&mesh.vertices),
        });

        let i_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&mesh.indecies),
        });

        let transform_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Transformation uniform"),
            size: std::mem::size_of::<Mat4x4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout_v =
            device.create_bind_group_layout(&grimoire::TRANSFORM_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Vertex bind group"),
            layout: &bind_group_layout_v,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &transform_uniform,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            transform: Transformation::default(),
            mesh,
            transform_uniform,
            transform_bind_group,
            vertex_buffer: v_buffer,
            index_buffer: i_buffer,
        }
    }
    pub fn update_uniforms(
        &self,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let device = DEVICE.get().unwrap();
        staging_belt
            .write_buffer(
                encoder,
                &self.transform_uniform,
                0,
                NonZeroU64::new(std::mem::size_of::<Mat4x4>() as u64).unwrap(),
                device,
            )
            .copy_from_slice(bytes_of(&self.transform.matrix().transpose()));
    }

    pub fn render<'a, 'b>(&'a self, render_pass: &mut RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_bind_group(
            grimoire::TRANS_BIND_GROUP_INDEX,
            &self.transform_bind_group,
            &[],
        );
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.mesh.indecies.len() as u32, 0, 0..1);
    }
}
