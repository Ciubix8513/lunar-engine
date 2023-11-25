use crate::grimoire;

use super::{transfomation::Transformation, DEVICE};
use renderer_lib::{math::mat4x4::Mat4x4, structrures::model::Mesh};
use wgpu::util::DeviceExt;

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
}
