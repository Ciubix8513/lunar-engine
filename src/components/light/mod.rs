use std::{cell::OnceCell, num::NonZero};

use lunar_engine_derive::{as_any, unique};
use wgpu::BufferUsages;

use crate::{ecs::Component, grimoire, math::Vec3, structures::Color, DEVICE};

///The directional light component, describes the behaviour of the main directional light of a
///scene, i.e. the sun
#[derive(Debug)]
pub struct DirectionalLight {
    ///Direction from which the light is shining
    pub direction: Vec3,
    ///Color of the light
    pub color: Color,
    ///Intensity of the light
    pub intensity: f32,
    buffer: OnceCell<wgpu::Buffer>,
    bind_group: OnceCell<wgpu::BindGroup>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Color::white(),
            intensity: 100.0,
            bind_group: OnceCell::new(),
            buffer: OnceCell::new(),
        }
    }
}

impl Component for DirectionalLight {
    #[as_any]
    #[unique]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn awawa(&mut self) {
        self.initialize_gpu();
    }
}

impl DirectionalLight {
    pub(crate) fn initialize_gpu(&mut self) {
        let device = DEVICE.get().unwrap();

        //3 floats for direction, 4 floats for color, 1 float for intensity
        let size = 8 * 4;

        let b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&grimoire::LIGHT_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light bg"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &b,
                    offset: 0,
                    size: NonZero::new(size),
                }),
            }],
        });

        self.buffer.set(b).unwrap();
        self.bind_group.set(bg).unwrap();
    }
}
