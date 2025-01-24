#![allow(clippy::too_many_lines)]

use std::{num::NonZeroU64, sync::Arc};

use log::{debug, trace};
use vec_key_value_pair::set::VecSet;
use wgpu::util::DeviceExt;

use crate::{
    asset_managment::AssetStore,
    assets::{BindgroupState, Material, Mesh},
    components,
    ecs::{ComponentReference, World},
    structures::Color,
    DEVICE, STAGING_BELT,
};

///Frustum culling experiment
pub mod frustum_culling;

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
        assets: &AssetStore,
        attachments: &AttachmentData,
    );

    ///Returns the priority of the extension, extensions with smaller priorities are rendered first.
    fn get_priority(&self) -> u32;
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
///   &state.assets,
///   &mut [&mut state.extension]
///  );
///}
///```
pub struct Base {
    ///Priority of the extension
    pub priority: u32,
    ///Clear color used for rendering
    pub clear_color: Color,
    //Stores vector of (mesh_id, material_id) for caching
    identifier: Vec<(u128, u128)>,
    v_buffers: Vec<wgpu::Buffer>,
    mesh_materials: Vec<MeshMaterial>,
    num_instances: Vec<usize>,
    mesh_refs: Vec<Vec<ComponentReference<components::mesh::Mesh>>>,
}

impl Base {
    #[must_use]
    ///Creates a new [`Base`]
    pub const fn new(order: u32) -> Self {
        Self {
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
        }
    }

    ///Creates a new [`Base`] with a pre defined clear color
    ///
    ///The clear color is the color that is used as a background
    ///
    ///Everything rendered with this extension will have that color in the parts not occupied by a mesh.
    #[must_use]
    pub const fn new_with_color(order: u32, color: Color) -> Self {
        Self {
            priority: order,
            clear_color: color,
            identifier: Vec::new(),
            v_buffers: Vec::new(),
            mesh_materials: Vec::new(),
            num_instances: Vec::new(),
            mesh_refs: Vec::new(),
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
        assets: &AssetStore,
        attachments: &AttachmentData,
    ) {
        #[cfg(feature = "tracy")]
        let _span = tracy_client::span!("Base render");

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

        let meshes = binding
            .iter()
            .filter(|i| i.borrow().get_visible())
            .collect::<Vec<_>>();
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
        //I... don't know...

        let mut matrices = matrices
            .iter()
            .zip(meshes)
            .map(|i| (i.0 .0, (i.0 .1, i.1)))
            .collect::<Vec<_>>();

        //determine if can re use cache
        let mut identical = true;

        if matrices.len() == self.identifier.len() {
            for (index, data) in self.identifier.iter().enumerate() {
                if data.0 == matrices[index].0 && data.1 == matrices[index].1 .0 {
                    continue;
                }
                identical = false;
                break;
            }
        } else {
            identical = false;
        }

        #[allow(clippy::if_not_else)]
        if !identical {
            #[cfg(feature = "tracy")]
            let _span = tracy_client::span!("Cache generation");

            debug!("Generating new cache data");
            self.identifier = matrices.iter().map(|i| (i.0, i.1 .0)).collect::<Vec<_>>();

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
                let mut current_window = matrices[points.0..points.1].iter().collect::<Vec<_>>();

                //Split into vectors and sorted by material
                //Sort the window by materials
                current_window.sort_unstable_by(|s, o| s.1 .0.cmp(&o.1 .0));

                //find where materials change, similar to how meshes were sorted
                let mut material_split_points = Vec::new();
                let mut old = 0;
                for (i, m) in current_window.iter().enumerate() {
                    if m.1 .0 != old {
                        material_split_points.push(i);
                        old = m.1 .0;
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
                    if last != (curent.0, curent.1 .0) {
                        last = MeshMaterial::new(curent.0, curent.1 .0);
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
                            .map(|i| i.1 .1.clone())
                            .collect::<Vec<_>>(),
                    );

                    let matrices = current_window
                        .iter()
                        .map(|i| i.1 .1.borrow().get_matrix())
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

        //Initialize bindgroups for all needed materials
        for m in materials {
            let mut m = assets.borrow_by_id_mut::<Material>(m).unwrap();

            if matches!(m.get_bindgroup_state(), BindgroupState::Initialized) {
                continue;
            }
            m.initialize_bindgroups(assets);
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("First pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &attachments.color,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color.into()),
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

        //Set the camera
        camera.set_bindgroup(&mut render_pass);

        let mut previous_mat = 0;

        //Iterate through the meshes and render them
        for (i, m) in self.mesh_materials.iter().enumerate() {
            let mat = m.material_id;

            if mat != previous_mat {
                let mat = assets.borrow_by_id::<Material>(mat).unwrap();
                mat.render(&mut render_pass);
            }
            previous_mat = mat;

            let mesh = assets.borrow_by_id::<Mesh>(m.mesh_id).unwrap();

            let vert = unsafe { Arc::as_ptr(&mesh.get_vertex_buffer()).as_ref().unwrap() };
            let ind = unsafe { Arc::as_ptr(&mesh.get_index_buffer()).as_ref().unwrap() };

            render_pass.set_vertex_buffer(0, vert.slice(..));
            render_pass.set_vertex_buffer(1, self.v_buffers[i].slice(..));

            render_pass.set_index_buffer(ind.slice(..), wgpu::IndexFormat::Uint32);
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
