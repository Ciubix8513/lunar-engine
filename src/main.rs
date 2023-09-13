use event_loop::State;
use winit::event_loop::EventLoop;

mod event_loop;

fn main() {
    let event_loop = EventLoop::new();
    let mut state = State::new(&event_loop);
    event_loop.run(move |event, _, control_flow| state.app_loop(&event, control_flow));
}