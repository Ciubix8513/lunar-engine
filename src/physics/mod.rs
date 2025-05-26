//! Physics simulation :3
//!
//!

use crate::{
    components::physics::{PhysObject, colliders},
    ecs::{EntityRefence, World},
};

#[cfg(test)]
mod tests;

///Describes a collision between 2 objects
pub struct Collision {
    ///Object A
    pub object_a: EntityRefence,
    ///Object B
    pub object_b: EntityRefence,
}

fn triangular_num(a: u64) -> u64 {
    let mut o = 0;

    for i in 0..a {
        o += i + 1;
    }

    o
}

///Checks collisions in a world
pub fn check_collisions(world: &World) -> Vec<Collision> {
    let objects = world.get_all_components::<PhysObject>().unwrap_or_default();

    //No physics objects in the world, therefore  no collisions
    if objects.is_empty() {
        return Vec::new();
    }

    let boxes = world
        .get_all_components::<colliders::Box>()
        .unwrap_or_default();
    let spheres = world
        .get_all_components::<colliders::Sphere>()
        .unwrap_or_default();
    let capsules = world
        .get_all_components::<colliders::Capsule>()
        .unwrap_or_default();

    //Less than 2 colliders therefore no collisions
    if boxes.len() + spheres.len() + capsules.len() <= 1 {
        return Vec::new();
    }

    //I think that covers all the simple cases...
    //Now the  actually difficult stuff

    //Total number of collision checks, without optimisations
    let num_col = triangular_num(objects.len() as u64 - 1);

    return Vec::new();
}
