use std::path::Path;

use futures::executor::block_on;

use crate::asset_managment::Asset;

async fn gen_gpu_async(instance: &wgpu::Instance) -> (wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .expect("Unable to get an adapter");

    adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .expect("Can not get device and queue")
}

fn generate_gpu() {
    let instance = wgpu::Instance::default();
    let (device, queue) = block_on(gen_gpu_async(&instance));
    crate::QUEUE.set(queue);
    crate::DEVICE.set(device);
}

#[test]
fn test_texture_load() {
    generate_gpu();
    let mut texture = super::Texture::new_bmp(Path::new("assets/test-data/blahaj1.bmp"));
    //You're not supposed to set ids manually, but it's fine :3
    texture.set_id(1).unwrap();

    //All assets MUST be tested this way
    texture.initialize().unwrap();
    texture.dispose();
    texture.initialize().unwrap();
}

#[test]
fn test_mesh_load() {
    generate_gpu();
    let mut mesh = super::Mesh::new_from_obj(Path::new("assets/test-data/cube.obj")).unwrap();
    mesh.set_id(1).unwrap();

    mesh.initialize().unwrap();
    mesh.dispose();
    mesh.initialize().unwrap();
}
