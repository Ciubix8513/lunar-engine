//! The rendering system, used with the ecs
//!
//! # Rendering system
//!
//! A scene consists of a world and an asset store.
//! The asset store SHOULD consists only of the assets needed for the scene, however they MAY
//! contain other assets.
//!
//! The render function accepts a world and an asset store.
//! The rendering function gets the asset ids and queries them from the store.

use log::trace;

use crate::{
    asset_managment::AssetStore, ecs::World, DEPTH, DEVICE, FORMAT, QUEUE, STAGING_BELT, SURFACE,
};

use self::extensions::{AttachmentData, RenderingExtension};

///System for making custom renderers for objects, also contains implemented rendering extensions
pub mod extensions;

///Renders all the entities in the world
pub fn render(
    world: &World,
    assets: &mut AssetStore,
    extensions: &mut [&mut dyn RenderingExtension],
) {
    #[cfg(feature = "tracy")]
    let _span = tracy_client::span!("render", 1000);

    trace!("Beginning of the render function");

    let device = DEVICE.get().unwrap();
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let color = SURFACE
        .get()
        .and_then(|i| i.read().ok())
        .unwrap()
        .get_current_texture()
        .unwrap();
    trace!("Accquiered surface");

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

    trace!("Created attachment data");

    #[cfg(feature = "tracy")]
    let _span = tracy_client::span!("Rendering extensions");

    for e in extensions {
        trace!("Calling render on an extension");
        e.render(&mut encoder, world, assets, &attachments);
    }

    let cmd_buffer = encoder.finish();

    let queue = QUEUE.get().unwrap();

    let mut belt = STAGING_BELT.get().unwrap().write().unwrap();
    belt.finish();

    queue.submit(Some(cmd_buffer));

    belt.recall();
    drop(belt);

    color.present();

    #[cfg(feature = "tracy")]
    tracy_client::frame_mark();
}
