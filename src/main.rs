use event_loop::State;
use winit::event_loop::EventLoop;

mod event_loop;

fn main() {
    env_logger::Builder::new()
        .filter_module("wgpu", log::LevelFilter::Info)
        .filter_module("renderer", log::LevelFilter::Info)
        .init();
    let event_loop = EventLoop::new();
    let mut state = State::new(&event_loop);
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        state.app_loop(&event, control_flow);
    });
}
