//!Internal globals.
//!
//!May also be used for custom assets, rendering, etc.
use std::sync::{OnceLock, RwLock};

#[cfg(target_arch = "wasm32")]
use crate::wrappers::WgpuWrapper;
use winit::dpi::PhysicalSize;

#[cfg(target_arch = "wasm32")]
///Connection to a graphics device (usually a gpu) used for creating objects
pub static DEVICE: OnceLock<WgpuWrapper<wgpu::Device>> = OnceLock::new();
#[cfg(target_arch = "wasm32")]
///Command queue of the device, used for submitting render and data transfer commands
pub static QUEUE: OnceLock<WgpuWrapper<wgpu::Queue>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
///Connection to a graphics device (usually a gpu) used for creating objects
pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
///Command queue of the device, used for submitting render and data transfer commands
pub static QUEUE: std::sync::OnceLock<wgpu::Queue> = OnceLock::new();
///Format of the frame buffer
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
///Staging belt used for easier RW of data
pub static STAGING_BELT: OnceLock<RwLock<WgpuWrapper<wgpu::util::StagingBelt>>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
///Staging belt used for easier RW of data
pub static STAGING_BELT: OnceLock<RwLock<wgpu::util::StagingBelt>> = OnceLock::new();
///Resolution of the frame buffer
pub static RESOLUTION: RwLock<PhysicalSize<u32>> = RwLock::new(PhysicalSize {
    width: 0,
    height: 0,
});
