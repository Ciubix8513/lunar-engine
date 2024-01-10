use std::sync::OnceLock;

use ecs::world::World;

pub mod abstractions;
pub mod ecs;
pub mod grimoire;
pub mod helpers;
pub mod import;
pub mod math;
pub mod structrures;

pub static DEVICE: OnceLock<wgpu::Device> = OnceLock::new();
pub static QUEUE: OnceLock<wgpu::Queue> = OnceLock::new();
pub static FORMAT: OnceLock<wgpu::TextureFormat> = OnceLock::new();
