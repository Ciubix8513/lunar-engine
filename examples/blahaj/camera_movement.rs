use std::borrow::BorrowMut;

// use log::info;
use lunar_engine::{
    components::transform::Transform,
    // delta_time,
    ecs::{Component, ComponentReference, SelfReferenceGuard},
    input,
    math::{mat4x4::Mat4x4, vec4::Vec4},
};
use lunar_engine_derive::as_any;
use winit::keyboard::KeyCode;

#[derive(Debug)]
pub struct FreeCam {
    speed: f32,
    // sensetivity: f32,
    // invert_y: bool,
    // locked: bool,
    transorm_reference: Option<ComponentReference<Transform>>,
}

impl FreeCam {
    pub fn new(speed: f32) -> Self {
        //sensetivity: f32, invert_y: bool) -> Self {
        Self {
            speed,
            // sensetivity,
            // invert_y,
            // locked: false,
            transorm_reference: None,
        }
    }
}

impl Component for FreeCam {
    #[as_any]
    fn mew() -> Self
    where
        Self: Sized,
    {
        FreeCam {
            speed: 1.0,
            // sensetivity: 1.0,
            // invert_y: false,
            // locked: false,
            transorm_reference: None,
        }
    }

    fn set_self_reference(&mut self, reference: SelfReferenceGuard) {
        self.transorm_reference = Some(reference.get_component().unwrap())
    }

    fn update(&mut self) {
        let trans = self.transorm_reference.as_ref().unwrap().borrow();
        let old_pos = trans.position;
        // let old_rot = trans.rotation;

        let rot = Mat4x4::rotation_matrix_euler(
            &self.transorm_reference.as_ref().unwrap().borrow().rotation,
        );
        //Rotate the movent vec
        let movement_vec = (rot * get_input_vec() * lunar_engine::delta_time() * self.speed).xyz();
        drop(trans);

        let mut trans = self
            .transorm_reference
            .as_ref()
            .borrow_mut()
            .unwrap()
            .borrow_mut();
        //Aply the movent vec
        trans.position = old_pos + movement_vec;

        // if self.locked {
        //     let mouse = input::cursor_delta().normalize();

        //     let x_rot = f32::asin(mouse.x);
        //     let y_rot = f32::asin(mouse.y);

        //     let rotation = Vec3::new(y_rot, x_rot, 0.0) * self.sensetivity * delta_time();

        //     trans.rotation = old_rot + rotation;
        //     if matches!(
        //         input::key(winit::keyboard::KeyCode::Escape),
        //         input::KeyState::Down
        //     ) {
        //         self.locked = false;
        //         lunar_engine::set_cursor_grab_mode(lunar_engine::CursorState::Free);
        //         // lunar_engine::set_cursor_visible(true);
        //     }
        // } else {
        //     if matches!(
        //         input::mouse_btn(winit::event::MouseButton::Left),
        //         input::KeyState::Down
        //     ) {
        //         lunar_engine::set_cursor_grab_mode(lunar_engine::CursorState::Locked);
        //         // lunar_engine::set_cursor_visible(false);
        //         self.locked = true;
        //     }
        // }
    }
}

fn get_input_vec() -> Vec4 {
    let mut movement_vec = Vec4::new(0.0, 0.0, 0.0, 1.0);

    if matches!(
        input::key(KeyCode::KeyW),
        input::KeyState::Pressed | input::KeyState::Down
    ) {
        movement_vec.z -= 1.0;
    }
    if matches!(
        input::key(KeyCode::KeyS),
        input::KeyState::Pressed | input::KeyState::Down
    ) {
        movement_vec.z += 1.0;
    }
    if matches!(
        input::key(KeyCode::KeyA),
        input::KeyState::Pressed | input::KeyState::Down
    ) {
        movement_vec.x -= 1.0;
    }
    if matches!(
        input::key(KeyCode::KeyD),
        input::KeyState::Pressed | input::KeyState::Down
    ) {
        movement_vec.x += 1.0;
    }
    if matches!(
        input::key(KeyCode::KeyQ),
        input::KeyState::Pressed | input::KeyState::Down
    ) {
        movement_vec.y += 1.0;
    }
    if matches!(
        input::key(KeyCode::KeyE),
        input::KeyState::Pressed | input::KeyState::Down
    ) {
        movement_vec.y -= 1.0;
    }

    movement_vec
}
