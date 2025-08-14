#![allow(clippy::too_many_lines)]

use core::f32;
use std::{cell::OnceCell, num::NonZeroU64};

use log::{debug, trace};
use vec_key_value_pair::set::VecSet;
use wgpu::{BufferUsages, util::DeviceExt};

use crate::{
    DEVICE, RESOLUTION, STAGING_BELT,
    asset_managment::AssetStore,
    assets::{BindgroupState, Material, Mesh, materials::helpers::storage_buffer_available},
    components::{
        self,
        light::{DirectionalLight, PointLight},
    },
    ecs::{ComponentReference, World},
    grimoire::{self, point_light_bind_group_layout_descriptor},
    math::{Mat4x4, Vec3, Vec4, Vec4Swizzles, Vector as _},
    structures::{Color, LightBuffer},
};

mod colliders;

pub use colliders::Collider;

///A color buffer and a depth stencil buffer
pub struct AttachmentData {
    ///Color buffer
    pub color: wgpu::TextureView,
    ///Depth stencil buffer
    pub depth_stencil: wgpu::TextureView,
}

///Trait that all rendering extensions must implement
///
///Allows for extending the renderer
pub trait RenderingExtension {
    ///Uses the extension to render scene
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        assets: &mut AssetStore,
        attachments: &AttachmentData,
    );

    ///Returns the priority of the extension, extensions with smaller priorities are rendered first.
    fn get_priority(&self) -> u32;

    ///Returns wether or not this extension is initialized
    fn is_initialized(&self) -> bool {
        true
    }

    ///Initializes the extension
    fn initialize(&mut self) {}
}

impl std::cmp::PartialEq for dyn RenderingExtension {
    fn eq(&self, other: &Self) -> bool {
        self.get_priority().eq(&other.get_priority())
    }
}

impl std::cmp::Eq for dyn RenderingExtension {}

impl std::cmp::PartialOrd for dyn RenderingExtension {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for dyn RenderingExtension {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_priority().cmp(&other.get_priority())
    }
}

#[derive(Debug)]
struct PointLights {
    buffer: wgpu::Buffer,
    count_buf: wgpu::Buffer,
    num_lights: usize,
    bindgroup: wgpu::BindGroup,
}

#[derive(Default)]
///Basic renderer that renders all [`crate::components::mesh::Mesh`] components
///
///# Usage
///```
///# use lunar_engine::rendering::extensions::Base;
///# use lunar_engine::rendering::render;
///# struct State {world: lunar_engine::ecs::World, assets: lunar_engine::asset_managment::AssetStore, extension: Base}
///
///fn init(state: &mut State) {
/// state.extension = Base::default();
///}
///
///fn update(state: &mut State) {
/// render(
///   &state.world,
///   &mut state.assets,
///   &mut [&mut state.extension]
///  );
///}
///```
pub struct Base {
    ///Priority of the extension
    pub priority: u32,
    ///Clear color used for rendering
    pub clear_color: Color,
    ///Whether or not to use frustum culling
    pub frustum_culling: bool,
    //Stores vector of (mesh_id, material_id) for caching
    identifier: Vec<(u128, u128)>,
    v_buffers: Vec<wgpu::Buffer>,
    mesh_materials: Vec<MeshMaterial>,
    num_instances: Vec<usize>,
    mesh_refs: Vec<Vec<ComponentReference<components::mesh::Mesh>>>,
    light_buffer: OnceCell<(wgpu::Buffer, wgpu::BindGroup)>,
    point_light_buffer: OnceCell<PointLights>,
    storage_buffer_available: OnceCell<bool>,
}

impl Base {
    #[must_use]
    ///Creates a new [`Base`]
    pub const fn new(order: u32, frustum_culling: bool) -> Self {
        Self {
            frustum_culling,
            priority: order,
            clear_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            identifier: Vec::new(),
            v_buffers: Vec::new(),
            mesh_materials: Vec::new(),
            num_instances: Vec::new(),
            mesh_refs: Vec::new(),
            light_buffer: OnceCell::new(),
            point_light_buffer: OnceCell::new(),
            storage_buffer_available: OnceCell::new(),
        }
    }

    ///Creates a new [`Base`] with a pre defined clear color
    ///
    ///The clear color is the color that is used as a background
    ///
    ///Everything rendered with this extension will have that color in the parts not occupied by a mesh.
    #[must_use]
    pub const fn new_with_color(order: u32, frustum_culling: bool, color: Color) -> Self {
        Self {
            frustum_culling,
            priority: order,
            clear_color: color,
            identifier: Vec::new(),
            v_buffers: Vec::new(),
            mesh_materials: Vec::new(),
            num_instances: Vec::new(),
            mesh_refs: Vec::new(),
            light_buffer: OnceCell::new(),
            point_light_buffer: OnceCell::new(),
            storage_buffer_available: OnceCell::new(),
        }
    }
}

#[derive(Clone, Copy)]
struct MeshMaterial {
    mesh_id: u128,
    material_id: u128,
}

impl PartialEq<(u128, u128)> for MeshMaterial {
    fn eq(&self, other: &(u128, u128)) -> bool {
        self.mesh_id == other.0 && self.material_id == other.1
    }
}

impl MeshMaterial {
    const fn new(mesh_id: u128, material_id: u128) -> Self {
        Self {
            mesh_id,
            material_id,
        }
    }
}

impl RenderingExtension for Base {
    #[allow(clippy::cognitive_complexity)]
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        assets: &mut AssetStore,
        attachments: &AttachmentData,
    ) {
        //Initialize needed stuff
        if self.storage_buffer_available.get().is_none() {
            let storage_buf_available = storage_buffer_available();
            self.storage_buffer_available
                .set(storage_buf_available)
                .unwrap();

            if storage_buf_available {
                log::info!("Storage buffer supported");
            } else {
                log::info!("Storage buffer not supported");
            }
        }

        #[cfg(feature = "tracy")]
        let _span = tracy_client::span!("Frustum culling render");
        trace!("Started frame");

        //Update camera first
        let binding = world
            .get_all_components::<components::camera::MainCamera>()
            .expect("Could not find the main camera");

        let camera = binding.first().unwrap().borrow();
        camera.update_gpu(encoder);
        trace!("Accquired camera");

        //This is cached, so should be reasonably fast
        let binding = world
            .get_all_components::<crate::components::mesh::Mesh>()
            .unwrap_or_default();

        #[cfg(feature = "tracy")]
        let _frustum_span = tracy_client::span!("Frustum checking");

        let meshes = if self.frustum_culling {
            let frustum = calculate_frustum(
                camera.inner.near,
                camera.inner.far,
                camera.inner.projection_type.fov().unwrap_or_default(),
            );
            let camera_transform = camera.camera_transform();
            //Precompute the transformation matrix, since it's the same for all the objects
            let matrix = calculate_frustum_matrix(frustum, camera_transform);
            binding
                .iter()
                .filter(|i| {
                    let m = i.borrow();
                    let binding = m.get_transform();
                    let t = binding.borrow();

                    m.get_visible()
                        && check_frustum(
                            frustum.z,
                            matrix,
                            t.position,
                            assets
                                .borrow_by_id::<Mesh>(m.get_mesh_id().unwrap())
                                .unwrap()
                                .get_extent(),
                            t.scale,
                        )
                        .0
                })
                .collect::<Vec<_>>()
        } else {
            binding
                .iter()
                .filter(|i| i.borrow().get_visible())
                .collect::<Vec<_>>()
        };
        #[cfg(feature = "tracy")]
        drop(_frustum_span);
        trace!("Got all the meshes");

        //List of materials used for rendering
        let mut materials = VecSet::new();
        //List of (mesh_ID, (transformation matrix, material_id))
        let mut matrices = Vec::new();

        //Collect all the matrices
        for m in &meshes {
            let m = m.borrow();
            materials.insert(m.get_material_id().unwrap());
            matrices.push((m.get_mesh_id().unwrap(), (m.get_material_id().unwrap())));
        }

        //What is even going on here?

        let mut matrices = matrices
            .iter()
            .zip(meshes)
            .map(|i| (i.0.0, (i.0.1, i.1)))
            .collect::<Vec<_>>();

        //determine if can re use cache
        let mut identical = true;

        #[cfg(feature = "tracy")]
        let _cache_check_span = tracy_client::span!("Cache reuse check");

        if matrices.len() == self.identifier.len() {
            for (index, data) in self.identifier.iter().enumerate() {
                if data.0 == matrices[index].0 && data.1 == matrices[index].1.0 {
                    continue;
                }
                identical = false;
                break;
            }
        } else {
            identical = false;
        }

        #[cfg(feature = "tracy")]
        drop(_cache_check_span);

        #[allow(clippy::if_not_else)]
        if !identical {
            #[cfg(feature = "tracy")]
            let _span = tracy_client::span!("Cache generation");

            debug!("Generating new cache data");
            self.identifier = matrices.iter().map(|i| (i.0, i.1.0)).collect::<Vec<_>>();

            //Sort meshes by mesh id for easier buffer creation
            //NO Sort by material id?
            matrices.sort_unstable_by(|a, b| a.0.cmp(&b.0));

            //This is so jank omg
            //Yea... i agree

            //Find points where mesh changes
            let mut split_points = Vec::new();
            let mut old = 0;
            for (index, m) in matrices.iter().enumerate() {
                if m.0 != old {
                    split_points.push(index);
                    old = m.0;
                }
            }

            //Guarantee that there's at least 1 window
            split_points.push(matrices.len());

            //assemble vertex buffers
            let mut v_buffers = Vec::new();

            let device = DEVICE.get().unwrap();

            let mut mesh_materials = Vec::new();
            let mut num_instances = Vec::new();

            let mut mesh_refs = Vec::new();

            for m in split_points.windows(2) {
                //beginning and end of the window
                let points = (*m.first().unwrap(), *m.last().unwrap());

                //Label for easier debugging
                let label = format!("Instances: {}..{}", m.first().unwrap(), m.last().unwrap());

                //(mesh_ID, (transformation matrix, material_id, mesh reference));
                let mut current_window = matrices[points.0..points.1].to_vec();

                //Split into vectors and sorted by material
                //Sort the window by materials
                current_window.sort_unstable_by(|s, o| s.1.0.cmp(&o.1.0));

                //find where materials change, similar to how meshes were sorted
                let mut material_split_points = Vec::new();
                let mut old = 0;
                for (i, m) in current_window.iter().enumerate() {
                    if m.1.0 != old {
                        material_split_points.push(i);
                        old = m.1.0;
                    }
                }
                //Again ensure there's at least one window
                material_split_points.push(current_window.len());

                let mut last = MeshMaterial {
                    mesh_id: 0,
                    material_id: 0,
                };

                //Need to iterate over it twice...
                //Get indicators for every block of what mesh and material they are
                for i in &material_split_points[..material_split_points.len() - 1] {
                    let curent = current_window[*i];
                    if last != (curent.0, curent.1.0) {
                        last = MeshMaterial::new(curent.0, curent.1.0);
                        mesh_materials.push(last);
                    }
                }

                //AGAIN!?!?
                //Create vertex buffers for matrices
                for m in material_split_points.windows(2) {
                    //Now this is stored per mesh per material
                    let points = (*m.first().unwrap(), *m.last().unwrap());

                    num_instances.push(points.1 - points.0);
                    let current_window = &current_window[points.0..points.1];

                    //Copy mesh references
                    mesh_refs.push(
                        current_window
                            .iter()
                            .map(|i| i.1.1.clone())
                            .collect::<Vec<_>>(),
                    );

                    let matrices = current_window
                        .iter()
                        .map(|i| i.1.1.borrow().get_matrix())
                        .collect::<Vec<_>>();

                    let matrices = matrices
                        .iter()
                        .flat_map(bytemuck::bytes_of)
                        .copied()
                        .collect::<Vec<u8>>();
                    v_buffers.push(
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&label),
                            contents: &matrices,
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        }),
                    );
                }
            }
            //Check if they're the same length
            debug_assert_eq!(
                v_buffers.len(),
                mesh_materials.len(),
                "You are a moron, they're not the same"
            );
            debug_assert_eq!(
                v_buffers.len(),
                mesh_refs.len(),
                "You are stupid, they're not the same"
            );
            debug_assert_eq!(
                num_instances.len(),
                mesh_materials.len(),
                "You are an idiot, they're not the same"
            );

            self.v_buffers = v_buffers;
            self.mesh_materials = mesh_materials;
            self.num_instances = num_instances;
            self.mesh_refs = mesh_refs;
        } else {
            #[cfg(feature = "tracy")]
            let _span = tracy_client::span!("Cache reuse");

            //Reusing data
            trace!("Cache exists, updating v buffers");
            let mut belt = STAGING_BELT.get().unwrap().write().unwrap();
            let device = DEVICE.get().unwrap();

            for (buffer, meshes) in self.v_buffers.iter().zip(self.mesh_refs.iter()) {
                //I do have to collect here
                let matrices = meshes
                    .iter()
                    .map(|m| m.borrow().get_matrix())
                    .collect::<Vec<_>>();

                let matrix_data = matrices
                    .iter()
                    .flat_map(bytemuck::bytes_of)
                    .copied()
                    .collect::<Vec<u8>>();

                belt.write_buffer(
                    encoder,
                    buffer,
                    0,
                    NonZeroU64::new(buffer.size()).unwrap(),
                    device,
                )
                .copy_from_slice(matrix_data.as_slice());
            }
        }

        trace!("Initializing the bindgroups");

        let mut is_lit = false;

        //Initialize bindgroups for all needed materials
        for m in materials {
            let binding = assets.get_by_id::<Material>(m).unwrap();
            let mut m = binding.borrow_mut();

            is_lit = is_lit || m.is_lit();

            if matches!(m.get_bindgroup_state(), BindgroupState::Initialized) {
                continue;
            }
            m.initialize_bindgroups(assets);
            m.update_bindgroups(encoder);
        }

        //There are lit materials, need to take care of them
        if is_lit {
            //riiiight, need to get some light buffers
            //If only i remembered what the fuck i was doing lol

            //Initialize the buffer if it is not created
            if self.light_buffer.get().is_none() {
                let device = DEVICE.get().unwrap();

                //Create an empty buffer
                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Lighting buffer"),
                    size: size_of::<LightBuffer>() as u64 + 4,
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let bingroup_layout = device.create_bind_group_layout(
                    &grimoire::DIRECTIONAL_LIGHT_BIND_GROUP_LAYOUT_DESCRIPTOR,
                );

                let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Light bindgroup"),
                    layout: &bingroup_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &buf,
                            offset: 0,
                            size: None,
                        }),
                    }],
                });

                //The buffer should be all zeros, so it should be fine to not initialize it?
                //Tho it may contain garbage data, and the docs don't say anything about it
                self.light_buffer.set((buf, bindgroup)).unwrap();
            }

            //get the light object

            let light = world.get_unique_component::<DirectionalLight>();
            if let Some(light) = light {
                let device = DEVICE.get().unwrap();
                //Update the light buffer if there is a light source
                let mut l = light.borrow().get_light();

                l.camera_direction = camera.view_direction();

                let mut belt = STAGING_BELT.get().unwrap().write().unwrap();
                belt.write_buffer(
                    encoder,
                    &self.light_buffer.get().unwrap().0,
                    0,
                    NonZeroU64::new(size_of::<LightBuffer>() as u64).unwrap(),
                    device,
                )
                .copy_from_slice(bytemuck::bytes_of(&l));
            }

            //Create a point lights buffer even if there are no point lights
            if self.point_light_buffer.get().is_none() {
                log::info!("Creating point lights buffer");
                //Initialize the buffer and the bindgroups
                let device = DEVICE.get().unwrap();

                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Empty point lights buffer"),
                    size: size_of::<PointLight>() as u64 * 256,
                    usage: BufferUsages::COPY_DST
                        | if *self.storage_buffer_available.get().unwrap() {
                            BufferUsages::STORAGE
                        } else {
                            BufferUsages::UNIFORM
                        },
                    mapped_at_creation: false,
                });

                let buf1 = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Empty point lights buffer"),
                    size: 16,
                    usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                    mapped_at_creation: false,
                });

                let bg_layout =
                    device.create_bind_group_layout(&point_light_bind_group_layout_descriptor(
                        *self.storage_buffer_available.get().unwrap(),
                    ));

                let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Point lights bind group"),
                    layout: &bg_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                buf1.as_entire_buffer_binding(),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(buf.as_entire_buffer_binding()),
                        },
                    ],
                });

                self.point_light_buffer
                    .set(PointLights {
                        buffer: buf,
                        count_buf: buf1,
                        num_lights: 0,
                        bindgroup: bg,
                    })
                    .unwrap();
            }

            //Handle point lights
            if let Some(lights) = world.get_all_components::<PointLight>() {
                #[repr(C)]
                #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
                struct PointLight {
                    position: Vec3,
                    intensity: f32,
                    color: Vec3,
                    range: f32,
                }

                let device = DEVICE.get().unwrap();

                //Check if need to create a new buffer
                if self.point_light_buffer.get().unwrap().num_lights < lights.len() {
                    let p_l = self.point_light_buffer.get_mut().unwrap();
                    //Recreate the buffer with the new size
                    p_l.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Point lights buffer"),
                        size: if *self.storage_buffer_available.get().unwrap() {
                            lights.len() as u64 * 8 * 4
                        } else {
                            size_of::<PointLight>() as u64 * 256
                        },

                        usage: wgpu::BufferUsages::COPY_DST
                            | if *self.storage_buffer_available.get().unwrap() {
                                BufferUsages::STORAGE
                            } else {
                                BufferUsages::UNIFORM
                            },

                        mapped_at_creation: false,
                    });
                    let layout =
                        device.create_bind_group_layout(&point_light_bind_group_layout_descriptor(
                            *self.storage_buffer_available.get().unwrap(),
                        ));

                    let belt = STAGING_BELT.get().unwrap();
                    belt.write()
                        .unwrap()
                        .write_buffer(
                            encoder,
                            &p_l.count_buf,
                            0,
                            NonZeroU64::new(4).unwrap(),
                            device,
                        )
                        .copy_from_slice(bytemuck::bytes_of(&(lights.len() as u32)));

                    p_l.bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Point lights bindgroup"),
                        layout: &layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: p_l.count_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(
                                    p_l.buffer.as_entire_buffer_binding(),
                                ),
                            },
                        ],
                    });
                }

                let data = lights
                    .iter()
                    .map(|l| {
                        let l = l.borrow();
                        PointLight {
                            position: l.transform_ref.get().unwrap().borrow().position_global(),
                            intensity: l.get_intensity(),
                            color: l.get_color().into(),
                            range: l.get_range(),
                        }
                    })
                    .collect::<Vec<_>>();

                let data = data
                    .iter()
                    .flat_map(bytemuck::bytes_of)
                    .copied()
                    .collect::<Vec<_>>();

                let mut belt = STAGING_BELT.get().unwrap().write().unwrap();

                belt.write_buffer(
                    encoder,
                    &self.point_light_buffer.get().unwrap().buffer,
                    0,
                    NonZeroU64::new(lights.len() as u64 * 8 * 4).unwrap(),
                    device,
                )
                .copy_from_slice(data.as_slice());
            }
        }

        trace!("Starting the render pass");

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("First pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &attachments.color,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color.into()),
                    store: wgpu::StoreOp::Store,
                },
                //I hope this is fine, i can't find any info on this
                depth_slice: None,
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

        //Set the camera
        camera.set_bindgroup(&mut render_pass);

        let mut previous_mat = 0;

        trace!("Rendering materials");
        //Iterate through the meshes and render them
        for (i, m) in self.mesh_materials.iter().enumerate() {
            let mat = m.material_id;

            if mat != previous_mat {
                let mat = assets.borrow_by_id::<Material>(mat).unwrap();

                if mat.is_lit() {
                    render_pass.set_bind_group(
                        grimoire::DIRECT_LIGHT_BIND_GROUP_INDEX,
                        &self.light_buffer.get().unwrap().1,
                        &[],
                    );
                    if let Some(light) = self.point_light_buffer.get() {
                        render_pass.set_bind_group(
                            grimoire::POINT_LIGHT_BIND_GROUP_INDEX,
                            &light.bindgroup,
                            &[],
                        );
                    }
                }
                mat.render(&mut render_pass);
            }
            previous_mat = mat;

            let mesh = assets.borrow_by_id::<Mesh>(m.mesh_id).unwrap();

            render_pass.set_vertex_buffer(0, mesh.get_vertex_buffer().slice(..));
            render_pass.set_vertex_buffer(1, self.v_buffers[i].slice(..));

            render_pass
                .set_index_buffer(mesh.get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(
                0..mesh.get_index_count(),
                0,
                0..(self.num_instances[i] as u32),
            );
        }
        drop(render_pass);
    }

    fn get_priority(&self) -> u32 {
        self.priority
    }
}

fn calculate_frustum(near: f32, far: f32, fov: f32) -> Vec3 {
    let beta = f32::consts::FRAC_PI_2 - (fov / 2.0);
    let bottom = 2.0 * (((near + far) * f32::sin(fov / 2.0)) / f32::sin(beta));

    let resolution = RESOLUTION.read().unwrap();
    let aspect = resolution.width as f32 / resolution.height as f32;
    drop(resolution);

    let side = bottom / aspect;

    (bottom, side, near + far).into()
}

fn calculate_frustum_matrix(frustum: Vec3, camera_transform: Mat4x4) -> Mat4x4 {
    let scale = Mat4x4::scale_matrix((Vec3::new(1.0 / frustum.x, 1.0, 1.0 / frustum.y)));
    let translation = Mat4x4::translation_matrix(Vec3::new(0.0, frustum.z, 0.0));
    let rotation = Mat4x4::rotation_matrix_euler(Vec3::new(90.0, 90.0, 0.0));

    translation * scale * rotation * camera_transform.inverted().unwrap()
}

fn check_frustum(
    h: f32,
    frustum_matrix: Mat4x4,
    point: Vec3,
    radius: f32,
    scale: Vec3,
) -> (bool, f32) {
    let p: Vec4 = (point, 1.0).into();

    let p = p * frustum_matrix;
    let p = p.xyz();

    let distance = sdf(p, h);

    if distance <= 0.0 {
        return (true, distance);
    }

    //Factor in scale
    (
        radius.mul_add(-f32::max(scale.x, f32::max(scale.y, scale.z)), distance) <= 0.001,
        distance,
    )
}

#[allow(clippy::many_single_char_names)]
fn sdf(mut p: Vec3, h: f32) -> f32 {
    // Original SDF license:
    // The MIT License
    // Copyright © 2019 Inigo Quilez
    // Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

    if p.y <= 0.0 {
        let p = p.abs() - Vec3::new(0.5, 0.0, 0.5);

        return if p > Vec3::new(0.0, 0.0, 0.0) {
            p.length()
        } else {
            0.0
        };
    }

    //Symmetry
    p.x = f32::abs(p.x);
    p.z = f32::abs(p.z);

    if p.z > p.x {
        std::mem::swap(&mut p.x, &mut p.z);
    }
    p.x -= 0.5;
    p.z -= 0.5;

    //project into face plane (2d)

    let m2 = h.mul_add(h, 0.25);

    let q = Vec3::new(p.z, h.mul_add(p.y, -(0.5 * p.x)), h.mul_add(p.x, 0.5 * p.y));

    let sign = f32::signum(f32::max(q.z, -p.y));

    if sign <= 0.0 {
        return f32::NEG_INFINITY;
    }

    let s = f32::max(-q.x, 0.0);

    let t = f32::clamp(0.5f32.mul_add(-q.x, q.y) / (m2 + 0.25), 0.0, 1.0);

    let a = (m2 * (q.x + s)).mul_add(q.x + s, q.y * q.y);

    let b = (m2 * 0.5f32.mul_add(t, q.x)).mul_add(
        0.5f32.mul_add(t, q.x),
        (m2.mul_add(-t, q.y)) * (m2.mul_add(-t, q.y)),
    );

    let d2 = if f32::max(-q.y, q.x.mul_add(m2, q.y * 0.5)) < 0.0 {
        0.0
    } else {
        f32::min(a, b)
    };

    f32::sqrt(q.z.mul_add(q.z, d2) / m2)
}
