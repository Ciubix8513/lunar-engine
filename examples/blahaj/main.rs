use std::path::Path;

use log::{debug, info};
use lunar_engine::{
    asset_managment::AssetStore,
    assets::{self, materials::TextureUnlit, Material},
    components::{camera::MainCamera, mesh::Mesh, transform::Transform},
    ecs::{Component, ComponentReference, EntityBuilder, World},
    input,
    math::vec3::Vec3,
    system::rendering::{self, extensions::Base},
    State,
};
use lunar_engine_derive::{as_any, dependencies, marker_component};
use winit::keyboard::KeyCode;

use crate::camera_movement::FreeCam;

#[derive(Default)]
struct MyState {
    frame: u32,
    world: World,
    assset_store: AssetStore,
    extension: Base,
    blahaj_mesh: u128,
    blahaj_mat: u128,
}

mod camera_movement;

#[marker_component]
struct Blahaj;

struct Spiny {
    pub speed: f32,
    transform: Option<ComponentReference<Transform>>,
}

impl Component for Spiny {
    #[as_any]
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            speed: 100.0,
            transform: None,
        }
    }

    fn set_self_reference(&mut self, reference: lunar_engine::ecs::SelfReferenceGuard) {
        self.transform = Some(reference.get_component().unwrap());
    }

    fn update(&mut self) {
        self.transform.as_ref().unwrap().borrow_mut().rotation.y +=
            self.speed * lunar_engine::delta_time();
    }
}

fn init(state: &mut MyState) {
    log::info!("Initializing scene");

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
    let _e = state.world.add_entity(
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
            .create_component(|| FreeCam::new(10.0))
            .create()
            .unwrap(),
    );

    state.world.add_entity(
        EntityBuilder::new()
            .create_component(|| Transform::default())
            .create_component(|| Mesh::new(state.blahaj_mesh, state.blahaj_mat))
            .add_component::<Blahaj>()
            .add_component::<Spiny>()
            .create()
            .unwrap(),
    );

    info!("Initialized!");
    info!("World contains {} entities", state.world.get_entity_count());

    if state.world.get_all_components::<MainCamera>().is_some() {
        info!("World contains a main camera");
    }
}

fn run(state: &mut MyState) {
    if input::KeyState::Down == input::key(KeyCode::KeyB) {
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
                .create()
                .unwrap(),
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
    debug!("Called render!");
    rendering::render(
        &state.world,
        &state.assset_store,
        &mut [&mut state.extension],
    );
    state.frame += 1;
}

fn close(state: &mut MyState) {
    info!("Ran for {} frames", state.frame);
}

fn main() {
    let state = State::<MyState>::default();
    state.run(init, run, close);
}
