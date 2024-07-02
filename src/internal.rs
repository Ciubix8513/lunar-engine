use std::sync::{OnceLock, RwLock};

#[cfg(target_arch = "wasm32")]
use crate::wrappers::WgpuWrapper;
use winit::dpi::PhysicalSize;

#[cfg(target_arch = "wasm32")]
pub static DEVICE: OnceLock<WgpuWrapper<wgpu::Device>> = OnceLock::new();
#[cfg(target_arch = "wasm32")]
pub static QUEUE: OnceLock<WgpuWrapper<wgpu::Queue>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
pub static QUEUE: std::sync::OnceLock<wgpu::Queue> = OnceLock::new();
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
pub static STAGING_BELT: OnceLock<RwLock<WgpuWrapper<wgpu::util::StagingBelt>>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
pub static STAGING_BELT: OnceLock<RwLock<wgpu::util::StagingBelt>> = OnceLock::new();
pub static RESOLUTION: RwLock<PhysicalSize<u32>> = RwLock::new(PhysicalSize {
    width: 0,
    height: 0,
});
