use std::{
    cell::OnceCell,
    sync::{OnceLock, RwLock},
};

use wgpu::{SurfaceConfiguration, Texture, TextureDescriptor};
use winit::{dpi::PhysicalSize, event::Event, event_loop::EventLoopWindowTarget};

pub mod abstractions;
pub mod asset_managment;
pub mod assets;
pub mod components;
pub mod ecs;
pub mod grimoire;
pub mod helpers;
pub mod import;
pub mod math;
pub mod structrures;
pub mod system;
#[cfg(test)]
mod test_utils;
pub mod windowing;

pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
pub static QUEUE: OnceLock<wgpu::Queue> = OnceLock::new();
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();
pub static STAGING_BELT: OnceLock<RwLock<wgpu::util::StagingBelt>> = OnceLock::new();
pub static RESOLUTION: RwLock<PhysicalSize<u32>> = RwLock::new(PhysicalSize {
    width: 0,
    height: 0,
});

pub struct StateItems {}

pub struct State {
    window: OnceCell<winit::window::Window>,
    surface: OnceCell<wgpu::Surface>,
    surface_config: OnceCell<SurfaceConfiguration>,
    depth: OnceCell<Texture>,
    items: StateItems,
    closed: bool, //Various state related stuff
}

impl State {
    pub fn new() -> Self {
        Self {
            window: OnceCell::new(),
            surface_config: OnceCell::new(),
            surface: OnceCell::new(),
            depth: OnceCell::new(),
            items: StateItems {},
            closed: false,
        }
    }

    //TODO Potentially ask for a window
    pub fn run<F, F1>(mut self, init: F, run: F1)
    where
        F: FnOnce(&StateItems),
        F1: Fn(&StateItems) + Copy,
    {
        let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
        let window = winit::window::Window::new(&event_loop).expect("Failed to create the window");
        self.window.set(window).unwrap();
        windowing::initialize_logging();
        let (surface, config, depth_stencil) =
            windowing::initialize_gpu(&self.window.get().unwrap());
        self.surface.set(surface).unwrap();
        self.surface_config.set(config).unwrap();
        self.depth.set(depth_stencil).unwrap();
        init(&self.items);

        event_loop
            .run(move |e, w| {
                w.set_control_flow(winit::event_loop::ControlFlow::Poll);
                self.event_handler(e, w, run);
            })
            .expect("Failed to start event loop");
    }

    fn event_handler<T, F>(
        &mut self,
        event: Event<T>,
        window: &EventLoopWindowTarget<T>,
        run_func: F,
    ) where
        F: Fn(&StateItems),
    {
        match event {
            Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    self.surface_config.get_mut().unwrap().width = size.width;
                    self.surface_config.get_mut().unwrap().height = size.height;
                    let device = DEVICE.get().unwrap();

                    self.surface
                        .get_mut()
                        .unwrap()
                        .configure(device, self.surface_config.get().unwrap());
                    let desc = windowing::get_depth_descriptor(size.width, size.height);
                    *self.depth.get_mut().unwrap() = device.create_texture(&desc);

                        // let bpr = helpers::calculate_bpr(size.width, *FORMAT.get().unwrap());
                    // self.screenshot_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    //     label: Some("Screenshot buffer"),
                    //     size: bpr * size.height as u64,
                    //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    //     mapped_at_creation: false,
                    // });
                }
                winit::event::WindowEvent::CloseRequested => {
                    window.exit();
                    self.closed = true;
                }
                winit::event::WindowEvent::RedrawRequested => {
                    if self.closed {
                        return;
                    }
                    run_func(&self.items);
                    self.window.get().unwrap().request_redraw();
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id: _,
                    event,
                    is_synthetic: _,
                } if event.state == winit::event::ElementState::Pressed => {
                    if let winit::keyboard::PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            winit::keyboard::KeyCode::KeyP => {
                                // if self.features.screenshot {
                                //     self.screenshot = true;
                                //     log::info!("Taking a screenshot");
                                // }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}



//Rendering remains
// let frame = self.surface.get_current_texture().unwrap_or_else(|_| {
//     self.surface.configure(device, &self.surface_config);
//     self.surface
//         .get_current_texture()
//         .expect("Failed to get the next surface")
// });
// let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
//     format: Some(self.surface_config.format),
//     ..Default::default()
// });

// let depth_view = self
//     .depth_stencil
//     .texture
//     .create_view(&wgpu::TextureViewDescriptor::default());

// self.frame += 1;
// frame.present();
