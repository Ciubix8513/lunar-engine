pub const SCREENSHOT_DIRECTORY: &str = "./screenshots";
pub const FILE_TIME_FORMAT: &str = "%Y-%m-%d-%h-%m-%s";

pub const CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera binding"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };

pub const CAMERA_BIND_GROUP_INDEX: u32 = 0;
pub const NUM_THREADS: usize = 8;
