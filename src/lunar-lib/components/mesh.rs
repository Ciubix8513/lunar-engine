use wgpu::util::DeviceExt;

use crate::{ecs::Component, grimoire, math::mat4x4::Mat4x4, DEVICE};

#[derive(Debug, Default)]
pub struct Mesh {
    // entity: Option<& Entity>,
    mesh: crate::structrures::model::Mesh,
    transform_uniform: Option<wgpu::Buffer>,
    transform_bind_group: Option<wgpu::BindGroup>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    initialized: bool,
}

impl Component for Mesh {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Mesh::default()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

impl Mesh {
    pub fn set_model(&mut self, mesh: &crate::structrures::model::Mesh) {
        let device = DEVICE.get().unwrap();

        self.vertex_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&mesh.vertices),
            }),
        );

        self.index_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::INDEX,
                contents: bytemuck::cast_slice(&mesh.indecies),
            }),
        );

        let transform_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Transformation uniform"),
            size: std::mem::size_of::<Mat4x4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout_v =
            device.create_bind_group_layout(&grimoire::TRANSFORM_BIND_GROUP_LAYOUT_DESCRIPTOR);

        self.transform_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        }));

        self.transform_uniform = Some(transform_uniform);
        self.initialized = true;
    }
}
