use core::f32;
use std::{mem, num::NonZeroU64, sync::Arc};

use log::{debug, trace};
use vec_key_value_pair::set::VecSet;
use wgpu::util::DeviceExt;

use crate::{
    asset_managment::AssetStore,
    assets::{BindgroupState, Material, Mesh},
    components,
    ecs::{ComponentReference, World},
    math::{Mat4x4, Vec2, Vec3, Vec4, Vector},
    structures::Color,
    DEVICE, RESOLUTION, STAGING_BELT,
};

use super::{AttachmentData, RenderingExtension};

///Base but with frustum culling
#[derive(Default)]
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
        trace!("Started frame");

        //Update camera first
        let binding = world
            .get_all_components::<components::camera::MainCamera>()
            .expect("Could not find the main camera");

        let camera = binding.first().unwrap().borrow();
        camera.update_gpu(encoder);
        trace!("Accquired camera");

        let frustum = calculate_frustum(
            camera.inner.near,
            camera.inner.far,
            camera.inner.projection_type.fov().unwrap_or_default(),
        );
        let camera_transform = camera.camera_transform();

        //This is cached, so should be reasonably fast
        let binding = world
            .get_all_components::<crate::components::mesh::Mesh>()
            .unwrap_or_default();

        //Precompute the transformation matrix, since it's the same for all the objects
        let matrix = calculate_frustum_matrix(frustum, camera_transform);

        let meshes = binding
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
                            .get_by_id::<Mesh>(m.get_mesh_id().unwrap())
                            .unwrap()
                            .borrow()
                            .get_extent(),
                        t.scale,
                    )
                    .0
            })
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
            assert_eq!(
                v_buffers.len(),
                mesh_materials.len(),
                "You are a moron, they're not the same"
            );
            assert_eq!(
                v_buffers.len(),
                mesh_refs.len(),
                "You are stupid, they're not the same"
            );
            assert_eq!(
                num_instances.len(),
                mesh_materials.len(),
                "You are an idiot, they're not the same"
            );

            self.v_buffers = v_buffers;
            self.mesh_materials = mesh_materials;
            self.num_instances = num_instances;
            self.mesh_refs = mesh_refs;
        } else {
            //Reusing data
            trace!("Cache exists, updating v buffers");
            let mut belt = STAGING_BELT.get().unwrap().write().unwrap();
            let device = DEVICE.get().unwrap();

            for (buffer, meshes) in self.v_buffers.iter().zip(self.mesh_refs.iter()) {
                //I do have to collect here
                // let matrices = matrices.iter().map(|i| i.1 .0).collect::<Vec<_>>();

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
            let m = assets.get_by_id::<Material>(m).unwrap();
            let mut m = m.borrow_mut();

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
                let mat = assets.get_by_id::<Material>(mat).unwrap();
                let mat = mat.borrow();

                mat.render(&mut render_pass);
            }
            previous_mat = mat;

            let mesh = assets.get_by_id::<Mesh>(m.mesh_id).unwrap();
            let mesh = mesh.borrow();

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

fn calculate_frustum(near: f32, far: f32, fov: f32) -> Vec3 {
    let beta = f32::consts::FRAC_PI_2 - (fov / 2.0);
    let bottom = 2.0 * (((near + far) * f32::sin(fov / 2.0)) / f32::sin(beta));

    let resolution = RESOLUTION.read().unwrap();
    let aspect = resolution.width as f32 / resolution.height as f32;
    drop(resolution);

    // let aspect = 1.3333333334;

    let side = bottom / aspect;

    (bottom, side, near + far).into()
}

fn calculate_frustum_matrix(frustum: Vec3, camera_transform: Mat4x4) -> Mat4x4 {
    let h = frustum.z;

    let scale = Mat4x4::scale_matrix(&(Vec3::new(frustum.x, 1.0, frustum.y)))
        .invert()
        .unwrap();
    let translation = Mat4x4::translation_matrix(&Vec3::new(0.0, h, 0.0));
    let rotation = Mat4x4::rotation_matrix_euler(&Vec3::new(-90.0, 0.0, 90.0))
        .invert()
        .unwrap();

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

// fn sdf(mut p: Vec3, h: f32) -> f32 {
//     // Original SDF license:
//     // The MIT License
//     // Copyright © 2019 Inigo Quilez
//     // Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//     if p.y <= 0.0 {
//         let p = p.abs() - Vec3::new(0.5, 0.0, 0.5);

//         return if p > Vec3::new(0.0, 0.0, 0.0) {
//             p.length()
//         } else {
//             0.0
//         };
//     }

//     //Symmetry
//     p.x = f32::abs(p.x);
//     p.z = f32::abs(p.z);

//     if p.z > p.x {
//         p.x = p.z;
//         p.z = p.x;
//     }
//     p.x -= 0.5;
//     p.z -= 0.5;

//     //project into face plane (2d)

//     let m2 = h * h + 0.25;

//     let q = Vec3::new(p.z, h * p.y - 0.5 * p.x, h * p.x + 0.5 * p.y);

//     let sign = f32::signum(f32::max(q.z, -p.y));

//     // if sign <= 0.0 {
//     //     return (true, -1.0);
//     // }

//     let s = f32::max(-q.x, 0.0);

//     let t = f32::clamp((q.y - 0.5 * q.x) / (m2 + 0.25), 0.0, 1.0);

//     let a = m2 * (q.x + s) * (q.x + s) + q.y * q.y;

//     let b = m2 * (q.x + 0.5 * t) * (q.x + 0.5 * t) + (q.y - m2 * t) * (q.y - m2 * t);

//     let d2 = if f32::max(-q.y, q.x * m2 + q.y * 0.5) < 0.0 {
//         0.0
//     } else {
//         f32::min(a, b)
//     };

//     f32::sqrt((d2 + q.z * q.z) / m2) * sign
// }

fn sdf(mut p: Vec3, h: f32) -> f32 {
    let half_h = h / 2.0;

    p.x = p.x.abs();
    p.z = p.z.abs();

    if p.x > p.z {
        mem::swap(&mut p.x, &mut p.z);
    }

    let d1 = Vec3::new(f32::max(p.x - 0.5, 0.0), p.y, f32::max(p.z - 0.5, 0.0));

    let mut q = p;

    let k = (4.0 * half_h).mul_add(half_h, 0.25);

    let h1 = Vec2::dot_product(
        &(Vec2::new(q.y, q.z) - Vec2::new(0.0, 0.5)),
        &Vec2::new(0.5, h),
    ) / k;

    //I don't really care about the insides, if it is inside, it IS inside, so i can computer the
    //sqrt if it is outside
    if f32::max(h1, -p.y) < 0.0 {
        f32::NEG_INFINITY
    } else {
        q.y -= 0.5 * h1;
        q.z -= h * h1;

        q -= Vec3::new(k, half_h, -0.25) * f32::max(q.x - q.z, 0.0) / (k + 0.25);

        let d2 = p - q.clamp(0.0.into(), Vec3::new(0.5, h, 0.5));
        f32::sqrt(f32::min(d1.dot_product(&d1), d2.dot_product(&d2)))
    }
}
