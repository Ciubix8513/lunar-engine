mod camera;
mod ray_cast;

use std::sync::OnceLock;

use lunar_engine::{
    UUID,
    asset_managment::AssetStore,
    assets,
    components::{
        self,
        camera::MainCamera,
        light::DirectionalLight,
        physics::{Collider, PhysObject},
        transform::Transform,
    },
    delta_time,
    ecs::{Component, ComponentReference, Entity, EntityBuilder, World},
    input::{self, CursorLock, CursorVisibily, KeyState},
    math::{Quaternion, Vec3, Vector},
    physics::PhysicsState,
    rendering::{
        extensions::{self, Base, RenderingExtension},
        render,
    },
    structures::Color,
};
use winit::keyboard::KeyCode;

struct CameraControls {
    transform: OnceLock<ComponentReference<Transform>>,
}

#[derive(Default)]
struct State {
    world: World,
    assets: AssetStore,
    extension: Base,
    dbg_ext: extensions::Debug,
    screenshot_ext: extensions::screenshot::Screenshot,
    phys_world: PhysicsState,
    world_running: bool,

    mesh_hndl: UUID,
    mat_hndl: UUID,

    lines: Vec<(Vec3, Vec3)>,
    boxes: Vec<(Vec3, Vec3)>,

    mat_hndl1: UUID,
}

fn run(state: &mut State) {
    state.world.update();

    state.phys_world.render(&mut state.dbg_ext);

    if input::key(KeyCode::Space) == KeyState::Down {
        state.world_running = !state.world_running;
    }

    if input::key(KeyCode::KeyU) == KeyState::Down {
        for o in state.world.get_all_components::<PhysObject>() {
            o.borrow().transform().borrow_mut().position.y += 5.0;
        }
    }

    if input::key(KeyCode::KeyN) == KeyState::Down {
        state
            .world
            .add_entity(
                EntityBuilder::new()
                    .create_component(|| {
                        Transform::new((0, 10, 0).into(), Quaternion::default(), 1.into())
                    })
                    .add_component::<PhysObject>()
                    .create_component(|| {
                        Collider::new(components::physics::Shape::Box {
                            dimensions: 0.5.into(),
                        })
                    })
                    .create_component(|| {
                        components::mesh::Mesh::new(state.mesh_hndl, state.mat_hndl)
                    })
                    .create()
                    .unwrap(),
            )
            .unwrap();
    }
    if input::key(KeyCode::KeyR) == KeyState::Down {
        for o in state.world.get_all_entities_with_component::<PhysObject>() {
            o.read()
                .get_component::<Transform>()
                .unwrap()
                .borrow_mut()
                .scale = Vec3::random(0.5, 1.25);
        }
    }

    if state.world_running || input::key(KeyCode::KeyO) == KeyState::Down {
        state.phys_world.step();
    }

    ray_cast::cast(state);

    for i in &state.lines {
        state.dbg_ext.draw_line(i.0, i.1, Color::red());
    }

    for i in &state.boxes {
        state
            .dbg_ext
            .draw_box(i.0, Quaternion::identity(), i.1, Color::red());
    }

    let ext: &mut [&mut dyn RenderingExtension] = if input::key(KeyCode::F12) == KeyState::Down {
        &mut [
            &mut state.extension,
            &mut state.dbg_ext,
            &mut state.screenshot_ext,
        ]
    } else {
        &mut [&mut state.extension, &mut state.dbg_ext]
    };

    render(&mut state.world, &mut state.assets, ext);
}
fn end(_: &mut State) {}

fn init(state: &mut State) {
    let assets = &mut state.assets;
    let world = &mut state.world;

    let b = assets.register(assets::Mesh::new_box(0.5.into()));
    let f = assets.register(assets::Mesh::new_box((10, 0.1, 10).into()));
    let m = assets.register(assets::materials::Lit::new(None, None, None, 0.5));

    state.mesh_hndl = b;
    state.mat_hndl = m;

    state.mat_hndl1 = assets.register(assets::materials::Lit::new(
        None,
        Some(Color::red()),
        None,
        0.5,
    ));

    let f_m = assets.register(assets::materials::Lit::new(
        None,
        Some(Into::<Vec3>::into(0.6).into()),
        None,
        0.0,
    ));

    let e = EntityBuilder::new()
        .add_existing_component(Transform::new(
            (0, 10, 0).into(),
            Quaternion::default(),
            0.0.into(),
        ))
        .create()
        .unwrap();
    let p = e.get_component().unwrap();

    world.add_entity(e).unwrap();
    world
        .add_entity(
            EntityBuilder::new()
                .create_component(|| {
                    let mut t = Transform::default();
                    t.set_parent(p);
                    t
                })
                .add_component::<MainCamera>()
                .add_component::<camera::CameraControls>()
                .create()
                .unwrap(),
        )
        .unwrap();

    world
        .add_entity(
            EntityBuilder::new()
                .create_component(|| DirectionalLight {
                    direction: Into::<Vec3>::into((-1, -1, -1)).normalized(),
                    intensity: 1.0,
                    ..Default::default()
                })
                // .add_component::<DirectionalLight>()
                .create()
                .unwrap(),
        )
        .unwrap();

    //Floor
    world
        .add_entity(
            EntityBuilder::new()
                .add_component::<Transform>()
                .create_component(|| components::mesh::Mesh::new(f, f_m))
                .create_component(|| {
                    Collider::new(components::physics::Shape::Box {
                        dimensions: (10, 0.1, 10).into(),
                    })
                })
                .create()
                .unwrap(),
        )
        .unwrap();

    //Boxes
    for i in 0..5 {
        world
            .add_entity(
                EntityBuilder::new()
                    .create_component(|| {
                        Transform::new(
                            (0, 5 + i * 3, 0).into(),
                            Quaternion {
                                w: -0.48387048,
                                x: 0.4455742,
                                y: -0.08311905,
                                z: -0.7486149,
                            }
                            .normalize(),
                            0.8.into(),
                        )
                    })
                    .add_component::<PhysObject>()
                    .create_component(|| components::mesh::Mesh::new(b, m))
                    .create_component(|| {
                        Collider::new(components::physics::Shape::Box {
                            dimensions: 0.5.into(),
                        })
                    })
                    .create()
                    .unwrap(),
            )
            .unwrap();
    }
    state.phys_world = PhysicsState::new();
    state.phys_world.set_gravity((0, -0.5, 0).into());

    state.phys_world.set_up(world);
}

fn main() {
    let state = lunar_engine::State::new(State::default());

    state.run(init, run, end);
}
