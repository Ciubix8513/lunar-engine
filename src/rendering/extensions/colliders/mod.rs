use std::num::NonZero;

use bytemuck::cast_slice;
use wgpu::{BufferUsages, ColorWrites, ShaderStages, util::DeviceExt};
use wgpu_shader_checker::include_wgsl;

use crate::{
    components::{camera::MainCamera, physics::colliders},
    grimoire::CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR,
    import,
    internal::{DEVICE, FORMAT, STAGING_BELT},
    rendering::extensions::RenderingExtension,
    structures::Color,
};

///A render  extension used for rendering colliders
pub struct Collider {
    ///Priority of the extension
    pub priority: u32,
    collider_color: Color,
    changed: bool,
    initialzed: bool,
    sphere_mesh: Option<(wgpu::Buffer, wgpu::Buffer)>,
    sphere_ind_count: u32,
    cube_mesh: Option<(wgpu::Buffer, wgpu::Buffer)>,
    color_buf: Option<wgpu::Buffer>,
    pipeline: Option<wgpu::RenderPipeline>,
    color_bg: Option<wgpu::BindGroup>,
    matrix_bufers: Vec<wgpu::Buffer>,
    matrix_buf_lens: Vec<u64>,
    supported: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            priority: Default::default(),
            collider_color: Color::green(),
            changed: Default::default(),
            initialzed: Default::default(),
            sphere_mesh: Default::default(),
            sphere_ind_count: Default::default(),
            cube_mesh: Default::default(),
            color_buf: Default::default(),
            pipeline: Default::default(),
            color_bg: Default::default(),
            matrix_bufers: Default::default(),
            matrix_buf_lens: Default::default(),
            supported: Default::default(),
        }
    }
}

impl Collider {
    ///Creates a new instance of this extension with the given priority and color
    pub fn new_with_color(priority: u32, collider_color: Color) -> Self {
        Self {
            priority,
            collider_color,
            changed: false,
            initialzed: false,
            supported: true,
            sphere_ind_count: 0,
            matrix_buf_lens: Vec::new(),
            color_buf: None,
            sphere_mesh: None,
            cube_mesh: None,
            pipeline: None,
            color_bg: None,
            matrix_bufers: Vec::new(),
        }
    }
}

impl RenderingExtension for Collider {
    fn is_initialized(&self) -> bool {
        self.initialzed
    }

    fn initialize(&mut self) {
        self.initialzed = true;

        //Check features
        let device = DEVICE.get().unwrap();

        if !(device.features() & wgpu::Features::POLYGON_MODE_LINE)
            == wgpu::Features::POLYGON_MODE_LINE
        {
            self.supported = false;
            log::error!("Displaying colliders not supported");
            return;
        }

        let binding = import::obj::parse(include_str!("./Sphere_wireframe.obj")).unwrap();
        let sphere_mesh = binding.first().unwrap();

        self.sphere_ind_count = sphere_mesh.indices.len() as u32;

        let s_v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere vert buf"),
            contents: bytemuck::cast_slice(sphere_mesh.vertices.as_slice()),
            usage: BufferUsages::VERTEX,
        });

        let s_i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere vert buf"),
            contents: bytemuck::cast_slice(sphere_mesh.indices.as_slice()),
            usage: BufferUsages::INDEX,
        });

        let color_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Collider color"),
            contents: bytemuck::bytes_of(&self.collider_color),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        self.sphere_mesh = Some((s_v_buf, s_i_buf));

        let cam_bg_layout = device.create_bind_group_layout(&CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);
        let color_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZero::new(16).unwrap()),
                },
                count: None,
            }],
        });

        let color_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &color_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buf.as_entire_binding(),
            }],
        });

        self.color_bg = Some(color_bg);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Collider renderer pipeline layout"),
            bind_group_layouts: &[&cam_bg_layout, &color_bg_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Collider renderer pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &device.create_shader_module(include_wgsl!("../../../shaders/vertex.wgsl")),
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
                buffers: &crate::assets::materials::helpers::vertex_binding(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Line,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::R32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &device.create_shader_module(include_wgsl!("./collider.wgsl")),
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: *FORMAT.get().unwrap(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        self.pipeline = Some(pipeline);
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        world: &crate::ecs::World,
        _: &mut crate::asset_managment::AssetStore,
        attachments: &super::AttachmentData,
    ) {
        //Check if we can even render
        if !self.supported {
            return;
        }

        //First get all of the colliders
        let spheres = world
            .get_all_components::<colliders::Sphere>()
            .unwrap_or_default();
        // let cubes = world.get_all_components::<colliders::Sphere>().unwrap_or_default();
        // let capsules = world.get_all_components::<colliders::Sphere>().unwrap_or_default()u;
        //No op if no colliders
        if spheres.is_empty() {
            //&& cubes.is_empty() && capsules.is_empty() {
            return;
        }

        let mut belt = STAGING_BELT.get().unwrap().write().unwrap();
        let device = DEVICE.get().unwrap();

        if self.changed {
            self.changed = false;

            belt.write_buffer(
                encoder,
                self.color_buf.as_ref().unwrap(),
                0,
                NonZero::new(16).unwrap(),
                device,
            )
            .copy_from_slice(bytemuck::bytes_of(&self.collider_color));
        }

        let binding = world.get_all_components::<MainCamera>().unwrap();
        let cam = binding.first().unwrap().borrow();

        if self.matrix_buf_lens.get(0).copied().unwrap_or(0) < spheres.len() as u64 {
            // let b = self.matrix_bufers.get_mut(0);
            let buf = device.create_buffer(&wgpu::wgt::BufferDescriptor {
                label: None,
                size: 64 * spheres.len() as u64,
                usage: BufferUsages::MAP_WRITE | BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            if self.matrix_buf_lens.is_empty() {
                self.matrix_bufers.push(buf);
                self.matrix_buf_lens.push(spheres.len() as u64);
            } else {
                *self.matrix_bufers.get_mut(0).unwrap() = buf;
                *self.matrix_buf_lens.get_mut(0).unwrap() = spheres.len() as u64;
            }
        }

        let transforms = spheres
            .iter()
            .map(|i| {
                i.borrow()
                    .transform
                    .get()
                    .unwrap()
                    .borrow()
                    .matrix_transposed()
            })
            .collect::<Vec<_>>();

        belt.write_buffer(
            encoder,
            self.matrix_bufers.get(0).unwrap(),
            0,
            NonZero::new(64 * self.matrix_buf_lens.get(0).unwrap()).unwrap(),
            device,
        )
        .copy_from_slice(cast_slice(transforms.as_slice()));

        let mut rp = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Collider pass"),
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
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            })
            .forget_lifetime();

        rp.set_pipeline(self.pipeline.as_ref().unwrap());
        rp.set_bind_group(1, self.color_bg.as_ref().unwrap(), &[]);

        cam.set_bindgroup(&mut rp);

        rp.set_vertex_buffer(0, self.sphere_mesh.as_ref().unwrap().0.slice(..));
        rp.set_index_buffer(
            self.sphere_mesh.as_ref().unwrap().1.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        rp.set_vertex_buffer(0, self.matrix_bufers.get(0).unwrap().slice(..));

        rp.draw_indexed(
            0..self.sphere_ind_count,
            0,
            0..*self.matrix_buf_lens.get(0).unwrap() as u32,
        );

        drop(rp);
    }

    fn get_priority(&self) -> u32 {
        self.priority
    }
}
