#![allow(clippy::too_many_lines)]
use std::num::NonZeroU64;
use std::sync::Arc;

use wgpu::util::DeviceExt;
use wgpu::BufferUsages;
use wgpu_shader_checker::include_wgsl;

use crate::assets::Material;
use crate::structures::Color;
use crate::{grimoire, DEVICE, FORMAT};

use crate::{assets::material::MaterialTrait, assets::BindgroupState};

use super::helpers::vertex_binding;

///Basic material that renders an object with a given texture, without lighting
pub struct ColorUnlit {
    #[cfg(target_arch = "wasm32")]
    pipeline: Option<Arc<crate::wrappers::WgpuWrapper<wgpu::RenderPipeline>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pipeline: Option<Arc<wgpu::RenderPipeline>>,
    #[cfg(target_arch = "wasm32")]
    bind_group: Option<Arc<crate::wrappers::WgpuWrapper<wgpu::BindGroup>>>,
    #[cfg(not(target_arch = "wasm32"))]
    bind_group: Option<Arc<wgpu::BindGroup>>,
    #[cfg(target_arch = "wasm32")]
    bind_group_layout_f: Option<crate::wrappers::WgpuWrapper<wgpu::BindGroupLayout>>,
    #[cfg(not(target_arch = "wasm32"))]
    bind_group_layout_f: Option<wgpu::BindGroupLayout>,
    #[cfg(target_arch = "wasm32")]
    uniform: Option<crate::wrappers::WgpuWrapper<wgpu::Buffer>>,
    #[cfg(not(target_arch = "wasm32"))]
    uniform: Option<wgpu::Buffer>,
    color: Color,
    bindgroup_sate: BindgroupState,
}

impl ColorUnlit {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    ///Creates a new material with a give texture id
    pub fn new(color: Color) -> Material {
        Self {
            color,
            pipeline: None,
            bind_group: None,
            bind_group_layout_f: None,
            bindgroup_sate: BindgroupState::Uninitialized,
            uniform: None,
        }
        .into()
    }
}

impl MaterialTrait for ColorUnlit {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        //SHOULD BE FINE
        //TODO: FIND A BETTER SOLUTION
        //This is a big FUCK OFF to the borrow checker
        let pipeline = unsafe {
            Arc::as_ptr(self.pipeline.as_ref().unwrap())
                .as_ref()
                .unwrap()
        };

        render_pass.set_pipeline(pipeline);
        let b = unsafe {
            Arc::as_ptr(&self.bind_group.clone().unwrap())
                .as_ref()
                .unwrap()
        };
        render_pass.set_bind_group(1, b, &[]);
    }

    fn intialize(&mut self) {
        let device = DEVICE.get().unwrap();

        let f_shader = device.create_shader_module(include_wgsl!("src/shaders/color_unlit.wgsl"));
        let v_shader = device.create_shader_module(include_wgsl!("src/shaders/vertex.wgsl"));

        let bind_group_layout_f =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Fragment binding"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(4 * 4).unwrap()),
                    },
                    count: None,
                }],
            });

        let cam_bind_group_layout =
            device.create_bind_group_layout(&grimoire::CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&cam_bind_group_layout, &bind_group_layout_f],
            push_constant_ranges: &[],
        });

        #[cfg(target_arch = "wasm32")]
        {
            self.bind_group_layout_f = Some(crate::wrappers::WgpuWrapper::new(bind_group_layout_f));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.bind_group_layout_f = Some(bind_group_layout_f);
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.uniform = Some(crate::wrappers::WgpuWrapper::new(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::bytes_of(&self.color),
                    usage: BufferUsages::UNIFORM,
                }),
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.uniform = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::bytes_of(&self.color),
                    usage: BufferUsages::UNIFORM,
                }),
            );
        }
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &v_shader,
                entry_point: "main",
                buffers: &vertex_binding(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
                module: &f_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: *FORMAT.get().unwrap(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
        });

        #[cfg(target_arch = "wasm32")]
        {
            self.pipeline = Some(Arc::new(crate::wrappers::WgpuWrapper::new(pipeline)));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.pipeline = Some(Arc::new(pipeline));
        }
    }

    fn dispose(&mut self) {
        self.bind_group = None;
        self.pipeline = None;
        self.bindgroup_sate = BindgroupState::Uninitialized;
        self.uniform = None;
    }

    fn set_bindgroups(&mut self, _asset_store: &mut crate::asset_managment::AssetStore) {
        let device = DEVICE.get().unwrap();

        let bind_group_f = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fragment bind group"),
            layout: self.bind_group_layout_f.as_ref().unwrap(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    self.uniform.as_ref().unwrap().as_entire_buffer_binding(),
                ),
            }],
        });
        #[cfg(target_arch = "wasm32")]
        {
            self.bind_group = Some(Arc::new(crate::wrappers::WgpuWrapper::new(bind_group_f)));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.bind_group = Some(Arc::new(bind_group_f));
        }
        self.bindgroup_sate = BindgroupState::Initialized;
    }

    fn bindgroup_sate(&self) -> crate::assets::BindgroupState {
        self.bindgroup_sate
    }
}
