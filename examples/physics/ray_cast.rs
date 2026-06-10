use std::os::linux::raw::stat;

use lunar_engine::{
    components::{self, camera::MainCamera, physics::PhysObject, transform::Transform},
    ecs::ComponentReference,
    input::{self, KeyState, mouse_btn},
    math::{Mat4x4, Vec2, Vec3, Vec4, Vec4Swizzles, Vector},
    physics::Ray,
};

use crate::State;

pub fn cast(state: &mut State) {
    if mouse_btn(winit::event::MouseButton::Left) == KeyState::Down {
        let camera = state.world.get_unique_component::<MainCamera>().unwrap();

        let ray = camera
            .borrow()
            .screen_point_to_ray(input::cursor_position(), None);

        let res = state.phys_world.ray_cast(&state.world, &ray);

        if res.is_none() {
            return;
        }

        let res = res.unwrap();

        let e = res.entity.read();

        if !e.has_component::<PhysObject>() {
            return;
        }

        let mesh = e.get_component::<components::mesh::Mesh>().unwrap();

        let mut mesh = mesh.borrow_mut();

        if mesh.get_material_id().unwrap() == state.mat_hndl {
            mesh.set_material(state.mat_hndl1);
        } else {
            mesh.set_material(state.mat_hndl);
        }
        //
    }
}
