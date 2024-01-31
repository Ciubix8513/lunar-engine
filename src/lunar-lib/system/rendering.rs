//! Contains the rendering system, used with the ecs

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
