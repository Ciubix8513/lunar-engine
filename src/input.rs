use std::sync::{OnceLock, RwLock};

use vec_key_value_pair::VecMap;
use winit::keyboard::KeyCode;

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

///Updates the states, downgrading Down and Up into Pressed and Neutral respectively
pub(crate) fn update() {
    let mut i = INPUT.get().unwrap().key_map.write().unwrap();

    for v in i.values_mut() {
        *v = match v {
            KeyState::Pressed | KeyState::Down => KeyState::Pressed,
            KeyState::Up | KeyState::Neutral => KeyState::Neutral,
        }
    }
}
