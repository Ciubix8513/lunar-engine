//! Functions for getting user input
use std::sync::{OnceLock, RwLock};

use vec_key_value_pair::map::VecMap;
use winit::{dpi::PhysicalPosition, event::MouseButton, keyboard::KeyCode, window::CursorGrabMode};

use crate::{math::Vec2, WINDOW};

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

// pub fn cursor_position() -> Vec2 {
//     *INPUT.get().unwrap().cursor_position.read().unwrap()
// }

// pub fn cursor_delta() -> Vec2 {
//     *INPUT.get().unwrap().cursor_delta.read().unwrap()
// }

///Updates the states, downgrading Down and Up into Pressed and Neutral respectively
pub(crate) fn update() {
    let input = INPUT.get().unwrap();

    let mut i = input.key_map.write().unwrap();
    let values = i.values_mut();

    for v in values {
        *v = match v {
            KeyState::Pressed | KeyState::Down => KeyState::Pressed,
            KeyState::Up | KeyState::Neutral => KeyState::Neutral,
        }
    }
    drop(i);

    let mut i = input.mouse_button_map.write().unwrap();
    let values = i.values_mut();

    for v in values {
        *v = match v {
            KeyState::Pressed | KeyState::Down => KeyState::Pressed,
            KeyState::Up | KeyState::Neutral => KeyState::Neutral,
        }
    }
    drop(i);

    let cur = input.cursor_position.read().unwrap();
    let mut last = input.previous_cursor_position.write().unwrap();
    *input.cursor_delta.write().unwrap() = *cur - *last;
    *last = *cur;
}

///Sets the cursor grab mode
pub fn set_cursor_grab_mode(mode: CursorLock) {
    let mut state = CURSOR_STATE.write().unwrap();
    state.grab_mode = mode;
    state.modified = true;
}

///Sets the cursor grab mode
pub fn set_cursor_visible(mode: CursorVisibily) {
    let mut state = CURSOR_STATE.write().unwrap();
    state.visible = mode;
    state.modified = true;
}

///Defines behaviour of the cursor inside the window
#[derive(Clone, Copy, Default)]
pub enum CursorLock {
    ///Cursor is locked in place
    Locked,
    ///Cursor is free
    #[default]
    Free,
}

///Defines the visibility of the cursor
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum CursorVisibily {
    #[default]
    ///Cursor is visible
    Visible,
    ///Cursor is hidden
    Hidden,
}

struct CursorStateInternal {
    grab_mode: CursorLock,
    lock_failed: bool,
    visible: CursorVisibily,
    modified: bool,
}
static CURSOR_STATE: RwLock<CursorStateInternal> = RwLock::new(CursorStateInternal {
    grab_mode: CursorLock::Free,
    lock_failed: false,
    visible: CursorVisibily::Visible,
    modified: false,
});

fn reset_cursor() {
    let window = WINDOW.get().unwrap();

    let pos = window.inner_size();
    if let Err(e) = window.set_cursor_position(PhysicalPosition {
        x: pos.width / 2,
        y: pos.height / 2,
    }) {
        log::error!("Failed to move cursor {e}");
    }
}

pub(crate) fn process_cursor() {
    let mut state = CURSOR_STATE.write().unwrap();

    if matches!(state.grab_mode, CursorLock::Locked) && state.lock_failed {
        reset_cursor();
    }

    if !state.modified {
        return;
    }
    state.modified = false;
    let window = WINDOW.get().unwrap();

    window.set_cursor_visible(match state.visible {
        CursorVisibily::Visible => true,
        CursorVisibily::Hidden => false,
    });

    let g_mode = state.grab_mode;
    let res = window.set_cursor_grab(match g_mode {
        CursorLock::Locked => CursorGrabMode::Locked,
        CursorLock::Free => CursorGrabMode::None,
    });
    if let Err(e) = res {
        match e {
            winit::error::ExternalError::NotSupported(_) => {
                //Once a lock has failed, it can never unfail, so no need to reset this
                //afterwards :3
                //
                //This can only unfail if the user changes platform, buuuut, i literally don't
                //think there's a way that could happen
                state.lock_failed = true;
                drop(state);

                log::warn!("Failed to lock cursor, doing manually");
                if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                    log::error!("Cursor is fucked :3 {e}");
                }
                reset_cursor();
            }

            winit::error::ExternalError::Ignored => {
                log::warn!("Cursor state change ignored");
            }
            winit::error::ExternalError::Os(e) => log::error!("Cursor state change error: {e}"),
        }
    }
}

///Returns the current lock state of the cursor
pub fn get_cursor_grab_mode() -> CursorLock {
    CURSOR_STATE.read().unwrap().grab_mode
}

///Returns the current visibility state of the cursor
pub fn get_cursor_visibility() -> CursorVisibily {
    CURSOR_STATE.read().unwrap().visible
}
