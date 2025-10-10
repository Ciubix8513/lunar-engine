use crate::{math::Vec3, rendering::extensions::RenderingExtension, structures::Color};

#[cfg(feature = "physics")]
pub mod collider;

#[derive(Debug)]
///An extension for rendering debug information
pub struct Debug {
    priority: u32,
    lines: Vec<(Vec3, Vec3, Color)>,
    vertex_buf: Option<wgpu::Buffer>,
    buf_size: u32,
    initialized: bool,
}

impl Debug {
    ///Creates a new extension with a given priority
    pub fn new(priority: u32) -> Self {
        Self {
            priority,
            lines: Vec::new(),
            vertex_buf: None,
            buf_size: 0,
            initialized: false,
        }
    }

    ///Draws a line from point A to point B in world space, with the given color
    pub fn draw_line(&mut self, point_a: Vec3, point_b: Vec3, color: Color) {
        self.lines.push((point_a, point_b, color));
    }

    fn setup(&mut self) {}

    fn create_buf(&mut self) {}
}

#[cfg(feature = "physics")]
impl rapier3d::pipeline::DebugRenderBackend for Debug {
    fn draw_line(
        &mut self,
        _: rapier3d::prelude::DebugRenderObject,
        a: rapier3d::prelude::Point<f32>,
        b: rapier3d::prelude::Point<f32>,
        color: rapier3d::prelude::DebugColor,
    ) {
        use crate::structures::Color;

        self.draw_line(
            Into::<[f32; 3]>::into(a).into(),
            Into::<[f32; 3]>::into(b).into(),
            Color::from_hsl(color[0], color[1], color[2]),
        );
    }
}

impl RenderingExtension for Debug {
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        world: &crate::ecs::World,
        _: &mut crate::asset_managment::AssetStore,
        attachments: &super::AttachmentData,
    ) {
        //There are no lines, don't do anything
        if self.lines.is_empty() {
            return;
        }

        //Set up all the pipelines and stuff if it has not been done yet
        if !self.initialized {
            self.setup();
            self.initialized = true;
        }

        if self.lines.len() as u32 > self.buf_size {
            //Create a new buffer for the lines
            self.create_buf();
            self.buf_size = self.lines.len() as u32;
        }

        //It's probably okay to keep the capacity since it is likely that the next frame will have
        //the same number of lines
        self.lines.clear();
    }

    fn get_priority(&self) -> u32 {
        self.priority
    }
}
