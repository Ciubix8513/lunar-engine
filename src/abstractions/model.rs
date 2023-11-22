use super::{transfomation::Transformation, DEVICE};
use renderer_lib::structrures::model::Mesh;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Model {
    pub transform: Transformation,
    pub mesh: Mesh,
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

        Self {
            transform: Transformation::default(),
            mesh,
            vertex_buffer: v_buffer,
            index_buffer: i_buffer,
        }
    }
}
