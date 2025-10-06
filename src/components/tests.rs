use crate as lunar_engine;
use lunar_engine_derive::marker_component;

use super::{mesh::Mesh, transform::Transform};
use crate::{ecs::*, math::Quaternion};

#[test]
fn test_mesh() {
    crate::test_utils::generate_gpu();

    let mut e = Entity::new();

    e.add_component::<Mesh>().unwrap();

    let m = e.get_component::<Mesh>().unwrap();
    m.borrow_mut().set_mesh(123);
}

#[test]
fn test_transform() {
    let mut e = Entity::new();

    e.add_component::<Transform>().unwrap();

    let t = e.get_component::<Transform>().unwrap();
    _ = t.borrow_mut().matrix();
}

#[test]
fn parent_child() {
    let mut world = World::new();

    let p = EntityBuilder::new()
        .add_component::<Transform>()
        .create()
        .unwrap();
    let p_t = p.get_component().unwrap();

    let p_ref = world.add_entity(p).unwrap();

    let c = EntityBuilder::new()
        .create_component(|| {
            Transform::with_parent(0.into(), Quaternion::default(), 1.0.into(), p_t.clone())
        })
        .create()
        .unwrap();

    let c0_ref = world.add_entity(c).unwrap();

    let c = EntityBuilder::new()
        .add_component::<Transform>()
        .create()
        .unwrap();

    c.get_component::<Transform>()
        .unwrap()
        .borrow_mut()
        .set_parent(p_t.clone());

    let c1_ref = world.add_entity(c).unwrap();

    let c = EntityBuilder::new()
        .add_component::<Transform>()
        .create()
        .unwrap();

    let c2_ref = world.add_entity(c).unwrap();

    c2_ref
        .upgrade()
        .unwrap()
        .read()
        .get_component::<Transform>()
        .unwrap()
        .borrow_mut()
        .set_parent(p_t.clone());

    let binding = p_t.borrow();
    let c = binding.get_children();

    assert_eq!(c.len(), 3);
}

#[marker_component]
struct Marker;

#[test]
fn get_child_components() {
    let mut world = World::new();

    let mut e = Entity::new();
    e.add_component::<Transform>().unwrap();
    e.add_component::<Marker>().unwrap();
    let p = e.get_component().unwrap();

    world.add_entity(e).unwrap();

    let e1 = EntityBuilder::new()
        .add_component::<Marker>()
        .add_existing_component(Transform::with_parent(
            0.into(),
            Quaternion::default(),
            1.into(),
            p.clone(),
        ))
        .create()
        .unwrap();

    let p1 = e1.get_component().unwrap();

    world.add_entity(e1).unwrap();

    let e2 = EntityBuilder::new()
        .add_component::<Marker>()
        .add_existing_component(Transform::with_parent(
            0.into(),
            Quaternion::default(),
            1.into(),
            p1,
        ))
        .create()
        .unwrap();

    world.add_entity(e2).unwrap();

    let comps = p.borrow().get_all_components_in_children::<Marker>();
    assert_eq!(comps.len(), 3);

    let comps = p.borrow().get_all_components_in_children::<Transform>();
    assert_eq!(comps.len(), 3);
}
