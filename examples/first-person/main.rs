use lunar_engine::{
    asset_managment::AssetStore,
    assets::{self, materials::ColorUnlit, mesh::SphereData},
    components::{camera::MainCamera, mesh::Mesh, transform::Transform},
    ecs::{Component, ComponentReference, EntityBuilder, World},
    input::{self, CursorLock, CursorVisibily, KeyState},
    math::{lerp, Mat4x4, Vec3, Vector},
    rendering::{extensions::Base, render},
    structures::Color,
};
use lunar_engine_derive::as_any;
use winit::keyboard::KeyCode;

struct CameraControls {
    transform: Option<ComponentReference<Transform>>,
}

impl Component for CameraControls {
    #[as_any]

    fn mew() -> Self
    where
        Self: Sized,
    {
        input::set_cursor_grab_mode(CursorLock::Locked);
        input::set_cursor_visible(CursorVisibily::Hidden);
        Self { transform: None }
    }

    fn update(&mut self) {
        //Cursor stuff
        if input::key(KeyCode::Escape) == KeyState::Down {
            input::set_cursor_grab_mode(CursorLock::Free);
            input::set_cursor_visible(CursorVisibily::Visible);
        }

        if input::mouse_btn(winit::event::MouseButton::Left) == KeyState::Down {
            input::set_cursor_grab_mode(CursorLock::Locked);
            input::set_cursor_visible(CursorVisibily::Hidden);
        }

        //Rotation
        let delta = input::cursor_delta();

        let mut trans = self.transform.as_ref().unwrap().borrow_mut();
        let rot = trans.rotation;

        let rot_y = lerp(rot.y, rot.y - delta.x, 0.1);
        let rot_x = lerp(rot.x, rot.x - delta.y, 0.1);
        trans.rotation.y = rot_y;
        trans.rotation.x = rot_x;

        //Movement
        let mut movement_vec = Vec3::default();
        if input::key(KeyCode::KeyW) == KeyState::Pressed {
            movement_vec.z += 1.0;
        }
        if input::key(KeyCode::KeyS) == KeyState::Pressed {
            movement_vec.z -= 1.0;
        }
        if input::key(KeyCode::KeyA) == KeyState::Pressed {
            movement_vec.x += 1.0;
        }
        if input::key(KeyCode::KeyD) == KeyState::Pressed {
            movement_vec.x -= 1.0;
        }
        if input::key(KeyCode::KeyE) == KeyState::Pressed {
            movement_vec.y += 1.0;
        }
        if input::key(KeyCode::KeyQ) == KeyState::Pressed {
            movement_vec.y -= 1.0;
        }

        if movement_vec.square_length() == 0.0 {
            return;
        }

        movement_vec *= 0.01;

        let mat = Mat4x4::rotation_matrix_euler(&trans.rotation);
        movement_vec = mat.transform3(movement_vec);
        trans.position += movement_vec;
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

fn end(_: &mut State) {}

fn generate_scene(world: &mut World, assets: &mut AssetStore, num_objects: u32, num_colors: u32) {
    let white = assets.register(ColorUnlit::new(Color::white()));
    let cube = assets.register(assets::Mesh::new_box(Vec3::new(1.0, 1.0, 1.0)));
    let sphere = assets.register(assets::Mesh::new_sphere(SphereData {
        radius: 0.5,
        rings: 16,
        segments: 32,
    }));

    world.add_entity(
        EntityBuilder::new()
            .add_component::<Transform>()
            .create_component(|| Mesh::new(cube, white))
            .create()
            .unwrap(),
    );
}

fn init(state: &mut State) {
    let assets = &mut state.asset_store;
    let world = &mut state.world;

    generate_scene(world, assets, 10, 4);

    world.add_entity(
        EntityBuilder::new()
            .create_component(|| Transform {
                position: (0.0, 0.0, -4.0).into(),
                rotation: (0.0, 0.0, 0.0).into(),
                ..Default::default()
            })
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
