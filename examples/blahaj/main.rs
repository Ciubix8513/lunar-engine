use std::path::Path;

use log::info;
use lunar_engine::{
    asset_managment::AssetStore,
    assets::{self, materials::TextureUnlit, Material},
    components::{camera::MainCamera, mesh::Mesh, transform::Transform},
    ecs::{EntityBuilder, World},
    input,
    math::vec3::Vec3,
    system::rendering::{self, extensions::Base},
    State,
};
use proc_macros::marker_component;
use winit::keyboard::KeyCode;

use crate::camera_movement::FreeCam;

#[derive(Default)]
struct MyState {
    frame: u32,
    world: World,
    assset_store: AssetStore,
    blahaj_mesh: u128,
    blahaj_mat: u128,
}

mod camera_movement;

#[marker_component]
struct Blahaj;

fn init(state: &mut MyState) {
    state.frame = 0;
    let mesh = state
        .assset_store
        .register(assets::Mesh::new_from_obj(Path::new("assets/blahaj.obj")).unwrap());
    let texture = state
        .assset_store
        .register(assets::Texture::new_bmp(Path::new("assets/blahaj1.bmp")));
    let material = state
        .assset_store
        .register::<Material>(TextureUnlit::new(texture).into());

    state.blahaj_mat = material;
    state.blahaj_mesh = mesh;
    let e = state.world.add_entity(
        EntityBuilder::new()
            .create_component(|| Transform {
                position: Vec3::new(0.0, 2.0, 10.0),
                rotation: Vec3::new(-15.0, 0.0, 0.0),
                ..Default::default()
            })
            .create_component(|| {
                let mut c = MainCamera::default();
                //60 degree FOV
                c.fov = std::f32::consts::FRAC_PI_3;
                c.near = 0.1;
                c.far = 100.0;
                c
            })
            .create_component(|| FreeCam {
                speed: 10.0,
                sensetivity: 1.0,
            })
            .create(),
    );
    info!("Initialized!");
    info!("World contains {} entities", state.world.get_entity_count());

    if state.world.get_all_components::<MainCamera>().is_some() {
        info!("World contains a main camera");
    }
}

fn run(state: &mut MyState) {
    if input::KeyState::Down == input::key(KeyCode::KeyS) {
        state.world.add_entity(
            EntityBuilder::new()
                .create_component(|| Transform {
                    // scale: Vec3::random(0.3, 3.0),
                    rotation: Vec3::random(0.0, 360.0),
                    position: Vec3::random(-5.0, 5.0),
                    ..Default::default()
                })
                .create_component(|| Mesh::new(state.blahaj_mesh, state.blahaj_mat))
                .add_component::<Blahaj>()
                .create(),
        );
    }

    if input::KeyState::Down == input::key(KeyCode::KeyC) {
        let e = state
            .world
            .get_all_entities_with_component::<Blahaj>()
            .unwrap_or_default();
        for b in e {
            let id = b.borrow().get_id();
            state.world.remove_entity_by_id(id).unwrap();
        }
    }

    state.world.update();
    rendering::render(&state.world, &state.assset_store, &[&Base::new(0)]);
    state.frame += 1;
}

fn close(state: &mut MyState) {
    info!("Ran for {} frames", state.frame);
}

fn main() {
    let state = State::<MyState>::default();
    state.run(init, run, close);
}
