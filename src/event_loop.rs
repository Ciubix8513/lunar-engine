#![allow(
    dead_code,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::too_many_lines
)]
use bytemuck::{bytes_of, Pod, Zeroable};
use renderer_lib::math::{
    complex_shit::{look_at_matrix, perspercive_projection, transform_matrix_euler},
    mat4x4::Mat4x4,
    vec3::Vec3,
};
use std::mem::size_of;
use wgpu::{util::StagingBelt, BufferSize, Extent3d, Features};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<wgpu::BindGroup>,
    buffers: Box<[wgpu::Buffer]>,
}

struct DepthStencil<'a> {
    texture: wgpu::Texture,
    descriptor: wgpu::TextureDescriptor<'a>,
}

pub struct State<'a> {
    closed: bool,
    window: Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    pipeline: Pipeline,
    v_buffer: wgpu::Buffer,
    i_buffer: wgpu::Buffer,
    ind_len: u32,
    staging_belt: wgpu::util::StagingBelt,
    depth_stencil: DepthStencil<'a>,
    frame: u64,
}
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct TransformationMatrices {
    object: Mat4x4,
    camera: Mat4x4,
    screen: Mat4x4,
}

impl<'a> State<'a> {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let window = winit::window::Window::new(event_loop).expect("Failed to create new window");

        let size = window.inner_size();
        let instance = wgpu::Instance::default();

        let surface =
            unsafe { instance.create_surface(&window) }.expect("Failed to create a surface");

        let adapter: wgpu::Adapter = futures::executor::block_on(req_adapter(
            instance,
            &wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            },
        ))
        .expect("Failed to get an adapter");

        let (device, queue): (wgpu::Device, wgpu::Queue) = futures::executor::block_on(req_device(
            &adapter,
            &wgpu::DeviceDescriptor {
                features: Features::DEPTH_CLIP_CONTROL,
                ..Default::default()
            },
        ))
        .expect("Failed to create a device and a queue");

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .last()
            .copied()
            .expect("Did not have last format");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let vert_shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/vertex.wgsl"));
        let frag_shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/color.wgsl"));

        log::info!("Loaded shaders");

        let bind_group_layout_v =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Vertex binding"),
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
            });

        let bind_group_layout_f =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Fragment binding"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        log::info!("Created bind group layouts");

        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<TransformationMatrices>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_data =
            renderer_lib::import::bmp::parse(include_bytes!("../assets/blahaj1.bmp")).unwrap();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: Extent3d {
                width: texture_data.width,
                height: texture_data.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(&texture_data.data[..]),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(texture_data.width * 4),
                rows_per_image: None,
            },
            Extent3d {
                width: texture_data.width,
                height: texture_data.height,
                depth_or_array_layers: 1,
            },
        );
        log::info!("Wrote to texture");

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let bind_group_v = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Vertex bind group"),
            layout: &bind_group_layout_v,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        let bind_group_f = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fragment bind group"),
            layout: &bind_group_layout_f,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.create_view(
                        &wgpu::TextureViewDescriptor {
                            label: None,
                            format: Some(wgpu::TextureFormat::Rgba8Unorm),
                            dimension: None,
                            aspect: wgpu::TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: Some(1),
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        log::info!("created bind group");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout_v, &bind_group_layout_f],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 36,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 16,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        // l
        // let data: [[f32; 3]; 3] = [[-0.5, -0.5, 0.0], [0.0, 0.5, 0.0], [0.5, -0.5, 0.0]];
        let binding =
            renderer_lib::import::obj::parse(include_str!("../assets/blahaj.obj")).unwrap();
        let data = binding.first().unwrap();

        let v_buffer = wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(data.vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        let i_buffer = wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(data.indecies.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        let descriptor = wgpu::TextureDescriptor {
            label: Some("Depth stencil"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        };
        let depth_stencil = device.create_texture(&descriptor);

        Self {
            closed: false,
            window,
            device,
            queue,
            surface,
            surface_config,
            pipeline: Pipeline {
                pipeline,
                bind_groups: vec![bind_group_v, bind_group_f],
                buffers: Box::new([uniform]),
            },
            v_buffer,
            i_buffer,
            staging_belt: StagingBelt::new(1024),
            ind_len: data.indecies.len() as u32,
            depth_stencil: DepthStencil {
                texture: depth_stencil,
                descriptor,
            },
            frame: 0,
        }
    }

    pub fn app_loop(&mut self, event: &Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::RedrawEventsCleared => self.window.request_redraw(),
            Event::RedrawRequested(_) => {
                if !self.closed {
                    self.render();
                }
            }
            Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    self.closed = true;
                }
                winit::event::WindowEvent::Resized(size) => {
                    self.surface_config.width = size.width;
                    self.surface_config.height = size.height;
                    self.depth_stencil.descriptor.size.width = size.width;
                    self.depth_stencil.descriptor.size.height = size.height;

                    self.surface.configure(&self.device, &self.surface_config);
                    self.depth_stencil.texture =
                        self.device.create_texture(&self.depth_stencil.descriptor);
                }
                _ => {}
            },

            _ => {}
        }
    }

    fn render(&mut self) {
        // let rotation = &Vec3::new(0.0, self.frame as f32 / 100.0, 0.0);
        let rotation = &Vec3::default();
        let object_matrix = transform_matrix_euler(
            &Vec3::new(0.0, 0.0, 5.0),
            &Vec3::new(0.1, 0.1, 0.1),
            rotation,
        );
        log::info!("Rotation = {rotation:?}");
        let camera_matrix = look_at_matrix(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        let screen_matrix = perspercive_projection(
            std::f32::consts::FRAC_PI_3,
            self.surface_config.width as f32 / self.surface_config.height as f32,
            0.1,
            10000.0,
        );

        let frame = self.surface.get_current_texture().unwrap_or_else(|_| {
            self.surface.configure(&self.device, &self.surface_config);
            self.surface
                .get_current_texture()
                .expect("Failed to get the next surface")
        });
        // .expect("Failed to get surface texture");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_view = self
            .depth_stencil
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            self.staging_belt
                .write_buffer(
                    &mut encoder,
                    &self.pipeline.buffers[0],
                    0,
                    BufferSize::new(size_of::<TransformationMatrices>() as u64).unwrap(),
                    &self.device,
                )
                .copy_from_slice(bytes_of(&TransformationMatrices {
                    object: object_matrix,
                    camera: camera_matrix,
                    screen: screen_matrix,
                }));
            self.staging_belt.finish();

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline.pipeline);
            for (index, g) in self.pipeline.bind_groups.iter().enumerate() {
                render_pass.set_bind_group(index as u32, g, &[]);
            }
            render_pass.set_vertex_buffer(0, self.v_buffer.slice(..));
            render_pass.set_index_buffer(self.i_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..self.ind_len, 0, 0..1);
        }

        let buffer = encoder.finish();

        self.queue.submit(Some(buffer));
        frame.present();
        self.staging_belt.recall();

        self.frame += 1;
    }
}

async fn req_adapter<'a>(
    instance: wgpu::Instance,
    options: &wgpu::RequestAdapterOptions<'a>,
) -> Option<wgpu::Adapter> {
    instance.request_adapter(options).await
}

async fn req_device<'a>(
    adapter: &wgpu::Adapter,
    descriptor: &wgpu::DeviceDescriptor<'a>,
) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    adapter.request_device(descriptor, None).await
}
