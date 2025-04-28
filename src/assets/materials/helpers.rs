use wgpu::VertexBufferLayout;

use crate::internal::DEVICE;

///Returns the default vertex buffer bindings
#[must_use]
pub const fn vertex_binding() -> [VertexBufferLayout<'static>; 2] {
    [
        //Vertex data
        wgpu::VertexBufferLayout {
            array_stride: 32,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                //UV
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                //Normals
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
            ],
        },
        //Transform data
        //Encoding a matrix as 4 vec4
        //Just that i can do instanced rendering
        wgpu::VertexBufferLayout {
            array_stride: 64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 6,
                },
            ],
        },
    ]
}

#[must_use]
///Returns whether or not storage buffers are available on the current device
pub fn storage_buffer_available() -> bool {
    let features = DEVICE.get().unwrap().features();

    features.contains(wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY)
}
