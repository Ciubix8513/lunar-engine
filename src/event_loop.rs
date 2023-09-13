use wgpu::{include_wgsl, util::DeviceExt, ColorWrites, VertexAttribute, VertexBufferLayout};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct State {
    window: Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    v_buffer: wgpu::Buffer,
}

impl State {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let window = winit::window::Window::new(event_loop).expect("Failed to create new window");
        env_logger::init();

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
        .unwrap();

        let (device, queue): (wgpu::Device, wgpu::Queue) = futures::executor::block_on(req_device(
            &adapter,
            &wgpu::DeviceDescriptor {
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
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let vert_shader = device.create_shader_module(include_wgsl!("./shaders/vertex.wgsl"));
        let frag_shader = device.create_shader_module(include_wgsl!("./shaders/color.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: 12,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
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
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let data: [[f32; 3]; 3] = [[-0.5, -0.5, 0.0], [0.0, 0.5, 0.0], [0.5, -0.5, 0.0]];

        let v_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            window,
            device,
            queue,
            surface,
            surface_config,
            pipeline,
            v_buffer,
        }
    }

    pub fn app_loop(&mut self, event: &Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent { window_id, event } => match event {
                winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::Resized(size) => {
                    self.surface_config.width = size.width;
                    self.surface_config.height = size.height;

                    self.surface.configure(&self.device, &self.surface_config);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let frame = self
                    .surface
                    .get_current_texture()
                    .expect("Failed to get surface texture");
                let frame_view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
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

                    render_pass.set_pipeline(&self.pipeline);

                    render_pass.set_vertex_buffer(0, self.v_buffer.slice(..));
                    render_pass.draw(0..3, 0..1);
                }

                let buffer = encoder.finish();

                self.queue.submit(Some(buffer));
                frame.present();
            }
            _ => {}
        }
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
