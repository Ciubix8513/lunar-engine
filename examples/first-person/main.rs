use core::f32;
use std::cell::OnceCell;

use log::info;
use lunar_engine::{
    asset_managment::AssetStore,
    assets::{self, materials::Lit, mesh::SphereData},
    components::{
        camera::MainCamera,
        fps::FpsRecorder,
        light::{DirectionalLight, PointLight},
        mesh::Mesh,
        transform::Transform,
    },
    delta_time,
    ecs::{Component, ComponentReference, EntityBuilder, World},
    input::{self, CursorLock, CursorVisibily, KeyState},
    math::{Mat4x4, Vec3, Vector, lerp},
    rendering::{extensions::Base, render},
    structures::Color,
};
use rand::Rng;
use winit::keyboard::KeyCode;

struct CameraControls {
    transform: OnceCell<ComponentReference<Transform>>,
}

impl Component for CameraControls {
    fn mew() -> Self
    where
        Self: Sized,
    {
        input::set_cursor_grab_mode(CursorLock::Locked);
        input::set_cursor_visible(CursorVisibily::Hidden);
        Self {
            transform: OnceCell::new(),
        }
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
        let sensetivity = 800.0;
        let delta = input::cursor_delta() * delta_time * sensetivity;
        let mut trans = self.transform.get().unwrap().borrow_mut();
        let rot = trans.rotation;

        let rot_y = lerp(rot.y, rot.y - delta.x, 0.1);
        let rot_x = lerp(rot.x, rot.x + delta.y, 0.1);
        trans.rotation.y = rot_y;
        trans.rotation.x = rot_x;

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
        self.transform
            .set(reference.get_component::<Transform>().unwrap())
            .unwrap();
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

fn end(_state: &mut State) {}

fn generate_scene(
    world: &mut World,
    assets: &mut AssetStore,
    num_objects: u32,
    num_colors: u32,
    num_lights: u32,
) {
    let objects = [
        assets.register(assets::Mesh::new_from_static_obj(include_str!(
            "../../assets/blahaj.obj"
        ))),
        assets.register(assets::Mesh::new_box(Vec3::new(1.0, 1.0, 1.0))),
        assets.register(assets::Mesh::new_sphere(SphereData {
            radius: 0.5,
            rings: 16,
            segments: 32,
        })),
    ];

    let mut colors = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..num_colors {
        colors.push(assets.register(Lit::new(
            None,
            Some(Color::from_hsl(
                rng.gen_range(0.0..360.0),
                rng.gen_range(0.5..1.0),
                rng.gen_range(0.4..0.8),
            )),
            Some(Color::from_hsl(
                rng.gen_range(0.0..360.0),
                rng.gen_range(0.5..1.0),
                rng.gen_range(0.4..0.8),
            )),
            rng.gen_range(0.0..1.0),
        )));
    }

    for _ in 0..num_objects {
        let obj_id = objects[rng.gen_range(0..objects.len())];
        let mat_id = colors[rng.gen_range(0..colors.len())];

        world
            .add_entity(
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
            )
            .unwrap();
    }

    world
        .add_entity(
            EntityBuilder::new()
                .add_component::<Transform>()
                .create_component(|| Mesh::new(objects[0], colors[0]))
                .create()
                .unwrap(),
        )
        .unwrap();

    //Add N point lights in random positions
    for _ in 0..num_lights {
        world
            .add_entity(
                EntityBuilder::new()
                    .create_component(|| Transform {
                        position: Vec3::random_with_rng(-20.0, 20.0, &mut rng),
                        ..Default::default()
                    })
                    .create_component(|| {
                        PointLight::new(
                            Color::white(),
                            rng.gen_range(10.0..50.0),
                            rng.gen_range(8.0..25.0),
                        )
                    })
                    .create()
                    .unwrap(),
            )
            .unwrap();
    }
}

fn init(state: &mut State) {
    let assets = &mut state.asset_store;
    let world = &mut state.world;

    let args = std::env::args().collect::<Vec<_>>();

    let mut num_objects = 400;
    let mut num_colors = 20;
    let mut num_lights = 10;
    if let Some(num_obj) = args.get(1) {
        num_objects = num_obj.parse().unwrap_or(num_objects);
    }
    if let Some(num_col) = args.get(2) {
        num_colors = num_col.parse().unwrap_or(num_colors);
    }

    if let Some(num_l) = args.get(3) {
        num_lights = num_l.parse().unwrap_or(num_lights);
    }

    info!("Num of objects: {num_objects}");
    info!("Num of colors: {num_colors}");
    info!("Num of lights: {num_lights}");

    generate_scene(world, assets, num_objects, num_colors, 8);

    world
        .add_entity(
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
        )
        .unwrap();

    world
        .add_entity(
            EntityBuilder::new()
                //A  directional light no intensity, just so that there's some ambient light
                .create_component(|| DirectionalLight {
                    color: Color::white(),
                    direction: Vec3::new(0.0, -1.0, 0.0),
                    ambient_color: Color::new(0.05, 0.05, 0.05, 1.0),
                    intensity: 0.0,
                })
                .create()
                .unwrap(),
        )
        .unwrap();

    assets.intialize_all().unwrap();
}

fn run(state: &mut State) {
    state.world.update();
    render(
        &state.world,
        &mut state.asset_store,
        &mut [&mut state.extension],
    );
    state.frames += 1;
    state.delta += delta_time();
}

fn main() {
    let state = lunar_engine::State::new(State::default());

    state.run(init, run, end);
}
