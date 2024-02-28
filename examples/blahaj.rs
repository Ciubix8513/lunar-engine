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
        .register(assets::Mesh::new_from_obj(Path::new("assets/cube_triangulated.obj")).unwrap());
    let texture = state
        .assset_store
        .register(assets::Texture::new_bmp(Path::new("../assets/blahaj1.bmp")));
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
                position: Vec3::new(0.0, 8.0, -8.0),
                rotation: Vec3::new(-45.0, 0.0, 0.0),
                ..Default::default()
            })
            .add_component::<MainCamera>()
            .create(),
    );
    info!("World contains {} entities", state.world.get_entity_count());
    info!("Initialized!")
}
fn run(state: &mut MyState) {
    info!("World contains {} entities", state.world.get_entity_count());
    state.world.update();
    //How to get attachments? hmmm
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
