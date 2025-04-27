//!
//! # Lunar engine
//! A small silly engine for fun :3
//!
//!
//! # Project setup
//! Setting up a project is really simple. The application is split into 3 states:
//! 1. Initialization
//! 2. Main loop
//! 3. Disposal
//!
//! First define the state of the app
//!
//! ```
//! struct MyState;
//! ```
//! The state can contain any data that needs to be persistent between frames, for example an
//! `AssetStore` or `World`
//!
//! Define the application functions, all of them have identical signature:
//! ```
//! # struct MyState;
//! fn initialize(state: &mut MyState) {}
//! fn run(state: &mut MyState) {}
//! fn close(state: &mut MyState) {}
//! ```
//! Then create an instance of that state and start the loop of the program
//! ```no_run
//! # #[derive(Default)]
//! # struct MyState;
//! # fn initialize(state: &mut MyState) {}
//! # fn run(state: &mut MyState) {}
//! # fn close(state: &mut MyState) {}
//! fn main() {
//!     let state = lunar_engine::State::<MyState>::default();
//!     state.run(initialize, run, close);
//! }
//! ```
//!
#![deny(missing_docs)]
#![allow(
    clippy::needless_doctest_main,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_panics_doc,
    clippy::inline_always
)]
use std::{
    cell::OnceCell,
    sync::{OnceLock, RwLock},
};

use chrono::DateTime;
use input::INPUT;
#[allow(clippy::wildcard_imports)]
use internal::*;
use wgpu::SurfaceConfiguration;
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event};

#[cfg(target_arch = "wasm32")]
use crate::wrappers::WgpuWrapper;

pub mod asset_managment;
pub mod assets;
pub mod components;
pub mod ecs;
mod grimoire;
mod helpers;
pub mod import;
pub mod input;
pub mod internal;
mod logging;
pub mod math;
pub mod rendering;
///Various structures
pub mod structures;
#[cfg(test)]
mod test_utils;
mod windowing;
#[cfg(target_arch = "wasm32")]
mod wrappers;

///UUID of an asset or an entity
pub type UUID = u128;

//TODO find a better way than just staticing it
static WINDOW: OnceLock<winit::window::Window> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
static SURFACE: OnceLock<RwLock<wrappers::WgpuWrapper<wgpu::Surface>>> = OnceLock::new();
#[cfg(target_arch = "wasm32")]
static DEPTH: OnceLock<RwLock<wrappers::WgpuWrapper<wgpu::Texture>>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
static SURFACE: OnceLock<RwLock<wgpu::Surface>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static DEPTH: OnceLock<RwLock<wgpu::Texture>> = OnceLock::new();

static QUIT: OnceLock<bool> = OnceLock::new();
static DELTA_TIME: RwLock<f32> = RwLock::new(0.01);
static VSYNC_CHANGE: RwLock<Option<Vsync>> = RwLock::new(None);

///Exits the application and closes the window
pub fn quit() {
    QUIT.set(true).unwrap();
}

///Returns time between frames in seconds
pub fn delta_time() -> f32 {
    *DELTA_TIME.read().unwrap()
}

///Vsync state
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Vsync {
    ///vsync enabled
    Vsync,
    ///vsync disabled
    NoVsync,
}

///Sets the vsync of the game
pub fn set_vsync(vsync: Vsync) {
    *VSYNC_CHANGE.write().unwrap() = Some(vsync);
}

///Contains main state of the app
#[allow(clippy::type_complexity)]
pub struct State<T> {
    first_resume: bool,
    surface_config: OnceCell<SurfaceConfiguration>,
    vsync: Vsync,
    contents: T,
    closed: bool,
    frame_start: Option<DateTime<chrono::Local>>,
    init: Option<Box<dyn FnOnce(&mut T)>>,
    run: Option<Box<dyn Fn(&mut T)>>,
    end: Option<Box<dyn FnOnce(&mut T)>>,
}

impl<T: Default> Default for State<T> {
    fn default() -> Self {
        Self {
            vsync: Vsync::NoVsync,
            first_resume: false,
            surface_config: OnceCell::default(),
            contents: Default::default(),
            closed: Default::default(),
            frame_start: None,
            init: None,
            run: None,
            end: None,
        }
    }
}

impl<T: 'static> State<T> {
    ///Creates a new state with the given custom state
    pub fn new(contents: T) -> Self {
        Self {
            vsync: Vsync::NoVsync,
            first_resume: false,
            surface_config: OnceCell::new(),
            contents,
            closed: false,
            frame_start: None,
            init: None,
            run: None,
            end: None,
        }
    }

    /// Starts the application with the 3 provided functions:
    /// 1. Initialization function for setting up assets, scene(s), etc.
    /// 2. Game loop
    /// 3. Disposal function
    //TODO Potentially ask for a window
    #[allow(clippy::missing_panics_doc)]
    pub fn run<F, F1, F2>(mut self, init: F, run: F1, end: F2)
    where
        F: FnOnce(&mut T) + 'static,
        F1: Fn(&mut T) + Copy + 'static,
        F2: FnOnce(&mut T) + Copy + 'static,
    {
        self.init = Some(Box::new(init));
        self.run = Some(Box::new(run));
        self.end = Some(Box::new(end));

        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(|e| {
                log::error!("{e}");
            }));
        }

        //Initialize logging first

        if logging::initialize_logging().is_err() {
            log::warn!("Logger already initialized");
        }

        let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
        log::debug!("Created event loop");

        //Start tracy
        //
        //I don't trust it so i'm only gonna start it if the tracy feature is on
        #[cfg(feature = "tracy")]
        tracy_client::Client::start();

        #[cfg(not(target_arch = "wasm32"))]
        {
            event_loop
                .run_app(&mut self)
                .expect("Failed to start event loop");
        }
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(self);
        }
    }
}
impl<T> State<T> {
    fn configure_surface(&self) {
        let device = DEVICE.get().unwrap();

        SURFACE
            .get()
            .unwrap()
            .write()
            .unwrap()
            .configure(device, self.surface_config.get().unwrap());
    }

    fn initialize(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        #[cfg(not(target_arch = "wasm32"))]
        let attributes = winit::window::Window::default_attributes();
        let window;
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            let mut attributes = winit::window::Window::default_attributes();

            //Acquire a canvas as a base for the window
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .expect("Failed to find canvas with id \"canvas\"")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            let width = canvas.width();
            let height = canvas.height();

            log::info!("Canvas size = {width} x {height}");

            log::debug!("Found canvas");
            attributes = attributes.with_canvas(Some(canvas));

            window = event_loop
                .create_window(attributes)
                .expect("Failed to create the window");
            //Resize window to the canvas size
            //TODO Find a better solution to this hack
            _ = window.request_inner_size(PhysicalSize::new(width, height));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            window = event_loop
                .create_window(attributes)
                .expect("Failed to create the window");
        }

        log::debug!("Created window");

        WINDOW.set(window).unwrap();
        let window = WINDOW.get().unwrap();

        let (surface, config, depth_stencil) = windowing::initialize_gpu(window);

        log::debug!("Inititalized GPU");

        self.surface_config.set(config).unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        {
            SURFACE.set(RwLock::new(surface)).unwrap();
            DEPTH.set(RwLock::new(depth_stencil)).unwrap();
        }
        #[cfg(target_arch = "wasm32")]
        {
            SURFACE.set(RwLock::new(WgpuWrapper::new(surface))).unwrap();
            DEPTH
                .set(RwLock::new(WgpuWrapper::new(depth_stencil)))
                .unwrap();
        }

        self.init.take().unwrap()(&mut self.contents);

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        RESOLUTION.write().unwrap().width = size.width;
        RESOLUTION.write().unwrap().height = size.height;
        self.surface_config.get_mut().unwrap().width = size.width;
        self.surface_config.get_mut().unwrap().height = size.height;
        let desc = windowing::get_depth_descriptor(size.width, size.height);

        self.configure_surface();

        let device = DEVICE.get().unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            **DEPTH.get().unwrap().write().unwrap() = device.create_texture(&desc);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            *DEPTH.get().unwrap().write().unwrap() = device.create_texture(&desc);
        }
    }

    fn redraw(&mut self) {
        #[cfg(feature = "tracy")]
        let _span = tracy_client::span!("Redraw call");

        //Frame time includes the wait between frames
        if let Some(start) = self.frame_start {
            let finish = chrono::Local::now();

            let delta = (finish - start).abs().num_microseconds().unwrap() as f32 / 1_000_000.0;

            *DELTA_TIME.write().unwrap() = delta;
        }

        //Check if vsync changed
        if VSYNC_CHANGE.read().unwrap().is_some() {
            let mut v = VSYNC_CHANGE.write().unwrap();
            if self.vsync != v.unwrap() {
                let conf = self.surface_config.get_mut().unwrap();
                self.vsync = v.unwrap();
                conf.present_mode = match v.unwrap() {
                    Vsync::Vsync => wgpu::PresentMode::AutoVsync,
                    Vsync::NoVsync => wgpu::PresentMode::AutoNoVsync,
                };
                self.configure_surface();
                log::info!("Changing vsync mode");
            }

            *v = None;
        }

        self.frame_start = Some(chrono::Local::now());

        input::process_cursor();

        if self.closed {
            //This should be fine but needs further testing
            self.end.take().unwrap()(&mut self.contents);

            return;
        }
        self.run.as_ref().unwrap()(&mut self.contents);
        input::update();

        WINDOW.get().unwrap().request_redraw();
    }
}

impl<T> ApplicationHandler for State<T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.first_resume {
            return;
        }
        self.initialize(event_loop);
    }

    fn device_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        #[allow(clippy::single_match)]
        match event {
            event::DeviceEvent::MouseMotion { delta } => {
                let d = math::Vec2::new(delta.0 as f32, delta.1 as f32);

                let i = INPUT.get().unwrap();
                *i.raw_curosor_delta.write().unwrap() = d;
                *i.delta_changed.write().unwrap() = true;
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: event::WindowEvent,
    ) {
        match event {
            event::WindowEvent::Resized(size) => self.resize(size),
            event::WindowEvent::CloseRequested => {
                event_loop.exit();
                self.closed = true;
            }
            event::WindowEvent::RedrawRequested => {
                if QUIT.get().is_some() {
                    event_loop.exit();
                    self.closed = true;
                }

                self.redraw();
            }
            event::WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let state = match event.state {
                    event::ElementState::Pressed => input::KeyState::Down,
                    event::ElementState::Released => input::KeyState::Up,
                };
                let keycode = if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    Some(code)
                } else {
                    None
                };
                if keycode.is_none() {
                    return;
                }
                input::set_key(keycode.unwrap(), state);
            }
            event::WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match state {
                event::ElementState::Pressed => {
                    input::set_mouse_button(button, input::KeyState::Down);
                }
                event::ElementState::Released => {
                    input::set_mouse_button(button, input::KeyState::Up);
                }
            },

            event::WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                input::set_cursor_position(math::Vec2 {
                    x: position.x as f32,
                    y: position.y as f32,
                });
            }
            _ => {}
        }
    }
}
