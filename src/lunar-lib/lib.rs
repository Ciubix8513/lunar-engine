use std::sync::OnceLock;

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

pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
pub static QUEUE: OnceLock<wgpu::Queue> = OnceLock::new();
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();
