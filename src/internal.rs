//!Internal globals.
//!
//!May also be used for custom assets, rendering, etc.
use std::sync::{OnceLock, RwLock};

use winit::dpi::PhysicalSize;

///Connection to a graphics device (usually a gpu) used for creating objects
pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
///Command queue of the device, used for submitting render and data transfer commands
pub static QUEUE: std::sync::OnceLock<wgpu::Queue> = OnceLock::new();
///Format of the frame buffer
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();

///Staging belt used for easier RW of data
pub static STAGING_BELT: OnceLock<RwLock<wgpu::util::StagingBelt>> = OnceLock::new();
///Resolution of the frame buffer
pub static RESOLUTION: RwLock<PhysicalSize<u32>> = RwLock::new(PhysicalSize {
    width: 0,
    height: 0,
});
