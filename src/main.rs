use std::path::Path;

use log::info;
use lunar_lib::{
    asset_managment::AssetStore,
    assets::{self, materials::TextureUnlit, Material},
    components::transform::Transform,
    ecs::{EntityBuilder, World},
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
    let mesh = state.assset_store.register(
        assets::Mesh::new_from_obj(Path::new("../assets/cube_triangulated.obj")).unwrap(),
    );
    let texture = state
        .assset_store
        .register(assets::Texture::new_bmp(Path::new("../assets/blahaj1.bmp")));
    let material = state
        .assset_store
        .register::<Material>(TextureUnlit::new(texture).into());
    state
        .world
        .add_entity(EntityBuilder::new().add_component::<Transform>().create());

    info!("Initialized!")
}
fn run(state: &mut MyState) {
    state.frame += 1;
}

fn close(state: &mut MyState) {
    info!("Ran for {} frames", state.frame);
}

fn main() {
    let state = State::<MyState>::default();
    state.run(init, run, close);
}
