#![allow(clippy::too_many_lines)]
use std::num::NonZeroU64;

use bytemuck::bytes_of;
use wgpu::util::DeviceExt;
use wgpu::{BufferUsages, TextureUsages};
use wgpu_shader_checker::include_wgsl;

use crate::UUID;
use crate::assets::{Material, Texture};
use crate::internal::STAGING_BELT;
use crate::structures::Color;
use crate::{DEVICE, FORMAT, grimoire};

use crate::{assets::BindgroupState, assets::material::MaterialTrait};

use super::helpers::vertex_binding;

///Basic material that renders an object, with an optional texture and color. This  material is NOT lit.
///
///If neither the color nor the texture is  set, the material will be white
pub struct Unlit {
    #[cfg(target_arch = "wasm32")]
    pipeline: Option<crate::wrappers::WgpuWrapper<wgpu::RenderPipeline>>,
    #[cfg(not(target_arch = "wasm32"))]
    pipeline: Option<wgpu::RenderPipeline>,
    #[cfg(target_arch = "wasm32")]
    bind_group: Option<crate::wrappers::WgpuWrapper<wgpu::BindGroup>>,
    #[cfg(not(target_arch = "wasm32"))]
    bind_group: Option<wgpu::BindGroup>,
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
    changed: bool,
    texture_id: Option<UUID>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct MaterialData {
    color: Color,
}

impl Unlit {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    ///Creates a new material with an optional color and optional texture
    pub fn new(texture_id: Option<UUID>, color: Option<Color>) -> Material {
        Self {
            bind_group: None,
            bind_group_layout_f: None,
            bindgroup_sate: BindgroupState::Uninitialized,
            changed: false,
            color: color.unwrap_or(Color::white()),
            pipeline: None,
            texture_id,
            uniform: None,
        }
        .into()
    }

    ///Returns the color of the material
    #[must_use]
    pub const fn get_color(&self) -> Color {
        self.color
    }

    ///Sets the color of the material
    pub const fn set_color(&mut self, color: Color) {
        self.color = color;
        self.changed = true;
    }
}

impl MaterialTrait for Unlit {
    fn update_bindgroups(&mut self, encoder: &mut wgpu::CommandEncoder) {
        //Do nothing if no changes to the data
        if !self.changed {
            return;
        }
        self.changed = false;

        let mut staging_belt = STAGING_BELT.get().unwrap().write().unwrap();
        let device = DEVICE.get().unwrap();

        let data = MaterialData { color: self.color };

        staging_belt
            .write_buffer(
                encoder,
                self.uniform.as_ref().unwrap(),
                0,
                NonZeroU64::new(size_of::<MaterialData>() as u64).unwrap(),
                device,
            )
            .copy_from_slice(bytes_of(&data));
    }

    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(self.pipeline.as_ref().unwrap());
        render_pass.set_bind_group(1, self.bind_group.as_ref(), &[]);
    }

    fn intialize(&mut self) {
        let device = DEVICE.get().unwrap();

        let v_shader = device.create_shader_module(include_wgsl!("src/shaders/vertex.wgsl"));
        let f_shader = device.create_shader_module(include_wgsl!("src/shaders/unlit.wgsl"));

        let bind_group_layout_f =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Fragment binding"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(size_of::<MaterialData>() as u64),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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

        let data = MaterialData { color: self.color };

        #[cfg(target_arch = "wasm32")]
        {
            self.uniform = Some(crate::wrappers::WgpuWrapper::new(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::bytes_of(&data),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                }),
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.uniform = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Material data"),
                    contents: bytemuck::bytes_of(&data),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                }),
            );
        }

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &v_shader,
                entry_point: Some("main"),
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
                entry_point: Some("main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: *FORMAT.get().unwrap(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        #[cfg(target_arch = "wasm32")]
        {
            self.pipeline = Some(crate::wrappers::WgpuWrapper::new(pipeline));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.pipeline = Some(pipeline);
        }
    }

    fn dispose(&mut self) {
        self.bind_group = None;
        self.pipeline = None;
        self.bindgroup_sate = BindgroupState::Uninitialized;
        self.uniform = None;
    }

    fn set_bindgroups(&mut self, asset_store: &mut crate::asset_managment::AssetStore) {
        let device = DEVICE.get().unwrap();

        if self.texture_id.is_none() {
            //Create empty texture
            self.texture_id = Some(grimoire::DEFAULT_TEXTURE_ASSET_ID);
        }

        let mut texture = asset_store.get_by_id::<Texture>(self.texture_id.unwrap());

        if self.texture_id.unwrap() == grimoire::DEFAULT_TEXTURE_ASSET_ID && texture.is_err() {
            drop(texture);
            //Register a new default texture
            //Ignore if it fails
            _ = asset_store.try_register_with_id(
                crate::assets::heleprs::generate_empty_texture(),
                grimoire::DEFAULT_TEXTURE_ASSET_ID,
            );
            texture = asset_store.get_by_id::<Texture>(self.texture_id.unwrap());
        }

        let binding = texture.unwrap();
        let texture = binding.borrow();

        let bind_group_f = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fragment bind group"),
            layout: self.bind_group_layout_f.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
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
                                usage: Some(TextureUsages::TEXTURE_BINDING),
                            },
                        ),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(texture.sampler.as_ref().unwrap()),
                },
            ],
        });

        drop(texture);

        #[cfg(target_arch = "wasm32")]
        {
            self.bind_group = Some(crate::wrappers::WgpuWrapper::new(bind_group_f));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.bind_group = Some(bind_group_f);
        }
        self.bindgroup_sate = BindgroupState::Initialized;
    }

    fn bindgroup_sate(&self) -> crate::assets::BindgroupState {
        self.bindgroup_sate
    }

    fn is_lit(&self) -> bool {
        false
    }
}
