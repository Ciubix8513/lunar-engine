//! Contains the rendering system, used with the ecs
//!
//! # Rendering system
//!
//! A scene consists of a world and an asset store.
//!
//! The asset store consists ONLY of the assets needed for the scene, nothing else.
//!
//! The render function accepts a world and an asset store
//!
//! The rendering function gets the asset ids and queries them from the store

use std::borrow::Borrow;

use log::info;

use crate::{
    asset_managment::AssetStore,
    assets::{BindgroupState, Material, Mesh},
    components,
    ecs::World,
    DEVICE, FORMAT, QUEUE,
};
pub struct AttachmentData {
    pub color: wgpu::Texture,
    pub depth_stencil: wgpu::Texture,
}

///Renders all the entities in the world
pub fn render(world: &World, asset_store: &AssetStore, attachments: AttachmentData) {
    let binding = world
        .get_all_components::<components::camera::MainCamera>()
        .expect("Could not find the main camera");
    let camera = binding.first().unwrap();

    //This is cached, so should be reasonably fast
    let meshes = world.get_all_components::<crate::components::mesh::Mesh>();
    // let main_camera = world.get_all_components::crate
    //
    //No rendering needs to be done
    //NO there IS work to be done here, like the skybox and shit
    if meshes.is_none() {
        info!("Rendered 0 meshes");
        return;
    }
    let meshes = meshes.unwrap();
    let device = DEVICE.get().unwrap();
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let mut camera = camera.borrow_mut();

    camera.update_gpu(&mut encoder);
    let mut materials = Vec::new();

    //Update the gpu data for every Mesh
    for m in &meshes {
        let m = m.borrow();
        m.update_gpu(&mut encoder);
        materials.push(m.get_material_id().unwrap());
    }

    //Initialize bindgroups for all needed materials
    for m in materials {
        let m = asset_store.get_by_id::<Material>(m).unwrap();
        let mut m = m.borrow_mut();

        if let BindgroupState::Initialized = m.get_bindgroup_state() {
            continue;
        }

        m.initialize_bindgroups(asset_store);
    }

    let color_view = attachments.color.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Color attachment view"),
        format: Some(*FORMAT.get().unwrap()),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    let depth_setencil_veiw = attachments
        .depth_stencil
        .create_view(&wgpu::TextureViewDescriptor {
            label: Some("Depth stencil attachment"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("First pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &color_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_setencil_veiw,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    //Set the camera
    camera.set_bindgroup(&mut render_pass);

    //Iterate through the meshes and render them
    for m in &meshes {
        let m = m.borrow();

        m.set_bindgroup(&mut render_pass);

        let mat = asset_store
            .get_by_id::<Material>(m.get_material_id().unwrap())
            .unwrap();
        let mat = mat.borrow();

        mat.render(&mut render_pass);

        let mesh = asset_store
            .get_by_id::<Mesh>(m.get_mesh_id().unwrap())
            .unwrap();
        let mesh = mesh.borrow();

        mesh.render(&mut render_pass);
    }
    drop(render_pass);

    let cmd_buffer = encoder.finish();

    let queue = QUEUE.get().unwrap();

    queue.submit(Some(cmd_buffer));
    //Done?
}
