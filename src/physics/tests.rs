use crate::{
    components::{
        physics::{Collider, PhysObject},
        transform::Transform,
    },
    ecs::{EntityBuilder, World},
    math::Quaternion,
    physics::PhysicsState,
};

#[test]
fn setup() {
    let mut world = World::new();

    for _ in 0..10 {
        world
            .add_entity(
                EntityBuilder::new()
                    .add_component::<Transform>()
                    .add_component::<Collider>()
                    .create()
                    .unwrap(),
            )
            .unwrap();

        let t = world
            .add_entity(
                EntityBuilder::new()
                    .add_component::<Transform>()
                    .add_component::<Collider>()
                    .add_component::<PhysObject>()
                    .create()
                    .unwrap(),
            )
            .unwrap()
            .upgrade()
            .unwrap()
            .read()
            .get_component()
            .unwrap();

        world
            .add_entity(
                EntityBuilder::new()
                    .add_existing_component(Transform::with_parent(
                        0.0.into(),
                        Quaternion::default(),
                        1.0.into(),
                        t,
                    ))
                    .add_component::<Collider>()
                    .create()
                    .unwrap(),
            )
            .unwrap();
    }

    let mut phys = PhysicsState::new();

    phys.set_up(&mut world);
}
