use winit::event::{ElementState, VirtualKeyCode};

use super::*;

/// Tracks which keys are pressed
#[derive(Default, Debug)]
pub struct InputState {
    pub cursor: na::Vector2<f32>,
    pub mouse_delta: na::Vector2<f32>,

    pressed_keys: [u32; 8],
}

impl InputState {
    pub fn handle_key_event(&mut self, keycode: &VirtualKeyCode, state: &ElementState) {
        let offset = *keycode as u32 / 32;
        match state {
            ElementState::Pressed => {
                self.pressed_keys[offset as usize] |= 1 << (*keycode as u32 - (offset * 32))
            }
            ElementState::Released => {
                self.pressed_keys[offset as usize] &= !(1 << (*keycode as u32 - (offset * 32)))
            }
        }
    }

    pub fn is_key_pressed(&self, keycode: &VirtualKeyCode) -> bool {
        let offset = *keycode as u32 / 32;
        self.pressed_keys[offset as usize] & 1 << (*keycode as u32 - (offset * 32)) != 0
    }

    /// Get the state of an axis
    /// Returns a value from -1.0 to 1.0
    pub fn get_axis_state(&self, axis: &Axis) -> f32 {
        match axis {
            // Simulate an axis with keyboard keys
            Axis::KeyboardAxis(pos, neg) => {
                match (self.is_key_pressed(pos), self.is_key_pressed(neg)) {
                    (false, false) => 0.0,
                    (true, true) => 0.0,
                    (true, false) => 1.0,
                    (false, true) => -1.0,
                }
            }
        }
    }

    /// Checks if a given action is pressed
    pub fn is_action_pressed(&self, action: &Action) -> bool {
        match action {
            Action::KeyboardAction(key) => self.is_key_pressed(key),
        }
    }
}
