use event_loop::State;
use winit::event_loop::EventLoop;

mod event_loop;
mod grimoire;
mod helpers;

fn main() {
    env_logger::Builder::new()
        .filter_module("wgpu", log::LevelFilter::Info)
        .filter_module("renderer", log::LevelFilter::Info)
        .init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut state = State::new(&event_loop);
    event_loop
        .run(move |event, target| {
            target.set_control_flow(winit::event_loop::ControlFlow::Poll);
            state.app_loop(&event, target);
        })
        .expect("Failed to run event loop");
}
