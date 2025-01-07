use log::info;
use lunar_engine::{
    asset_managment::AssetStore,
    assets::{self, materials::ColorUnlit, mesh::SphereData},
    components::{camera::MainCamera, fps::FpsRecorder, mesh::Mesh, transform::Transform},
    delta_time,
    ecs::{Component, ComponentReference, EntityBuilder, World},
    input::{self, CursorLock, CursorVisibily, KeyState},
    math::{lerp, Mat4x4, Vec3, Vector},
    rendering::{extensions::frustum_culling::Base, render},
    structures::Color,
};
use lunar_engine_derive::as_any;
use rand::{Rng, SeedableRng};
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

        let delta_time = delta_time();

        //Rotation
        let sensetivity = 300.0;
        let delta = input::cursor_delta() * delta_time * sensetivity;
        let mut trans = self.transform.as_ref().unwrap().borrow_mut();
        let rot = trans.rotation;

        let rot_y = lerp(rot.y, rot.y - delta.x, 0.1);
        let rot_x = lerp(rot.x, rot.x + delta.y, 0.1);
        trans.rotation.y = rot_y;
        trans.rotation.x = rot_x; //* delta_time * sensetivity;

        //Movement
        let mut speed = 400.0;
        if input::key(KeyCode::ShiftLeft) == KeyState::Pressed {
            speed *= 2.0;
        }

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

        movement_vec *= 0.01 * speed * delta_time;

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
    frames: u64,
    delta: f32,
}

fn end(_state: &mut State) {
    // state.world.
}

fn generate_scene(world: &mut World, assets: &mut AssetStore, num_objects: u32, num_colors: u32) {
    let objects = [
        assets.register(assets::Mesh::new_box(Vec3::new(1.0, 1.0, 1.0))),
        assets.register(assets::Mesh::new_sphere(SphereData {
            radius: 0.5,
            rings: 16,
            segments: 32,
        })),
    ];

    let mut colors = Vec::new();
    let mut rng = rand::rngs::StdRng::from_seed(Default::default());

    for _ in 0..num_colors {
        // colors.push(assets.register(ColorUnlit::new(Vec3::random(0.0, 1.0).into())));
        colors.push(assets.register(ColorUnlit::new(Color::from_hsl(
            rng.gen_range(0.0..360.0),
            rng.gen_range(0.5..1.0),
            rng.gen_range(0.4..0.8),
        ))));
    }

    for _ in 0..num_objects {
        let obj_id = objects[rng.gen_range(0..objects.len())];
        let mat_id = colors[rng.gen_range(0..colors.len())];

        world.add_entity(
            EntityBuilder::new()
                .create_component(|| Transform {
                    position: Vec3::random_with_rng(-20.0, 20.0, &mut rng),
                    rotation: Vec3::random_with_rng(-180.0, 180.0, &mut rng),
                    scale: Vec3::random_with_rng(0.3, 1.5, &mut rng),
                    ..Default::default()
                })
                .create_component(|| Mesh::new(obj_id, mat_id))
                .create()
                .unwrap(),
        );
    }
}

fn init(state: &mut State) {
    let assets = &mut state.asset_store;
    let world = &mut state.world;

    let args = std::env::args().collect::<Vec<_>>();

    let mut num_objects = 400;
    let mut num_colors = 20;
    if let Some(num_obj) = args.get(1) {
        num_objects = num_obj.parse().unwrap_or(num_objects);
    }
    if let Some(num_col) = args.get(2) {
        num_colors = num_col.parse().unwrap_or(num_colors);
    }

    info!("Num of objects: {num_objects}");
    info!("Num of colors: {num_colors}");

    generate_scene(world, assets, num_objects, num_colors);

    world.add_entity(
        EntityBuilder::new()
            .create_component(|| Transform {
                position: (0.0, 0.0, -4.0).into(),
                rotation: (0.0, 0.0, 0.0).into(),
                ..Default::default()
            })
            .add_component::<MainCamera>()
            .add_component::<CameraControls>()
            .add_component::<FpsRecorder>()
            .create()
            .unwrap(),
    );

    assets.intialize_all().unwrap();
}

fn run(state: &mut State) {
    state.world.update();
    render(
        &state.world,
        &state.asset_store,
        &mut [&mut state.extension],
    );
    state.frames += 1;
    state.delta += delta_time();

    if state.delta >= 5.0 {
        log::info!("Delta = {}", state.delta);
        lunar_engine::quit();
    }
}

fn main() {
    let state = lunar_engine::State::new(State::default());

    state.run(init, run, end);
}
