use wgpu::{RenderPipeline, VertexAttribute};
use wgpu_shader_checker::include_wgsl;

use crate::{
    components::camera::MainCamera,
    internal::{DEVICE, FORMAT, QUEUE},
    math::{Quaternion, Vec3},
    rendering::extensions::RenderingExtension,
    structures::Color,
};

#[cfg(feature = "physics")]
pub mod collider;

#[derive(Debug, Default)]
///An extension for rendering debug information
pub struct Debug {
    priority: u32,
    lines: Vec<(Vec3, Vec3, Color)>,
    vertex_buf: Option<wgpu::Buffer>,
    pipeline: Option<RenderPipeline>,
    buf_size: u32,
    initialized: bool,
}

#[repr(C)]
#[derive(bytemuck::Zeroable, bytemuck::Pod, Clone, Copy)]
struct Vertex {
    pos: Vec3,
    color: Color,
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
            pipeline: None,
        }
    }

    ///Draws a line from point A to point B in world space, with the given color
    pub fn draw_line(&mut self, point_a: Vec3, point_b: Vec3, color: Color) {
        self.lines.push((point_a, point_b, color));
    }

    ///Draws a box at the given position with the given rotation and size
    pub fn draw_box(
        &mut self,
        position: Vec3,
        rotation: Quaternion,
        dimensions: Vec3,
        color: Color,
    ) {
        let mut points = [
            position + dimensions * Into::<Vec3>::into((1, 1, 1)),
            position + dimensions * Into::<Vec3>::into((1, 1, -1)),
            position + dimensions * Into::<Vec3>::into((-1, 1, 1)),
            position + dimensions * Into::<Vec3>::into((-1, 1, -1)),
            position + dimensions * Into::<Vec3>::into((1, -1, 1)),
            position + dimensions * Into::<Vec3>::into((1, -1, -1)),
            position + dimensions * Into::<Vec3>::into((-1, -1, 1)),
            position + dimensions * Into::<Vec3>::into((-1, -1, -1)),
        ];
        let mat = rotation.matrix();

        for i in &mut points {
            *i = mat.transform3(*i);
        }

        //ab ad, cb cd
        let indices = [
            0, 1, 0, 2, 3, 1, 3, 2, // TOP
            4, 5, 4, 6, 7, 5, 7, 6, //Bottom
            0, 4, 1, 5, 2, 6, 3, 7, //Sides
        ];

        for i in indices.chunks(2) {
            self.lines.push((points[i[0]], points[i[1]], color));
        }
    }

    fn setup(&mut self) {
        let device = DEVICE.get().unwrap();
        //
        //
        //

        let vb_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 3,
                    shader_location: 1,
                },
            ],
        };
        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug renderer bg layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug renderer pipeline layout"),
            bind_group_layouts: &[&bg_layout],
            push_constant_ranges: &[],
        });

        self.pipeline = Some(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Debug renderer pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &device.create_shader_module(include_wgsl!("./debug_vertex.wgsl")),
                    entry_point: None,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[vb_layout],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &device.create_shader_module(include_wgsl!("./debug_fragment.wgsl")),
                    entry_point: None,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: *FORMAT.get().unwrap(),
                        blend: None,
                        write_mask: wgpu::ColorWrites::COLOR,
                    })],
                }),
                multiview: None,
                cache: None,
            }),
        );
    }

    fn create_buf(&mut self) {
        let device = DEVICE.get().unwrap();
        self.vertex_buf = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug renderer VB"),
            //Each line is 2 vertices
            size: (self.lines.len() * 2 * size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }))
    }
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

        let cam = world
            .get_unique_component::<MainCamera>()
            .expect("Could not find the main camera");

        cam.borrow_mut().update_gpu(encoder);

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

        {
            //TODO: Optimize this copying nightmare
            let queue = QUEUE.get().unwrap();
            let data = self
                .lines
                .iter()
                .flat_map(|i| {
                    [
                        Vertex {
                            pos: i.0,
                            color: i.2,
                        },
                        Vertex {
                            pos: i.1,
                            color: i.2,
                        },
                    ]
                })
                .collect::<Vec<_>>();

            queue.write_buffer(
                self.vertex_buf.as_ref().unwrap(),
                0,
                &data
                    .iter()
                    .flat_map(bytemuck::bytes_of)
                    .copied()
                    .collect::<Vec<_>>(),
            );
        }

        {
            let cam = cam.borrow();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Debug Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &attachments.color,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &attachments.depth_stencil,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            cam.set_bindgroup(&mut pass);
            pass.set_pipeline(self.pipeline.as_ref().unwrap());
            pass.set_vertex_buffer(0, self.vertex_buf.as_ref().unwrap().slice(..));
            pass.draw(0..(self.lines.len() as u32 * 2), 0..1);
        }

        //It's probably okay to keep the capacity since it is likely that the next frame will have
        //the same number of lines
        self.lines.clear();
    }

    fn get_priority(&self) -> u32 {
        self.priority
    }
}
