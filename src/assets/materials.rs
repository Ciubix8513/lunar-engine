#![allow(clippy::too_many_lines)]
use std::sync::Arc;

use crate::{asset_managment::UUID, grimoire, DEVICE, FORMAT};

use super::{material::MaterialTrait, BindgroupState, Texture};

///Basic material that renders an object with a given texture, without lighting
pub struct TextureUnlit {
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
    texture_id: UUID,
    bindgroup_sate: BindgroupState,
}

impl TextureUnlit {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    ///Creates a new material with a give texture id
    pub fn new(texture_id: UUID) -> Box<dyn MaterialTrait + 'static + Sync + Send> {
        Box::new(Self {
            pipeline: None,
            bind_group: None,
            bind_group_layout_f: None,
            texture_id,
            bindgroup_sate: BindgroupState::Uninitialized,
        }) as Box<dyn MaterialTrait + 'static + Send + Sync>
    }
}

impl MaterialTrait for TextureUnlit {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        //SHOULD BE FINE
        //TODO: FIND A BETTER SOLUTION
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

        let v_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/vertex.wgsl"));
        let f_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/texture_unlit.wgsl"));

        let bind_group_layout_f =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Fragment binding"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
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

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &v_shader,
                entry_point: "main",
                buffers: &[
                    //Vertex data
                    wgpu::VertexBufferLayout {
                        array_stride: 36,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 16,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 24,
                                shader_location: 2,
                            },
                        ],
                    },
                    //Transform data
                    wgpu::VertexBufferLayout {
                        array_stride: 64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 3,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 4,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 32,
                                shader_location: 5,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 48,
                                shader_location: 6,
                            },
                        ],
                    },
                ],
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
    }

    fn set_bindgroups(&mut self, asset_store: &crate::asset_managment::AssetStore) {
        let device = DEVICE.get().unwrap();

        let texture = asset_store.get_by_id::<Texture>(self.texture_id).unwrap();
        let texture = texture.borrow();

        let bind_group_f = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fragment bind group"),
            layout: self.bind_group_layout_f.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.texture.as_ref().unwrap().create_view(
                            &wgpu::TextureViewDescriptor {
                                label: None,
                                format: Some(wgpu::TextureFormat::Rgba8Unorm),
                                dimension: None,
                                aspect: wgpu::TextureAspect::All,
                                base_mip_level: 0,
                                mip_level_count: Some(1),
                                base_array_layer: 0,
                                array_layer_count: None,
                            },
                        ),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture.sampler.as_ref().unwrap()),
                },
            ],
        });
        drop(texture);

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

    fn bindgroup_sate(&self) -> super::BindgroupState {
        self.bindgroup_sate
    }
}
