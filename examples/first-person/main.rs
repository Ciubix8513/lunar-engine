use lunar_engine::{
    asset_managment::AssetStore,
    components::{camera::MainCamera, transform::Transform},
    ecs::{Component, EntityBuilder, World},
    input::{self, CursorLock, KeyState},
    rendering::{extensions::Base, render},
};
use lunar_engine_derive::as_any;

struct CameraControls {}

impl Component for CameraControls {
    #[as_any]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn update(&mut self) {
        if input::key(winit::keyboard::KeyCode::Space) == KeyState::Down {
            input::set_cursor_grab_mode(match input::get_cursor_grab_mode() {
                CursorLock::Locked => CursorLock::Free,
                CursorLock::Free => CursorLock::Locked,
            });
        }
    }
}

#[derive(Default)]
struct State {
    extension: Base,
    asset_store: AssetStore,
    world: World,
}

fn end(state: &mut State) {}

fn init(state: &mut State) {
    let world = &mut state.world;

    world.add_entity(
        EntityBuilder::new()
            .add_component::<Transform>()
            .add_component::<MainCamera>()
            .add_component::<CameraControls>()
            .create()
            .unwrap(),
    );
}

fn run(state: &mut State) {
    state.world.update();
    render(
        &state.world,
        &state.asset_store,
        &mut [&mut state.extension],
    )
}

fn main() {
    let state = lunar_engine::State::new(State::default());
    state.run(init, run, end);
}
