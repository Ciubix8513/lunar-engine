use crate::{
    abstractions::{DEVICE, FORMAT, QUEUE},
    grimoire,
};

use super::Material;

pub struct TextureUnlit {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<wgpu::BindGroup>,
}

impl TextureUnlit {
    pub fn new(texture: renderer_lib::structrures::image::Image) -> TextureUnlit {
        let device = DEVICE.get().unwrap();

        let v_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/vertex.wgsl"));
        let f_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/texture_unlit.wgsl"));

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

        let texture_ = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        let queue = QUEUE.get().unwrap();
        queue.write_texture(
            texture_.as_image_copy(),
            bytemuck::cast_slice(&texture.data[..]),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(texture.width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
        );
        drop(queue);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let bind_group_f = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fragment bind group"),
            layout: &bind_group_layout_f,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_.create_view(
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
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let trans_bind_group_layout =
            device.create_bind_group_layout(&grimoire::TRANSFORM_BIND_GROUP_LAYOUT_DESCRIPTOR);
        let cam_bind_group_layout =
            device.create_bind_group_layout(&grimoire::CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &trans_bind_group_layout,
                &cam_bind_group_layout,
                &bind_group_layout_f,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &v_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
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
                }],
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
                    format: FORMAT.get().unwrap().clone(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        TextureUnlit {
            pipeline,
            bind_groups: vec![bind_group_f],
        }
    }
}

impl<'a, 'b> Material<'a, 'b> for TextureUnlit {
    fn render(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        let render_pass = render_pass;
        render_pass.set_pipeline(&self.pipeline);
        for (index, b) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(index as u32 + 2, b, &[]);
        }
    }
}
