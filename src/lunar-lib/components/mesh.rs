#![allow(dead_code)]
use std::{num::NonZeroU64, rc::Rc};

use crate::{
    asset_managment::UUID,
    ecs::{Component, ComponentReference},
    grimoire::TRANS_BIND_GROUP_INDEX,
    math::mat4x4::Mat4x4,
    DEVICE, STAGING_BELT,
};

use super::transform::Transform;

#[derive(Debug, Default)]
pub struct Mesh {
    entity_id: crate::ecs::UUID,
    mesh_id: Option<UUID>,
    material_id: Option<UUID>,
    transform_uniform: Option<wgpu::Buffer>,
    transform_bind_group: Option<std::rc::Rc<wgpu::BindGroup>>,
    transform_reference: Option<ComponentReference<Transform>>,
}

impl Component for Mesh {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn awawa(&mut self) {
        self.gen_gpu();
    }

    #[allow(unused_variables)]
    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        self.transform_reference = Some(reference.get_component().unwrap())
    }
}

impl Mesh {
    ///Changes the asset used by the component
    ///Does not chedk if the provided id is valid
    pub fn set_mesh(&mut self, id: UUID) {
        self.mesh_id = Some(id);
    }

    ///Returns asset id of the component
    ///
    ///Returns none if it is not set
    pub const fn get_mesh_id(&self) -> Option<UUID> {
        self.mesh_id
    }

    ///Changes the asset used by the component
    ///Does not check if the provided id is valid
    pub fn set_material(&mut self, id: UUID) {
        self.material_id = Some(id);
    }

    ///Returns asset id of the component
    ///
    ///Returns none if it is not set
    pub const fn get_material_id(&self) -> Option<UUID> {
        self.material_id
    }

    ///Creates the buffers and loads data into them
    fn gen_gpu(&mut self) {
        let device = crate::DEVICE.get().unwrap();
        let label = format!("Mesh {}", self.entity_id);
        let uniform = crate::helpers::create_uniform_matrix(Some(&label));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&label),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        NonZeroU64::new(std::mem::size_of::<Mat4x4>() as u64).unwrap(),
                    ),
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&label),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        self.transform_bind_group = Some(Rc::new(bind_group));
        self.transform_uniform = Some(uniform);
    }

    pub(crate) fn update_gpu(&self, encoder: &mut wgpu::CommandEncoder) {
        let transform = self.transform_reference.as_ref().unwrap().borrow();
        let mat = transform.matrix();
        drop(transform);

        let uniform = self.transform_uniform.as_ref().unwrap();
        let device = DEVICE.get().unwrap();
        let mut staging_belt = STAGING_BELT.get().unwrap().write().unwrap();

        staging_belt
            .write_buffer(
                encoder,
                uniform,
                0,
                NonZeroU64::new(std::mem::size_of::<Mat4x4>() as u64).unwrap(),
                device,
            )
            .copy_from_slice(bytemuck::bytes_of(&mat));
    }

    pub(crate) fn set_bindgroup(&self, pass: &mut wgpu::RenderPass) {
        let rc = self.transform_bind_group.as_ref().unwrap().clone();

        //I don't like this
        //but i don't see any other sollutions
        //TODO look into other sollutions
        let rc = unsafe { Rc::as_ptr(&rc).as_ref().unwrap() };

        pass.set_bind_group(TRANS_BIND_GROUP_INDEX, rc, &[]);
    }
}
