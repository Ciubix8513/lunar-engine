use std::sync::Arc;

use log::{debug, info};
use wgpu::util::DeviceExt;

use crate::{
    asset_managment::AssetStore,
    assets::{BindgroupState, Material, Mesh},
    components,
    ecs::World,
    DEVICE,
};

pub struct AttachmentData {
    pub color: wgpu::TextureView,
    pub depth_stencil: wgpu::TextureView,
}

///Trait that all rendering extensions must implement
///
///Allows for extending the renderer
pub trait RenderingExtension {
    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        assets: &AssetStore,
        attachments: &AttachmentData,
    );

    fn get_order(&self) -> u32;
}

impl std::cmp::PartialEq for dyn RenderingExtension {
    fn eq(&self, other: &Self) -> bool {
        self.get_order().eq(&other.get_order())
    }
}

impl std::cmp::Eq for dyn RenderingExtension {}

impl std::cmp::PartialOrd for dyn RenderingExtension {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_order().partial_cmp(&other.get_order())
    }
}

impl std::cmp::Ord for dyn RenderingExtension {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_order().cmp(&other.get_order())
    }
}

#[derive(Clone)]
pub struct Base {
    order: u32,
}
impl Base {
    pub fn new(order: u32) -> Self {
        Self { order }
    }
}

impl RenderingExtension for Base {
    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        assets: &AssetStore,
        attachments: &AttachmentData,
    ) {
        debug!("Started frame");
        let binding = world
            .get_all_components::<components::camera::MainCamera>()
            .expect("Could not find the main camera");
        let camera = binding.first().unwrap();

        //This is cached, so should be reasonably fast
        let meshes = world
            .get_all_components::<crate::components::mesh::Mesh>()
            .unwrap_or_default();

        let mut camera = camera.borrow_mut();

        camera.update_gpu(encoder);
        let mut materials = Vec::new();

        let mut matrices = Vec::new();
        //Collect all the matrices
        for m in &meshes {
            let m = m.borrow();
            materials.push(m.get_material_id().unwrap());
            matrices.push((
                m.get_mesh_id().unwrap(),
                (m.get_matrix(), m.get_material_id().unwrap()),
            ));
        }

        //Sort meshes for easier buffer creation
        matrices.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        //This is so jank omg

        //Find points where they differ
        let mut split_points = Vec::new();
        let mut old = 0;
        for (i, m) in matrices.iter().enumerate() {
            if m.0 != old {
                split_points.push(i);
                old = m.0;
            }
        }
        //Guarantee that there's at least 1 window
        split_points.push(matrices.len());

        //assemble vertex buffers
        let mut v_buffers = Vec::new();

        let device = DEVICE.get().unwrap();

        let mut m_m = Vec::new();
        let mut num_instances = Vec::new();

        for m in split_points.windows(2) {
            let points = (*m.first().unwrap(), *m.last().unwrap());
            let label = format!("Instances: {}..{}", m.first().unwrap(), m.last().unwrap());

            //(Mesh, (Matrix, Material))
            let mut stuff = matrices[points.0..points.1].iter().collect::<Vec<_>>();

            //Split into vectors and sorted by material
            stuff.sort_unstable_by(|s, o| s.1 .1.cmp(&o.1 .1));

            let mut split_points = Vec::new();
            let mut old = 0;
            for (i, m) in stuff.iter().enumerate() {
                if m.1 .1 != old {
                    split_points.push(i);
                    old = m.1 .1;
                }
            }
            split_points.push(stuff.len());

            let mut last = (0, 0);
            //Need to iterate over it twice...
            for i in &split_points[..split_points.len() - 1] {
                let curent = stuff[*i];
                if last != (curent.0, curent.1 .1) {
                    last = (curent.0, curent.1 .1);
                    m_m.push(last);
                }
            }

            //AGAIN!?!?
            for m in split_points.windows(2) {
                //Now this is sored per mesh per material
                let points = (*m.first().unwrap(), *m.last().unwrap());
                num_instances.push(points.1 - points.0);
                let matrices = stuff
                    .iter()
                    .map(|i| bytemuck::bytes_of(&i.1 .0))
                    .flatten()
                    .copied()
                    .collect::<Vec<u8>>();
                v_buffers.push(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&label),
                        contents: &matrices,
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                );
            }
        }
        //Check if they're the same length
        assert_eq!(
            v_buffers.len(),
            m_m.len(),
            "You are a moron, they're not the same"
        );
        assert_eq!(
            num_instances.len(),
            m_m.len(),
            "You are an idiot, they're not the same"
        );

        //Initialize bindgroups for all needed materials
        for m in materials {
            let m = assets.get_by_id::<Material>(m).unwrap();
            let mut m = m.borrow_mut();

            if let BindgroupState::Initialized = m.get_bindgroup_state() {
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
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
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
        for (i, m) in m_m.iter().enumerate() {
            let mat = m.1;

            if mat != previous_mat {
                let mat = assets.get_by_id::<Material>(mat).unwrap();
                let mat = mat.borrow();

                mat.render(&mut render_pass);
            }
            previous_mat = mat;

            let mesh = assets.get_by_id::<Mesh>(m.0).unwrap();
            let mesh = mesh.borrow();

            let vert = unsafe { Arc::as_ptr(&mesh.get_vertex_buffer()).as_ref().unwrap() };
            let ind = unsafe { Arc::as_ptr(&mesh.get_index_buffer()).as_ref().unwrap() };

            render_pass.set_vertex_buffer(0, vert.slice(..));
            render_pass.set_vertex_buffer(1, v_buffers[i].slice(..));

            render_pass.set_index_buffer(ind.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.get_index_count(), 0, 0..(num_instances[i] as u32))
        }
        drop(render_pass);
    }

    fn get_order(&self) -> u32 {
        self.order
    }
}
