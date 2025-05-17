use std::cell::OnceCell;
use std::num::NonZeroU64;

use lunar_engine_derive::{alias, dependencies};
use wgpu::BufferUsages;

use crate as lunar_engine;
use crate::math::Vec3;

use crate::{
    DEVICE, RESOLUTION, STAGING_BELT,
    ecs::{Component, ComponentReference},
    grimoire::{CAMERA_BIND_GROUP_INDEX, CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR},
    math::{Mat4x4, Vec4},
};

use super::transform::Transform;

#[derive(Debug)]
///Type of the camera projection
pub enum ProjectionType {
    ///Perspective projection
    Perspective {
        ///Fov of the camera
        fov: f32,
    },
    ///Orthographic projection
    Orthographic {
        ///Half size of the viewing volume
        size: f32,
    },
}

impl ProjectionType {
    ///Returns the FOV if the type is perspective, returns `None` otherwise
    #[must_use]
    pub const fn fov(&self) -> Option<f32> {
        match self {
            Self::Perspective { fov } => Some(*fov),
            Self::Orthographic { size: _ } => None,
        }
    }

    ///Returns the size of the viewing volume if the type is orthographic, returns `None` otherwise
    #[must_use]
    pub const fn size(&self) -> Option<f32> {
        match self {
            Self::Perspective { fov: _ } => None,
            Self::Orthographic { size } => Some(*size),
        }
    }
}

#[derive(Debug)]
///Camera used for rendering of the objects
pub struct Camera {
    ///Projection type of the camera
    pub projection_type: ProjectionType,
    ///Near plane of the camera
    pub near: f32,
    ///Far plane of the camera
    pub far: f32,
    transorm_reference: OnceCell<ComponentReference<Transform>>,
    buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
}

impl Default for Camera {
    ///The default camera has the following settings:
    /// - Fov: 60
    /// - Near plane: 0.1
    /// - Far plane: 100
    fn default() -> Self {
        Self {
            projection_type: ProjectionType::Perspective {
                fov: std::f32::consts::FRAC_PI_3,
            },
            near: 0.1,
            far: 100.0,
            transorm_reference: OnceCell::new(),
            buffer: None,
            bind_group: None,
        }
    }
}

impl Component for Camera {
    #[dependencies(Transform)]
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn awawa(&mut self) {
        self.initialize_gpu();
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transorm_reference
            .set(reference.get_component().unwrap())
            .unwrap();
    }
}

impl Camera {
    #[must_use]
    ///Creates a new Camera
    pub fn new(projection_type: ProjectionType, near: f32, far: f32) -> Self {
        Self {
            projection_type,
            near,
            far,
            ..Default::default()
        }
    }

    #[must_use]
    ///Returns the transformation matrix of the camera;
    pub fn camera_transform(&self) -> Mat4x4 {
        self.transorm_reference.get().unwrap().borrow().matrix()
    }

    #[must_use]
    ///Returns the transformation matrix of the camera multiplied by the projection matrix
    pub fn matrix(&self) -> Mat4x4 {
        let binding = self.transorm_reference.get().unwrap();
        let transform = binding.borrow();
        let rotation_matrix = Mat4x4::rotation_matrix_euler(&transform.rotation);

        let up = (rotation_matrix * Vec4::new(0.0, 1.0, 0.0, 1.0)).xyz();
        let forward = (rotation_matrix * Vec4::new(0.0, 0.0, 1.0, 1.0)).xyz() + transform.position;

        let camera_matrix = Mat4x4::look_at_matrix(transform.position, up, forward);

        let resolution = RESOLUTION.read().unwrap();
        let aspect = resolution.width as f32 / resolution.height as f32;

        drop(resolution);

        let projection_matrix = match self.projection_type {
            ProjectionType::Perspective { fov } => {
                Mat4x4::perspercive_projection(fov, aspect, self.near, self.far)
            }
            ProjectionType::Orthographic { size } => {
                Mat4x4::orth_aspect_projection(size, aspect, self.near, self.far)
            }
        };

        camera_matrix * projection_matrix
    }

    ///Initializes gpu related components of the camera: Buffers, bindgroups, etc.
    pub(crate) fn initialize_gpu(&mut self) {
        let device = DEVICE.get().unwrap();

        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            // 16 floats for a matrix, plus 4 floats for a vector
            size: 4 * 16 * 2 + 4 * 4,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buf,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        self.buffer = Some(buf);
        self.bind_group = Some(bind_group);
    }

    ///Updates the buffer of the camera with the new camera matrix
    pub(crate) fn update_gpu(&self, encoder: &mut wgpu::CommandEncoder) {
        #[repr(C)]
        #[derive(bytemuck::Zeroable, bytemuck::Pod, Clone, Copy)]
        struct CameraData {
            cam_matrix: Mat4x4,
            t_matrix: Mat4x4,
            position: Vec4,
        }

        let data = CameraData {
            cam_matrix: self.matrix(),
            t_matrix: self.matrix().invert().unwrap(),
            position: self
                .transorm_reference
                .get()
                .unwrap()
                .borrow()
                .position
                .into(),
        };

        let mut staging_belt = STAGING_BELT.get().unwrap().write().unwrap();
        staging_belt
            .write_buffer(
                encoder,
                self.buffer.as_ref().unwrap(),
                0,
                NonZeroU64::new(size_of::<CameraData>() as u64).unwrap(),
                DEVICE.get().unwrap(),
            )
            .copy_from_slice(bytemuck::bytes_of(&data));
    }

    ///Sets bindgroups of the camera for rendering
    pub(crate) fn set_bindgroup<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_bind_group(
            CAMERA_BIND_GROUP_INDEX,
            self.bind_group.as_ref().unwrap(),
            &[],
        );
    }

    ///Returns the rotated forwrard vector of the camera
    pub fn view_direction(&self) -> Vec3 {
        let t = self.transorm_reference.get().unwrap().borrow();
        let matrix = Mat4x4::rotation_matrix_euler(&t.rotation);
        drop(t);
        let forward = Vec4::new(0.0, 0.0, 1.0, 1.0);

        (forward * matrix).into()
    }
}

// #[derive(Debug, Default)]
#[alias(Camera)]
pub struct MainCamera;
