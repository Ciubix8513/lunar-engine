#![allow(
    dead_code,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
    clippy::single_match
)]
use bytemuck::bytes_of;
use renderer_lib::math::{
    mat4x4::{self, Mat4x4},
    vec3::Vec3,
};
use std::{num::NonZeroU64, path::Path, thread};
use wgpu::{
    util::{DeviceExt, StagingBelt},
    Extent3d,
};
use winit::{
    event::{ElementState, Event},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::PhysicalKey,
};

use crate::{
    abstractions::{
        self,
        material::{texture_unlit::TextureUnlit, Material},
        model::Model,
        DEVICE, FORMAT, QUEUE,
    },
    grimoire, helpers,
};

pub struct Camera {
    pub position: Vec3,
    pub rotation: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub screen_aspect: f32,
    pub uniform: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
impl Camera {
    pub fn matrix(&self) -> mat4x4::Mat4x4 {
        //Todo actual camera stuff
        let camera_matrix = Mat4x4::look_at_matrix(
            self.position,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        let projection_matrix =
            Mat4x4::perspercive_projection(self.fov, self.screen_aspect, self.near, self.far);
        camera_matrix * projection_matrix
    }
}
struct DepthStencil<'a> {
    texture: wgpu::Texture,
    descriptor: wgpu::TextureDescriptor<'a>,
}
//Datastructure to keep track of all the active, supported feautres
#[derive(Default)]
pub struct Features {
    pub screenshot: bool,
}

pub struct State<'stencil> {
    features: Features,
    closed: bool,
    window: winit::window::Window,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    models: Vec<abstractions::model::Model>,
    staging_belt: wgpu::util::StagingBelt,
    depth_stencil: DepthStencil<'stencil>,
    screenshot_buffer: wgpu::Buffer,
    frame: u64,
    screenshot: bool,
    camera: Camera,
    materials: Vec<Box<dyn Material>>,
}

impl State<'_> {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut features = Features::default();
        let window = winit::window::Window::new(event_loop).expect("Failed to create new window");

        let size = window.inner_size();
        let instance = wgpu::Instance::default();

        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Failed to createate surface")
        };

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
                features: wgpu::Features::DEPTH_CLIP_CONTROL,
                ..Default::default()
            },
        ))
        .expect("Failed to create a device and a queue");

        DEVICE.set(device).unwrap();
        QUEUE.set(queue).unwrap();

        let device = DEVICE.get().unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .last()
            .copied()
            .expect("Did not have last format");
        FORMAT.set(format).unwrap();
        assert!(capabilities.usages & wgpu::TextureUsages::RENDER_ATTACHMENT == wgpu::TextureUsages::RENDER_ATTACHMENT, "Rendering not supported... What shitty ancient piece of shit are you fucking using wtf?");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: if capabilities.usages & wgpu::TextureUsages::COPY_SRC
                == wgpu::TextureUsages::COPY_SRC
            {
                features.screenshot = true;
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
            } else {
                log::warn!("Screenshot feature not supported!");
                wgpu::TextureUsages::RENDER_ATTACHMENT
            },
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            view_formats: vec![format],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(device, &surface_config);

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

        let bpr = helpers::calculate_bpr(size.width, format);
        let screenshot_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screenshot buffer"),
            size: bpr * u64::from(size.height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let camera_matrix = Mat4x4::look_at_matrix(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        let projection_matrix = Mat4x4::perspercive_projection(
            std::f32::consts::FRAC_PI_3,
            surface_config.width as f32 / surface_config.height as f32,
            0.1,
            10000.0,
        );
        let camera_mat = camera_matrix * projection_matrix;

        let camera_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytes_of(&camera_mat),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&grimoire::CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind gropu"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_uniform,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let camera = Camera {
            position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: Vec3::default(),
            fov: std::f32::consts::FRAC_PI_3,
            screen_aspect: surface_config.width as f32 / surface_config.height as f32,
            near: 0.1,
            far: 10000.0,
            uniform: camera_uniform,
            bind_group: camera_bind_group,
        };

        let texture_data =
            renderer_lib::import::bmp::parse(include_bytes!("../assets/blahaj1.bmp")).unwrap();
        let blahaj_material = TextureUnlit::new(&texture_data);

        let mut model =
            renderer_lib::import::obj::parse(include_str!("../assets/blahaj.obj")).unwrap();
        let mut blahaj = Model::new(model.remove(0));
        blahaj.transform.position.z = -3.5;

        Self {
            features,
            closed: false,
            window,
            surface,
            surface_config,
            staging_belt: StagingBelt::new(1024),
            depth_stencil: DepthStencil {
                texture: depth_stencil,
                descriptor,
            },
            frame: 0,
            screenshot_buffer,
            screenshot: false,
            models: vec![blahaj],
            camera,
            materials: vec![Box::new(blahaj_material)],
        }
    }

    pub fn app_loop(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    self.surface_config.width = size.width;
                    self.surface_config.height = size.height;

                    self.depth_stencil.descriptor.size.width = size.width;
                    self.depth_stencil.descriptor.size.height = size.height;

                    self.camera.screen_aspect = size.width as f32 / size.height as f32;

                    let device = DEVICE.get().unwrap();

                    self.surface.configure(device, &self.surface_config);
                    self.depth_stencil.texture =
                        device.create_texture(&self.depth_stencil.descriptor);
                    let bpr =
                        helpers::calculate_bpr(size.width, *abstractions::FORMAT.get().unwrap());

                    self.screenshot_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Screenshot buffer"),
                        size: bpr * size.height as u64,
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                        mapped_at_creation: false,
                    });
                }
                winit::event::WindowEvent::CloseRequested => {
                    target.exit();
                    self.closed = true;
                }
                winit::event::WindowEvent::RedrawRequested => {
                    if self.closed {
                        return;
                    }
                    log::debug!("Frame start");
                    self.render();
                    log::debug!("Frame end");
                    self.window.request_redraw();
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id: _,
                    event,
                    is_synthetic: _,
                } if event.state == ElementState::Pressed => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            winit::keyboard::KeyCode::KeyP => {
                                if self.features.screenshot {
                                    self.screenshot = true;
                                    log::info!("Taking a screenshot");
                                }
                            }
                            winit::keyboard::KeyCode::KeyS => {
                                let mut new_blahaj =
                                    Model::new(self.models.get(0).unwrap().mesh.clone());
                                new_blahaj.transform.position = Vec3::random(-10.0, 10.0);
                                new_blahaj.transform.scale = Vec3::random(0.01, 2.0);
                                self.models.push(new_blahaj);
                            }
                            _ => {}
                        }
                    }
                }
                // winit::event::WindowEvent::CursorMoved {
                //     device_id,
                //     position,
                // } => todo!(),
                // winit::event::WindowEvent::CursorEntered { device_id } => todo!(),
                // winit::event::WindowEvent::CursorLeft { device_id } => todo!(),
                // winit::event::WindowEvent::MouseWheel {
                //     device_id,
                //     delta,
                //     phase,
                // } => todo!(),
                // winit::event::WindowEvent::MouseInput {
                //     device_id,
                //     state,
                //     button,
                // } => todo!(),
                _ => {}
            },
            _ => {}
        }
    }

    fn render(&mut self) {
        let device = DEVICE.get().unwrap();
        let queue = QUEUE.get().unwrap();

        let frame = self.surface.get_current_texture().unwrap_or_else(|_| {
            self.surface.configure(device, &self.surface_config);
            self.surface
                .get_current_texture()
                .expect("Failed to get the next surface")
        });
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.surface_config.format),
            ..Default::default()
        });
        let depth_view = self
            .depth_stencil
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        //Render pass
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        //update models
        for m in &self.models {
            m.update_uniforms(&mut self.staging_belt, &mut encoder);
        }
        self.staging_belt
            .write_buffer(
                &mut encoder,
                &self.camera.uniform,
                0,
                NonZeroU64::new(std::mem::size_of::<mat4x4::Mat4x4>() as u64).unwrap(),
                device,
            )
            .copy_from_slice(bytes_of(&self.camera.matrix()));
        self.staging_belt.finish();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,

            occlusion_query_set: None,
        });

        //Set the material
        self.materials.get(0).unwrap().render(&mut render_pass);

        render_pass.set_bind_group(
            grimoire::CAMERA_BIND_GROUP_INDEX,
            &self.camera.bind_group,
            &[],
        );

        for m in &self.models {
            m.render(&mut render_pass);
        }
        drop(render_pass);

        if self.screenshot && self.features.screenshot {
            let image_size = frame.texture.size();
            let bpr = helpers::calculate_bpr(image_size.width, frame.texture.format()) as u32;

            encoder.copy_texture_to_buffer(
                frame.texture.as_image_copy(),
                wgpu::ImageCopyBufferBase {
                    buffer: &self.screenshot_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(bpr), //(image_size.width * 4 * 4),
                        rows_per_image: Some(image_size.height), //(image_size.height),
                    },
                },
                frame.texture.size(),
            );

            queue.submit(Some(encoder.finish()));
            self.staging_belt.recall();

            let slice = self.screenshot_buffer.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::Maintain::Wait);
            let buffer = slice
                .get_mapped_range()
                .iter()
                .copied()
                .collect::<Vec<u8>>();
            self.screenshot_buffer.unmap();

            let p = Path::new(grimoire::SCREENSHOT_DIRECTORY);
            if !p.exists() {
                if let Err(e) = std::fs::create_dir(p) {
                    log::error!("Failed to create screenshots directory {e}");
                }
            }
            let filename = format!(
                "{}/screenshot_{}.png",
                grimoire::SCREENSHOT_DIRECTORY,
                chrono::Local::now().format(grimoire::FILE_TIME_FORMAT)
            );
            log::info!("Screenshot filename = {filename}");

            thread::spawn(move || {
                let image = renderer_lib::helpers::arr_to_image(
                    &buffer,
                    bpr / 4,
                    image_size.width,
                    image_size.height,
                    image::ImageOutputFormat::Png,
                )
                .unwrap();

                if let Err(e) = std::fs::write(filename, image) {
                    log::error!("Failed to write image {e}");
                }
            });
            self.screenshot = false;
        } else {
            queue.submit(Some(encoder.finish()));
            self.staging_belt.recall();
        }

        self.frame += 1;
        frame.present();
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
