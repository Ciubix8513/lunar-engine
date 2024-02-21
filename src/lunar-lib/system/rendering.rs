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
    assets::{BindgroupState, Material},
    ecs::World,
    DEVICE,
};

///Renders all the entities in the world
pub fn render(world: &World, asset_store: &AssetStore) {
    //This is cached, so should be reasonably fast
    let meshes = world.get_all_components::<crate::components::mesh::Mesh>();
    // let main_camera = world.get_all_components::crate
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

    let mut materials = Vec::new();

    //Update the gpu data for every Mesh
    for m in meshes {
        let m = m.borrow();
        m.update_gpu(&mut encoder);
        materials.push(m.get_material_id().unwrap());
    }

    //Initialize bindgroups for all needed materials
    for m in materials {
        let m = asset_store.get_by_id::<Material>(m).unwrap();
        let mut m = m.borrow_mut();

        if let BindgroupState::Initialized = m.get_bindgroup_state() {
            continue;
        }

        m.initialize_bindgroups(asset_store);
    }

    todo!();
}
