use std::sync::{OnceLock, RwLock};

use winit::dpi::PhysicalSize;

pub mod abstractions;
pub mod asset_managment;
pub mod assets;
pub mod components;
pub mod ecs;
pub mod grimoire;
pub mod helpers;
pub mod import;
pub mod math;
pub mod structrures;
pub mod system;
#[cfg(test)]
mod test_utils;

pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
pub static QUEUE: OnceLock<wgpu::Queue> = OnceLock::new();
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();
pub static STAGING_BELT: OnceLock<RwLock<wgpu::util::StagingBelt>> = OnceLock::new();
pub static RESOLUTION: RwLock<PhysicalSize<u32>> = RwLock::new(PhysicalSize {
    width: 0,
    height: 0,
});
