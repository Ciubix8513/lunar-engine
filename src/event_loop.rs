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
            .expect("Did not have last format");
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        State {
            window,
            device,
            queue,
            surface,
            surface_config,
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
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
