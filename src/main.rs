use log::info;

#[derive(Default)]
struct State {
    frame: u32,
}
// mod event_loop;

fn init(state: &mut State) {
    state.frame = 0;
    info!("Initialized!")
}
fn run(state: &mut State) {
    state.frame += 1;
}

fn close(state: &mut State) {
    info!("Ran for {} frames", state.frame);
}

fn main() {
    let state = lunar_lib::State::<State>::default();
    state.run(init, run, close);
}
