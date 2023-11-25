use std::sync::OnceLock;

pub mod material;
pub mod model;
pub mod transfomation;

pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
pub static QUEUE: OnceLock<wgpu::Queue> = OnceLock::new();
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();
