use crate::{
    components::{
        physics::{PhysObject, colliders::Box},
        transform::Transform,
    },
    ecs::{EntityBuilder, World},
};

use super::check_collisions;

#[test]
fn test_no_collisions() {
    let mut world = World::new();
    let e = world
        .add_entity(
            EntityBuilder::new()
                .add_component::<Transform>()
                .create()
                .unwrap(),
        )
        .unwrap();

    //There's nothing physics related in the world so no collisions :3
    let c = check_collisions(&world);
    assert!(c.is_empty());

    e.upgrade()
        .unwrap()
        .borrow_mut()
        .add_component::<Box>()
        .unwrap();

    //There are no physics objects in the world, so no collisions
    let c = check_collisions(&world);
    assert!(c.is_empty());

    e.upgrade()
        .unwrap()
        .borrow_mut()
        .add_component::<PhysObject>()
        .unwrap();

    //There's just one object in the  world, so  no collisions
    let c = check_collisions(&world);
    assert!(c.is_empty());

    let e1 = world
        .add_entity(
            EntityBuilder::new()
                .add_component::<Transform>()
                .add_component::<PhysObject>()
                .create()
                .unwrap(),
        )
        .unwrap();

    //There's just one collider in the  world, so  no collisions
    let c = check_collisions(&world);
    assert!(c.is_empty());

    e1.upgrade()
        .unwrap()
        .borrow_mut()
        .add_component::<Box>()
        .unwrap();

    //Finally there are 2 objects, so there must be collisions
    let c = check_collisions(&world);
    assert!(!c.is_empty());
}
