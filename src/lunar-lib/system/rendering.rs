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
    //This is cached, so should be reasonably fast
    let meshes = world.get_all_components::<crate::components::mesh::Mesh>();
    //Need to be able to get a component from the same entity
    //Oh god i'm gonna have to pass the entity reference to the component...

    //No rendering needs to be done
    //NO there IS work to be done here, like theskybox and shit
    if meshes.is_none() {
        info!("Rendered 0 meshes");
        return;
    }

    todo!();
}
