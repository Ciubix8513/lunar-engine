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
        physics::colliders,
        transform::Transform,
    },
    delta_time,
    ecs::{Component, ComponentReference, Entity, EntityBuilder, World},
    input::{self, CursorLock, CursorVisibily, KeyState},
    math::{Quaternion, Vec3, Vector},
    rendering::{
        extensions::{self, Base},
        render,
    },
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

        //Using a parent for y axis rotation...
        //
        //kinda scuffed but eh, should work
        let parent = trans.parent.clone().unwrap();
        let mut p = parent.borrow_mut();

        // trans.rotate((delta.y * 0.1, delta.x * -0.1, 0.0).into());
        trans.rotate((delta.y * 0.1, 0.0, 0).into());
        p.rotate((0, delta.x * -0.1, 0).into());

        drop(p);

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

        let mat = trans.rotation_global().matrix();
        movement_vec = mat.transform3(movement_vec);

        parent.borrow_mut().position += movement_vec;
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
    collision_ext: extensions::Collider,
    rendering_colliders: bool,
    only_render_colliders: bool,
    asset_store: AssetStore,
    world: World,
    frames: u64,
    delta: f32,
}

const fn end(_state: &mut State) {}

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
        assets.register(assets::Mesh::new_box(Vec3::new(1, 1, 1))),
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
                        position: Vec3::random_with_rng(-20, 20, &mut rng),
                        rotation: Quaternion::from_euler(Vec3::random_with_rng(
                            -180, 180, &mut rng,
                        )),
                        scale: Vec3::random_with_rng(0.3, 1.5, &mut rng),
                        ..Default::default()
                    })
                    .create_component(|| Mesh::new(obj_id, mat_id))
                    .create_component(|| colliders::Sphere::new(0.5))
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
                        position: Vec3::random_with_rng(-20, 20, &mut rng),
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

    let e = world
        .add_entity(
            EntityBuilder::new()
                .create_component(|| Transform {
                    position: (0, 0, -4).into(),
                    ..Default::default()
                })
                .create()
                .unwrap(),
        )
        .unwrap()
        .upgrade()
        .unwrap();
    let t = e.borrow().get_component::<Transform>().unwrap();

    drop(e);

    world
        .add_entity(
            EntityBuilder::new()
                .create_component(|| Transform {
                    parent: Some(t),
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
                    direction: Vec3::new(0, -1, 0),
                    ambient_color: Color::new(0.05, 0.05, 0.05, 1),
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

    if lunar_engine::input::key(KeyCode::KeyC) == KeyState::Down {
        state.rendering_colliders = !state.rendering_colliders;
    }

    if lunar_engine::input::key(KeyCode::KeyH) == KeyState::Down {
        state.only_render_colliders = !state.only_render_colliders;
        state
            .collision_ext
            .only_render_colliders(state.only_render_colliders);
    }

    if !state.rendering_colliders {
        render(
            &state.world,
            &mut state.asset_store,
            &mut [&mut state.extension],
        );
    } else {
        render(
            &state.world,
            &mut state.asset_store,
            &mut [&mut state.extension, &mut state.collision_ext],
        );
    }
    state.frames += 1;
    state.delta += delta_time();
}

fn main() {
    let state = lunar_engine::State::new(State::default());

    state.run(init, run, end);
}
