//! Contains the rendering system, used with the ecs
//!
//! # Rendering system
//!
//! A scene consists of a world and an asset store.
//!
//! The asset store consists ONLY of the assets needed for the scene, nothing else.
//!
//! The render function accepts a world and an asset store
//!
//! The rendering function gets the asset ids and queries them from the store

use log::info;

use crate::{
    asset_managment::AssetStore,
    ecs::{Component, World},
    DEVICE,
};

///Renders all the entities in the world
pub fn render(world: &World, asset_store: &AssetStore) {
    //This is cached, so should be reasonably fast
    let meshes = world.get_all_components::<crate::components::mesh::Mesh>();
    //
    //No rendering needs to be done
    //NO there IS work to be done here, like the skybox and shit
    if meshes.is_none() {
        info!("Rendered 0 meshes");
        return;
    }
    let meshes = meshes.unwrap();
    let device = DEVICE.get().unwrap();
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    for m in meshes {
        let m = m.borrow();
        m.update_gpu(&mut encoder);
    }

    todo!();
}
