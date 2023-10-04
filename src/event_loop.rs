#![allow(dead_code)]
use std::mem::size_of;

use bytemuck::{bytes_of, Pod, Zeroable};
use math::{
    complex_shit::{look_at_matrix, perspercive_projection, transform_matrix_euler},
    mat4x4::Mat4x4,
    vec3::Vec3,
};
use wgpu::{util::StagingBelt, BufferSize, Features};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    buffers: Box<[wgpu::Buffer]>,
}

pub struct State {
    closed: bool,
    window: Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    pipeline: Pipeline,
    v_buffer: wgpu::Buffer,
    staging_belt: wgpu::util::StagingBelt,
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct TransformationMatrices {
    object: Mat4x4,
    camera: Mat4x4,
    screen: Mat4x4,
}

impl State {
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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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

        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<TransformationMatrices>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 12,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, //Some(wgpu::Face::Back),
                unclipped_depth: true,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
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

        let data: [[f32; 3]; 3] = [[-0.5, -0.5, 0.0], [0.0, 0.5, 0.0], [0.5, -0.5, 0.0]];

        let v_buffer = wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        Self {
            closed: false,
            window,
            device,
            queue,
            surface,
            surface_config,
            pipeline: Pipeline {
                pipeline,
                bind_group,
                buffers: Box::new([uniform]),
            },
            v_buffer,
            staging_belt: StagingBelt::new(1024),
        }
    }

    pub fn app_loop(&mut self, event: &Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::RedrawEventsCleared => self.window.request_redraw(),
            Event::RedrawRequested(_) => {
                if !self.closed {
                    self.render()
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
                    self.surface.configure(&self.device, &self.surface_config);
                }
                _ => {}
            },

            _ => {}
        }
    }

    fn render(&mut self) {
        let object_matrix = transform_matrix_euler(
            &Vec3::new(0.0, 1.0, 3.0),
            &Vec3::new(1.0, 1.0, 1.0),
            &Vec3::default(),
        );
        let camera_matrix = look_at_matrix(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        let screen_matrix = perspercive_projection(
            std::f32::consts::FRAC_PI_3,
            self.surface_config.width as f32 / self.surface_config.height as f32,
            0.1,
            100.0,
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
                    object: (object_matrix * camera_matrix * screen_matrix).transpose(),
                    // object: object_matrix.transpose(),
                    camera: object_matrix.transpose(),
                    screen: screen_matrix.transpose(),
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
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline.pipeline);
            render_pass.set_bind_group(0, &self.pipeline.bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.v_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }

        let buffer = encoder.finish();

        self.queue.submit(Some(buffer));
        frame.present();
        self.staging_belt.recall();
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
