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

use crate::{
    asset_managment::AssetStore, ecs::World, DEPTH, DEVICE, FORMAT, QUEUE, STAGING_BELT, SURFACE,
};

use self::extensions::{AttachmentData, RenderingExtension};

pub mod extensions;

///Renders all the entities in the world
pub fn render(world: &World, assets: &AssetStore, extensions: &[&dyn RenderingExtension]) {
    let device = DEVICE.get().unwrap();
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let color = SURFACE.get().map(|i| i.read().ok()).flatten().unwrap();

    let color = color.get_current_texture().unwrap();

    let color_view = color.texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Color attachment view"),
        format: Some(*FORMAT.get().unwrap()),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    let depth_setencil_veiw =
        DEPTH
            .get()
            .unwrap()
            .read()
            .unwrap()
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

    let attachments = AttachmentData {
        color: color_view,
        depth_stencil: depth_setencil_veiw,
    };

    for e in extensions {
        e.render(&mut encoder, world, assets, &attachments);
    }

    let cmd_buffer = encoder.finish();

    let queue = QUEUE.get().unwrap();

    let mut belt = STAGING_BELT.get().unwrap().write().unwrap();
    belt.finish();

    queue.submit(Some(cmd_buffer));

    belt.recall();
}
