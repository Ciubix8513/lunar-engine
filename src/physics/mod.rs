//! Physics simulation :3
//!
//!

use std::collections::VecDeque;

use crate::{
    UUID,
    components::{
        physics::{PhysObject, colliders},
        transform::Transform,
    },
    ecs::{ComponentReference, Entity, EntityRefence, World},
    math::{Mat4x4, Vec3, Vector},
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

#[derive(Default, Debug)]
///Some data describing behavior of the physics
pub struct PhysicsData {}

#[derive(Default, Debug)]
///Additional data about what should collide with what and how
pub struct CollisionData {}

fn triangular_num(a: usize) -> usize {
    (a * (a + 1)) / 2
}

enum Collider {
    Box(ComponentReference<colliders::Box>),
    Sphere(ComponentReference<colliders::Sphere>),
    Capsule(ComponentReference<colliders::Capsule>),
}

//I hope this is enough data to handle everything
struct Object {
    entity: EntityRefence,
    entity_id: UUID,
    collider: Collider,
    transform: ComponentReference<Transform>,
    position: Vec3,
    scale: Vec3,
    rotation: Vec3,
    transform_matrix: Mat4x4,
}

//For collision detection we only care about whether the objects collide, not where exactly they do
//that, exact position only matters for raycasting
fn check_collision(obj_a: &Object, obj_b: &Object) -> bool {
    //this is awful and i hate it
    match &obj_a.collider {
        Collider::Box(component_reference) => match &obj_b.collider {
            Collider::Box(component_reference) => todo!(),
            Collider::Sphere(component_reference) => todo!(),
            Collider::Capsule(component_reference) => todo!(),
        },
        Collider::Sphere(component_reference_a) => match &obj_b.collider {
            Collider::Box(component_reference_b) => todo!(),
            //The only simple  case
            Collider::Sphere(component_reference_b) => {
                let direction = obj_a.position - obj_b.position;
                let len = direction.length();

                //Normalized direction towards the other sphere
                let dir_normalized = direction / len;

                //Now we multiply  it by the scale of the object and the  radius
                let radius_a =
                    (dir_normalized * obj_a.scale * component_reference_a.borrow().radius).length();
                let radius_b =
                    (dir_normalized * obj_b.scale * component_reference_b.borrow().radius).length();

                len - (radius_a + radius_b) <= 0.0
            }
            Collider::Capsule(component_reference_b) => todo!(),
        },

        Collider::Capsule(component_reference) => match &obj_b.collider {
            Collider::Box(component_reference) => todo!(),
            Collider::Sphere(component_reference) => todo!(),
            Collider::Capsule(component_reference) => todo!(),
        },
    }
}

///Checks collisions in a world
pub fn process_collisions(world: &World) -> Vec<Collision> {
    #[cfg(feature = "tracy")]
    let _span = tracy_client::span!("collision detection");

    let mut objects = world
        .get_all_entities_with_component::<PhysObject>()
        .unwrap_or_default()
        .iter()
        .map(|i| i.borrow().get_id())
        .collect::<Vec<_>>();

    //No physics objects in the world, therefore  no collisions
    if objects.is_empty() {
        return Vec::new();
    }

    let colliders = world
        .get_all_entities_with_component::<colliders::Box>()
        .unwrap_or_default()
        .into_iter()
        .map(|i| {
            (
                i.clone(),
                i.borrow().get_id(),
                Collider::Box(i.borrow().get_component().unwrap()),
            )
        })
        .chain(
            world
                .get_all_entities_with_component::<colliders::Sphere>()
                .unwrap_or_default()
                .into_iter()
                .map(|i| {
                    (
                        i.clone(),
                        i.borrow().get_id(),
                        Collider::Sphere(i.borrow().get_component().unwrap()),
                    )
                })
                .chain(
                    world
                        .get_all_entities_with_component::<colliders::Capsule>()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|i| {
                            (
                                i.clone(),
                                i.borrow().get_id(),
                                Collider::Capsule(i.borrow().get_component().unwrap()),
                            )
                        }),
                ),
        )
        .map(|i| {
            let t = i.0.borrow().get_component::<Transform>().unwrap();

            Object {
                entity: i.0,
                entity_id: i.1,
                collider: i.2,
                transform: t.clone(),
                scale: t.borrow().scale_global(),
                position: t.borrow().position_global(),
                rotation: t.borrow().rotation;
                transform_matrix: t.borrow().matrix(),
            }
        });

    //A bit of a hack for performance reasons?
    let s = colliders.size_hint();
    let s = s.1.unwrap_or(s.0);

    println!("{s}");

    //Less than 2 colliders therefore no collisions
    if s <= 1 {
        return Vec::new();
    }

    let (objs, colliders): (Vec<_>, Vec<_>) = colliders.partition(|i| {
        let mut found = false;
        let mut ind = None;

        for (idx, j) in objects.iter().enumerate() {
            if &i.entity_id == j {
                found = true;
                ind = Some(idx);
                break;
            }
        }

        //Already found this element so can remove it from the object list
        if let Some(ind) = ind {
            objects.swap_remove(ind);
        }

        found
    });
    //I think that covers all the simple cases...
    //Now the  actually difficult stuff

    //
    //T_(N - 1) + (M - N) *N
    //
    //N = Number of phys objects
    //M = Total number of colliders
    //
    //but since we already separated objects and colliders we can just do T_(N - 1) + M *N

    //Total number of collision checks, without optimisations
    let num_col = triangular_num(objs.len() - 1) + colliders.len() * objs.len();

    let mut collisions = Vec::new();

    log::info!("Total collision checks without optimisations:  {num_col}");

    #[cfg(feature = "tracy")]
    let check_phys_obj = tracy_client::span!("Checking Phys Objects");

    //Need to check collisions between phys objects, so to optimally do it we just need to
    //only check each pair once:
    //A B C
    //B C D
    //C D
    //D
    //
    //Here we first check AB, AC, AD, after which we check BC, BD, and finally just need to check CD

    //A list without the first item
    let mut list_obj = objs.iter().skip(1).collect::<VecDeque<_>>();

    //Iterate over all objects skipping the last one
    for i in &objs[..objs.len() - 2] {
        //Iterate over other objects
        for j in &list_obj {
            //Check collision between the 2
            if check_collision(i, j) {
                collisions.push(Collision {
                    object_a: i.entity.clone(),
                    object_b: j.entity.clone(),
                });
            }
        }
        //Remove the first object since we already checked it
        list_obj.pop_front();
    }

    #[cfg(feature = "tracy")]
    drop(check_phys_obj);

    return collisions;
}
