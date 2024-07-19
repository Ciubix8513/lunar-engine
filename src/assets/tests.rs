use std::path::Path;

use crate::asset_managment::Asset;

#[test]
fn test_texture_load() {
    crate::test_utils::generate_gpu();
    let mut texture = super::Texture::new_png(Path::new("assets/test-data/blahaj.png"));
    //You're not supposed to set ids manually, but it's fine :3
    texture.set_id(1).unwrap();

    //All assets MUST be tested this way
    texture.initialize().unwrap();
    texture.dispose();
    texture.initialize().unwrap();
}

#[test]
fn test_mesh_load() {
    crate::test_utils::generate_gpu();
    let mut mesh = super::Mesh::new_from_obj(Path::new("assets/test-data/cube.obj")).unwrap();
    mesh.set_id(1).unwrap();

    mesh.initialize().unwrap();
    mesh.dispose();
    mesh.initialize().unwrap();
}
