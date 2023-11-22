use std::sync::OnceLock;

pub mod material;
pub mod model;
pub mod transfomation;

pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
