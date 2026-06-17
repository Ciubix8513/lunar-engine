use lunar_engine::{
    components::{self, camera::MainCamera, physics::PhysObject},
    input::{self, KeyState, mouse_btn},
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

        let id = e.get_id();
        drop(e);

        state.world.remove_entity_by_id(id).unwrap();
    }
}
