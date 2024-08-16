use lunar_engine::{
    asset_managment::AssetStore,
    assets::{
        self,
        materials::{self, ColorUnlit},
        Material,
    },
    components::{camera::MainCamera, mesh::Mesh, transform::Transform},
    ecs::{Component, ComponentReference, EntityBuilder, World},
    import,
    input::{self, CursorLock, KeyState},
    math::{lerp, Vector},
    rendering::{extensions::Base, render},
    structures::Color,
};
use lunar_engine_derive::as_any;

struct CameraControls {
    transform: Option<ComponentReference<Transform>>,
}

impl Component for CameraControls {
    #[as_any]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self { transform: None }
    }

    fn update(&mut self) {
        if input::key(winit::keyboard::KeyCode::Space) == KeyState::Down {
            input::set_cursor_grab_mode(match input::get_cursor_grab_mode() {
                CursorLock::Locked => CursorLock::Free,
                CursorLock::Free => CursorLock::Locked,
            });
        }

        if input::key(winit::keyboard::KeyCode::KeyH) == KeyState::Down {
            input::set_cursor_visible(match input::get_cursor_visibility() {
                input::CursorVisibily::Visible => input::CursorVisibily::Hidden,
                input::CursorVisibily::Hidden => input::CursorVisibily::Visible,
            });
        }

        let delta = input::cursor_delta();

        let mut trans = self.transform.as_ref().unwrap().borrow_mut();

        let rot = trans.rotation;

        let rot_y = lerp(rot.y, rot.y - delta.x, 0.1);
        let rot_x = lerp(rot.x, rot.x - delta.y, 0.1);

        trans.rotation.y = rot_y;
        trans.rotation.x = rot_x;
    }

    fn set_self_reference(&mut self, reference: lunar_engine::ecs::SelfReferenceGuard) {
        self.transform = reference.get_component::<Transform>().ok();
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
    let assets = &mut state.asset_store;
    let white = assets.register(ColorUnlit::new(Color::white()));
    let cube = assets.register(assets::Mesh::new_from_static_obj(include_str!(
        "../../assets/cube_triangulated.obj"
    )));

    let camera = MainCamera::mew();
    let world = &mut state.world;

    world.add_entity(
        EntityBuilder::new()
            .create_component(|| Transform {
                position: (0.0, 0.0, 10.0).into(),

                ..Default::default()
            })
            .add_existing_component(camera)
            .add_component::<CameraControls>()
            .create()
            .unwrap(),
    );

    world.add_entity(
        EntityBuilder::new()
            .add_component::<Transform>()
            .create_component(|| Mesh::new(cube, white))
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
