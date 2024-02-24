use log::info;
use lunar_lib::StateItems;

// mod event_loop;

fn init(items: &StateItems) {
    info!("Initialized!")
}
fn run(items: &StateItems) {
    // info!("Frame!");
}

fn main() {
    let state = lunar_lib::State::new();
    state.run(init, run);
}
