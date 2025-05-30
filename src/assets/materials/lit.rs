#![allow(clippy::too_many_lines)]
use std::num::NonZeroU64;

use bytemuck::bytes_of;
use wgpu::util::DeviceExt;
use wgpu::{BufferUsages, TextureUsages};
use wgpu_shader_checker::include_wgsl;

use crate::UUID;
use crate::assets::{Material, Texture};
use crate::internal::STAGING_BELT;
use crate::math::Vec3;
use crate::structures::Color;
use crate::{DEVICE, FORMAT, grimoire};

use crate::{assets::BindgroupState, assets::material::MaterialTrait};

use super::helpers::{preprocess_shader, storage_buffer_available, vertex_binding};

///Basic material that renders an object, with an optional texture and color. This  material is lit.
///
///If neither the color nor the texture is  set, the material will be white
pub struct Lit {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout_f: Option<wgpu::BindGroupLayout>,
    uniform: Option<wgpu::Buffer>,

    color: Color,
    specular_color: Color,
    shininess: f32,
    bindgroup_sate: BindgroupState,
    changed: bool,
    texture_id: Option<UUID>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct MaterialData {
    color: Color,
    specular_color: Color,
    shininess: f32,
    pading: Vec3,
}

impl Lit {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    ///Creates a new material with an optional color and optional texture
    pub fn new(
        texture_id: Option<UUID>,
        color: Option<Color>,
        specular_color: Option<Color>,
        shininess: f32,
    ) -> Material {
        Self {
            bind_group: None,
            bind_group_layout_f: None,
            bindgroup_sate: BindgroupState::Uninitialized,
            changed: false,
            color: color.unwrap_or(Color::white()),
            pipeline: None,
            shininess,
            specular_color: specular_color.unwrap_or(Color::white()),
            texture_id,
            uniform: None,
        }
        .into()
    }

    ///Returns the shininess of the material
    #[must_use]
    pub const fn get_shininess(&self) -> f32 {
        self.shininess
    }

    ///Returns the color of the material
    #[must_use]
    pub const fn get_color(&self) -> Color {
        self.color
    }

    ///Sets the shininess of the material
    pub const fn set_shininess(&mut self, shininess: f32) {
        self.shininess = shininess;
        self.changed = true;
    }

    ///Sets the color of the material
    pub const fn set_color(&mut self, color: Color) {
        self.color = color;
        self.changed = true;
    }

    ///Returns the specular color of the material
    #[must_use]
    pub const fn get_specular_color(&self) -> Color {
        self.specular_color
    }

    ///Sets the specular color of the material
    pub const fn set_specular_color(&mut self, specular_color: Color) {
        self.specular_color = specular_color;
        self.changed = true;
    }
}

impl MaterialTrait for Lit {
    fn update_bindgroups(&mut self, encoder: &mut wgpu::CommandEncoder) {
        //Do nothing if no changes to the data
        if !self.changed {
            return;
        }
        self.changed = false;

        let mut staging_belt = STAGING_BELT.get().unwrap().write().unwrap();
        let device = DEVICE.get().unwrap();

        let data = MaterialData {
            color: self.color,
            pading: Vec3::default(),
            shininess: self.shininess,
            specular_color: self.specular_color,
        };

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
        render_pass.set_bind_group(1, self.bind_group.as_ref().unwrap(), &[]);
    }

    fn intialize(&mut self) {
        let storage_buf_available = storage_buffer_available();
        let device = DEVICE.get().unwrap();

        let v_shader = device.create_shader_module(include_wgsl!("src/shaders/vertex.wgsl"));

        let f_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl({
                preprocess_shader(
                    include_str!("../../shaders/lit.wgsl"),
                    u32::from(!storage_buf_available),
                )
                .unwrap()
                .into()
            }),
        });

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

        let directional_light_bind_group_layout = device
            .create_bind_group_layout(&grimoire::DIRECTIONAL_LIGHT_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let point_light_bind_group_layout = device.create_bind_group_layout(
            &grimoire::point_light_bind_group_layout_descriptor(storage_buf_available),
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &cam_bind_group_layout,
                &bind_group_layout_f,
                &directional_light_bind_group_layout,
                &point_light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        self.bind_group_layout_f = Some(bind_group_layout_f);

        let data = MaterialData {
            color: self.color,
            pading: Vec3::default(),
            shininess: self.shininess,
            specular_color: self.specular_color,
        };

        self.uniform = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Material data"),
                contents: bytemuck::bytes_of(&data),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
        );

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

        self.pipeline = Some(pipeline);
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

        self.bind_group = Some(bind_group_f);
        self.bindgroup_sate = BindgroupState::Initialized;
    }

    fn bindgroup_sate(&self) -> crate::assets::BindgroupState {
        self.bindgroup_sate
    }

    fn is_lit(&self) -> bool {
        true
    }
}
