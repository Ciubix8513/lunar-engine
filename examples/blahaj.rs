use std::path::Path;

use log::info;
use lunar_engine::{
    asset_managment::AssetStore,
    assets::{self, materials::TextureUnlit, Material},
    components::{camera::MainCamera, mesh::Mesh, transform::Transform},
    ecs::{EntityBuilder, World},
    math::vec3::Vec3,
    system::rendering::{self, extensions::Base},
    State,
};

#[derive(Default)]
struct MyState {
    frame: u32,
    world: World,
    assset_store: AssetStore,
}

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
    state.world.add_entity(
        EntityBuilder::new()
            .add_component::<Transform>()
            .create_component(|| Mesh::new(mesh, material))
            .create(),
    );
    state.world.add_entity(
        EntityBuilder::new()
            .create_component(|| Transform {
                position: Vec3::new(0.0, 0.0, 4.0),
                rotation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            })
            .create_component(|| {
                let mut c = MainCamera::default();
                c.fov = 60.0;
                c.near = 0.1;
                c.far = 100.0;
                c
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
