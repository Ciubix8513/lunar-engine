use super::{mesh::Mesh, transform::Transform};
use crate::ecs::*;

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
