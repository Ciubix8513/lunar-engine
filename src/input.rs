use std::sync::{OnceLock, RwLock};

use vec_key_value_pair::VecMap;
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::math::vec2::Vec2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
///Represents the state of the key
pub enum KeyState {
    ///First frame the key was pressed
    Down,
    ///Key is pressed
    Pressed,
    ///Key was just released
    Up,
    ///Key is not pressed
    Neutral,
}

#[derive(Debug)]
pub(crate) struct InputState {
    pub(crate) key_map: RwLock<VecMap<KeyCode, KeyState>>,
    pub(crate) mouse_button_map: RwLock<VecMap<MouseButton, KeyState>>,
    pub(crate) cursor_position: RwLock<Vec2>,
    pub(crate) previous_cursor_position: RwLock<Vec2>,
    pub(crate) cursor_delta: RwLock<Vec2>,
}

pub(crate) static INPUT: OnceLock<InputState> = OnceLock::new();

///Returns the state of the requested key
pub fn key(key: KeyCode) -> KeyState {
    let mut i = INPUT.get().unwrap().key_map.write().unwrap();

    *i.entry(key).or_insert(KeyState::Neutral)
}

///Sets the state of a key
pub(crate) fn set_key(key: KeyCode, state: KeyState) {
    let mut i = INPUT.get().unwrap().key_map.write().unwrap();

    *i.entry(key).or_insert(KeyState::Neutral) = state;
}

///Sets the state of a mouse button
pub(crate) fn set_mouse_button(btn: MouseButton, state: KeyState) {
    let mut i = INPUT.get().unwrap().mouse_button_map.write().unwrap();

    *i.entry(btn).or_insert(KeyState::Neutral) = state;
}

///Sets the cursor position
pub(crate) fn set_cursor_position(pos: Vec2) {
    let mut i = INPUT.get().unwrap().cursor_position.write().unwrap();

    *i = pos;
}

///Returns the state of the requested mouse button
pub fn mouse_btn(btn: MouseButton) -> KeyState {
    let mut i = INPUT.get().unwrap().mouse_button_map.write().unwrap();

    *i.entry(btn).or_insert(KeyState::Neutral)
}

///Updates the states, downgrading Down and Up into Pressed and Neutral respectively
pub(crate) fn update() {
    let input = INPUT.get().unwrap();
    let mut i = input.key_map.write().unwrap();

    for v in i.values_mut() {
        *v = match v {
            KeyState::Pressed | KeyState::Down => KeyState::Pressed,
            KeyState::Up | KeyState::Neutral => KeyState::Neutral,
        }
    }

    let mut i = input.mouse_button_map.write().unwrap();

    for v in i.values_mut() {
        *v = match v {
            KeyState::Pressed | KeyState::Down => KeyState::Pressed,
            KeyState::Up | KeyState::Neutral => KeyState::Neutral,
        }
    }
    let cur = input.cursor_position.read().unwrap();
    let mut last = input.previous_cursor_position.write().unwrap();
    *input.cursor_delta.write().unwrap() = *cur - *last;
    *last = *cur;
}
