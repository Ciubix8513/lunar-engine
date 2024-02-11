//! Contains the rendering system, used with the ecs
//!
//! # Rendering system
//!
//! A scene consists of a world and an asset store.
//! The asset store consists ONLY of the assets needed for the scene, nothing else.
//! The render function accepts a world and an asset store
//! The rendering function gets the asset ids and queries them from the store

use log::info;

use crate::ecs::World;

///Renders all the entities in the world
pub fn render(world: &World) {
    let meshes = world.get_all_components::<crate::components::mesh::Mesh>();

    //No rendering needs to be done
    if meshes.is_none() {
        info!("Rendered 0 meshes");
        return;
    }
    todo!();
}
