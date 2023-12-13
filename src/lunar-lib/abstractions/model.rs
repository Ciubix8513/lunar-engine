#![allow(clippy::cast_possible_truncation)]
use std::num::NonZeroU64;

// use crate::grimoire;

use super::DEVICE;
use crate::{grimoire, math::mat4x4::Mat4x4, structrures::model::Mesh};
use bytemuck::bytes_of;
use wgpu::{util::DeviceExt, RenderPass};

#[derive(Debug)]
pub struct Model {
    pub mesh: Mesh,
    transform_uniform: wgpu::Buffer,
    transform_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Model {
    pub fn new(mesh: Mesh) -> Self {
        Self {
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
