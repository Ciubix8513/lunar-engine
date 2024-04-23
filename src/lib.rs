use std::{
    cell::OnceCell,
    sync::{OnceLock, RwLock},
};

use wgpu::SurfaceConfiguration;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{self, Event},
    event_loop::EventLoopWindowTarget,
    window::CursorGrabMode,
};
use wrappers::WgpuWrapper;

pub mod asset_managment;
pub mod assets;
pub mod components;
pub mod ecs;
mod grimoire;
mod helpers;
pub mod import;
pub mod input;
pub mod math;
pub mod structures;
pub mod system;
#[cfg(test)]
mod test_utils;
pub mod windowing;
mod wrappers;
#[cfg(target_arch = "wasm32")]
pub static DEVICE: OnceLock<wrappers::WgpuWrapper<wgpu::Device>> = OnceLock::new();
#[cfg(target_arch = "wasm32")]
pub static QUEUE: OnceLock<wrappers::WgpuWrapper<wgpu::Queue>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
pub static QUEUE: OnceLock<wgpu::Queue> = OnceLock::new();
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
pub static STAGING_BELT: OnceLock<RwLock<wrappers::WgpuWrapper<wgpu::util::StagingBelt>>> =
    OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
pub static STAGING_BELT: OnceLock<RwLock<wgpu::util::StagingBelt>> = OnceLock::new();
pub static RESOLUTION: RwLock<PhysicalSize<u32>> = RwLock::new(PhysicalSize {
    width: 0,
    height: 0,
});
//TODO find a better way than just staticing it

#[cfg(target_arch = "wasm32")]
static SURFACE: OnceLock<RwLock<wrappers::WgpuWrapper<wgpu::Surface>>> = OnceLock::new();
#[cfg(target_arch = "wasm32")]
static DEPTH: OnceLock<RwLock<wrappers::WgpuWrapper<wgpu::Texture>>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
static SURFACE: OnceLock<RwLock<wgpu::Surface>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static DEPTH: OnceLock<RwLock<wgpu::Texture>> = OnceLock::new();

static QUIT: OnceLock<bool> = OnceLock::new();
static DELTA_TIME: RwLock<f32> = RwLock::new(0.0);

///Defines behaviour of the cursor inside the window
pub enum CursorState {
    //Cursor is locked to the window
    Locked,
    //Cursor is free
    Free,
}

struct CursorStateInternal {
    grab_mode: CursorState,
    lock_failed: bool,
    visible: bool,
    modified: bool,
}
static CURSOR_STATE: RwLock<CursorStateInternal> = RwLock::new(CursorStateInternal {
    grab_mode: CursorState::Free,
    lock_failed: false,
    visible: true,
    modified: false,
});

//Exits the application and closes the window
pub fn quit() {
    QUIT.set(true).unwrap();
}

///Returns time between frames in seconds
pub fn delta_time() -> f32 {
    *DELTA_TIME.read().unwrap()
}

///Sets the cursor grab mode
pub fn set_cursor_grab_mode(mode: CursorState) {
    let mut state = CURSOR_STATE.write().unwrap();
    state.grab_mode = mode;
    state.modified = true;
}

///Sets the cursor grab mode
pub fn set_cursor_visible(mode: bool) {
    let mut state = CURSOR_STATE.write().unwrap();
    state.visible = mode;
    state.modified = true;
}

pub struct State<T> {
    window: OnceCell<winit::window::Window>,
    surface_config: OnceCell<SurfaceConfiguration>,
    contents: T,
    closed: bool, //Various state related stuff
}

impl<T: Default> Default for State<T> {
    fn default() -> Self {
        Self {
            window: Default::default(),
            surface_config: Default::default(),
            contents: Default::default(),
            closed: Default::default(),
        }
    }
}

impl<T> State<T> {
    fn reset_cursor(&self) {
        let window = self.window.get().unwrap();

        let pos = window.inner_size();
        if let Err(e) = window.set_cursor_position(PhysicalPosition {
            x: pos.width / 2,
            y: pos.height / 2,
        }) {
            log::error!("Failed to move cursor {e}");
        }
    }

    fn process_cursor(&self) {
        let mut state = CURSOR_STATE.write().unwrap();

        if matches!(state.grab_mode, CursorState::Locked) && state.lock_failed {
            self.reset_cursor();
        }

        if !state.modified {
            return;
        }
        state.modified = false;
        let window = self.window.get().unwrap();

        self.window.get().unwrap().set_cursor_visible(state.visible);

        if let Err(e) = window.set_cursor_grab(match state.grab_mode {
            CursorState::Locked => CursorGrabMode::Locked,
            CursorState::Free => CursorGrabMode::None,
        }) {
            match e {
                winit::error::ExternalError::NotSupported(_) => {
                    //Once a lock has failed, it can never unfail, so no need to reset this
                    //afterwards :3
                    //
                    //This can only unfail if the user changes platform, buuuut, i literally don't
                    //think there's a way that could happen
                    state.lock_failed = true;
                    log::warn!("Failed to lock cursor, doing manually");
                    if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                        log::error!("Cursor is fucked :3 {e}");
                    }
                    self.reset_cursor();
                }

                winit::error::ExternalError::Ignored => {
                    log::warn!("Cursor state change ignored")
                }
                winit::error::ExternalError::Os(e) => log::error!("Cursor state change error: {e}"),
            }
        }
    }

    pub fn new(contents: T) -> Self {
        Self {
            window: OnceCell::new(),
            surface_config: OnceCell::new(),
            contents,
            closed: false,
        }
    }

    //TODO Potentially ask for a window
    pub fn run<F, F1, F2>(mut self, init: F, run: F1, end: F2)
    where
        F: FnOnce(&mut T),
        F1: Fn(&mut T) + Copy,
        F2: FnOnce(&mut T) + Copy,
    {
        let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
        let mut builder = winit::window::WindowBuilder::new();
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowBuilderExtWebSys;

            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .expect("Failed to find canvas with id \"canvas\"")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            builder = builder.with_canvas(Some(canvas));
        }

        let window = builder
            .build(&event_loop)
            .expect("Failed to create the window");
        self.window.set(window).unwrap();
        windowing::initialize_logging();
        let (surface, config, depth_stencil) =
            windowing::initialize_gpu(&self.window.get().unwrap());
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
        init(&mut self.contents);

        event_loop
            .run(move |e, w| {
                w.set_control_flow(winit::event_loop::ControlFlow::Poll);
                self.event_handler(e, w, run, end);
            })
            .expect("Failed to start event loop");
    }

    fn event_handler<T1, F, F1>(
        &mut self,
        event: Event<T1>,
        window: &EventLoopWindowTarget<T1>,
        run_func: F,
        end: F1,
    ) where
        F: Fn(&mut T),
        F1: FnOnce(&mut T),
    {
        match event {
            Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                event::WindowEvent::Resized(size) => {
                    RESOLUTION.write().unwrap().width = size.width;
                    RESOLUTION.write().unwrap().height = size.height;
                    self.surface_config.get_mut().unwrap().width = size.width;
                    self.surface_config.get_mut().unwrap().height = size.height;
                    let device = DEVICE.get().unwrap();

                    SURFACE
                        .get()
                        .unwrap()
                        .write()
                        .unwrap()
                        .configure(device, self.surface_config.get().unwrap());
                    let desc = windowing::get_depth_descriptor(size.width, size.height);

                    #[cfg(target_arch = "wasm32")]
                    {
                        **DEPTH.get().unwrap().write().unwrap() = device.create_texture(&desc);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        *DEPTH.get().unwrap().write().unwrap() = device.create_texture(&desc);
                    }

                    // let bpr = helpers::calculate_bpr(size.width, *FORMAT.get().unwrap());
                    // self.screenshot_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    //     label: Some("Screenshot buffer"),
                    //     size: bpr * size.height as u64,
                    //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    //     mapped_at_creation: false,
                    // });
                }
                event::WindowEvent::CloseRequested => {
                    window.exit();
                    self.closed = true;
                }
                event::WindowEvent::RedrawRequested => {
                    self.process_cursor();

                    let start = chrono::Local::now();

                    if QUIT.get().is_some() {
                        window.exit();
                        self.closed = true;
                    }
                    if self.closed {
                        //This should be fine but needs further testing
                        end(&mut self.contents);

                        return;
                    }
                    run_func(&mut self.contents);
                    self.window.get().unwrap().request_redraw();
                    input::update();
                    let finish = chrono::Local::now();

                    let delta =
                        (finish - start).abs().num_microseconds().unwrap() as f32 / 1000000.0;

                    *DELTA_TIME.write().unwrap() = delta;
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
                    let keycode =
                        if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
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
                        input::set_mouse_button(button, input::KeyState::Down)
                    }
                    event::ElementState::Released => {
                        input::set_mouse_button(button, input::KeyState::Up)
                    }
                },

                event::WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                } => {
                    input::set_cursor_position(math::vec2::Vec2 {
                        x: position.x as f32,
                        y: position.y as f32,
                    });
                }
                _ => {}
            },
            _ => {}
        }
    }
}
