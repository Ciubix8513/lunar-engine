use std::sync::OnceLock;

use lunar_engine::{
    components::transform::Transform,
    delta_time,
    ecs::{Component, ComponentReference},
    input::{self, CursorLock, CursorVisibily, KeyState},
    math::{Vec3, Vector},
};
use winit::keyboard::KeyCode;

pub struct CameraControls {
    transform: OnceLock<ComponentReference<Transform>>,
    cursor_free: bool,
}

impl Component for CameraControls {
    fn mew() -> Self
    where
        Self: Sized,
    {
        input::set_cursor_grab_mode(CursorLock::Locked);
        input::set_cursor_visible(CursorVisibily::Hidden);
        Self {
            transform: OnceLock::new(),
            cursor_free: false,
        }
    }

    fn update(&mut self) {
        //Cursor stuff
        if input::key(KeyCode::Escape) == KeyState::Down {
            input::set_cursor_grab_mode(CursorLock::Free);
            input::set_cursor_visible(CursorVisibily::Visible);
        }

        if input::mouse_btn(winit::event::MouseButton::Left) == KeyState::Down && !self.cursor_free
        {
            input::set_cursor_grab_mode(CursorLock::Locked);
            input::set_cursor_visible(CursorVisibily::Hidden);
        }

        if input::key(KeyCode::AltLeft) == KeyState::Down {
            self.cursor_free = true;
            input::set_cursor_grab_mode(CursorLock::Free);
            input::set_cursor_visible(CursorVisibily::Visible);
        }

        if input::key(KeyCode::AltLeft) == KeyState::Up {
            self.cursor_free = false;
            input::set_cursor_grab_mode(CursorLock::Locked);
            input::set_cursor_visible(CursorVisibily::Hidden);
        }

        let delta_time = delta_time();

        //Rotation
        let sensetivity = 800.0;
        let delta = input::cursor_delta() * delta_time * sensetivity;
        let mut trans = self.transform.get().unwrap().borrow_mut();

        //Using a parent for y axis rotation...
        //
        //kinda scuffed but eh, should work
        let parent = trans.get_parent().clone().unwrap();
        let mut p = parent.borrow_mut();

        // trans.rotate((delta.y * 0.1, delta.x * -0.1, 0.0).into());
        if !self.cursor_free {
            trans.rotate((delta.y * 0.1, 0.0, 0).into());
            p.rotate((0, delta.x * -0.1, 0).into());
        }

        drop(p);

        //Movement
        let mut speed = 400.0;
        if input::key(KeyCode::ShiftLeft) == KeyState::Pressed {
            speed *= 2.0;
        }

        let mut movement_vec = Vec3::default();
        if input::key(KeyCode::KeyW) == KeyState::Pressed {
            movement_vec.z += 1.0;
        }
        if input::key(KeyCode::KeyS) == KeyState::Pressed {
            movement_vec.z -= 1.0;
        }
        if input::key(KeyCode::KeyA) == KeyState::Pressed {
            movement_vec.x += 1.0;
        }
        if input::key(KeyCode::KeyD) == KeyState::Pressed {
            movement_vec.x -= 1.0;
        }
        if input::key(KeyCode::KeyE) == KeyState::Pressed {
            movement_vec.y += 1.0;
        }
        if input::key(KeyCode::KeyQ) == KeyState::Pressed {
            movement_vec.y -= 1.0;
        }

        if movement_vec.square_length() == 0.0 {
            return;
        }

        movement_vec *= 0.01 * speed * delta_time;

        let mat = trans.rotation_global().matrix();
        movement_vec = mat.transform3(movement_vec);

        parent.borrow_mut().position += movement_vec;
    }

    fn set_self_reference(&mut self, reference: lunar_engine::ecs::SelfReferenceGuard) {
        self.transform
            .set(reference.get_component::<Transform>().unwrap())
            .unwrap();
    }
}
