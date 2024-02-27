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

use crate::{asset_managment::AssetStore, ecs::World, DEVICE, QUEUE, STAGING_BELT};

use self::extensions::{AttachmentData, RenderingExtension};

pub mod extensions;

///Renders all the entities in the world
pub fn render(
    world: &World,
    assets: &AssetStore,
    attachments: &AttachmentData,
    extensions: &[&dyn RenderingExtension],
) {
    let device = DEVICE.get().unwrap();
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    for e in extensions {
        e.render(&mut encoder, world, assets, attachments);
    }

    let cmd_buffer = encoder.finish();

    let queue = QUEUE.get().unwrap();

    let mut belt = STAGING_BELT.get().unwrap().write().unwrap();
    belt.finish();

    queue.submit(Some(cmd_buffer));

    belt.recall();
}
