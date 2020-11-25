use nalgebra as na;

use crate::graphics::Camera;
use crate::time::Time;
use crate::game_state::GameState;
use crate::settings::GameSettings;

use legion::{World, Resources};

use winit::event::VirtualKeyCode;
use winit::event::ElementState;

pub enum Axis {
    KeyboardAxis(VirtualKeyCode, VirtualKeyCode),
    // TODO: GamepadAxis
}

pub enum Action {
    KeyboardAction(VirtualKeyCode),
    // TODO: GamepadAction, MouseAction
}

// TODO: Move to an AxisBindings struct and load it from a config file
pub const FWD_AXIS: Axis = Axis::KeyboardAxis(VirtualKeyCode::W, VirtualKeyCode::S);
pub const SIDE_AXIS: Axis = Axis::KeyboardAxis(VirtualKeyCode::D, VirtualKeyCode::A);
pub const UP_AXIS: Axis = Axis::KeyboardAxis(VirtualKeyCode::Space, VirtualKeyCode::LControl);

// TODO: Move to an ActionBindings struct and load it from a config file
pub const SPRINT_ACTION: Action = Action::KeyboardAction(VirtualKeyCode::LShift);

// TODO: Key bindings (remove hardcoded Scancodes)
/// Tracks which keys are pressed
#[derive(Default, Debug)]
pub struct InputState {
    // TODO: Use a na::Vector2::<f64>
    last_cursor_pos: (f64, f64),
    curr_cursor_pos: (f64, f64),

    pressed_keys: [u32; 8],
}

impl InputState {
    pub fn handle_key_event(&mut self, keycode: &VirtualKeyCode, state: &ElementState) {
        let offset = *keycode as u32 / 32;
        match state {
            ElementState::Pressed => self.pressed_keys[offset as usize] |= 1 << (*keycode as u32 - (offset*32)),
            ElementState::Released => self.pressed_keys[offset as usize] &= !(1 << (*keycode as u32 - (offset*32))),
        }
    }

    pub fn is_key_pressed(&self, keycode: &VirtualKeyCode) -> bool {
        let offset = *keycode as u32 / 32;
        match self.pressed_keys[offset as usize] & 1 << (*keycode as u32 - (offset*32)) {
            0 => false,
            _ => true,
        }
    }

    /// Get the state of an axis
    /// Returns a value from -1.0 to 1.0
    pub fn get_axis_state(&self, axis: &Axis) -> f32 {
        match axis {
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


pub fn handle_keyboard_input(input: winit::event::KeyboardInput, _world: &mut World, resources: &mut Resources) {
    if let Some(vkeycode) = input.virtual_keycode {

        let mut input_state = resources.get_mut::<InputState>().unwrap();

        match (vkeycode, input.state) {
            (VirtualKeyCode::Escape, ElementState::Pressed) => {
                let mut game_state = resources.get_mut::<GameState>().unwrap();
                game_state.paused = !game_state.paused;
            },
            (keycode, state) => input_state.handle_key_event(&keycode, &state),
        }
    }
}

// TODO: Move camera/delta/settings etc to a System, InputState should be the only thing altered in this function
pub fn handle_cursor_moved(input: winit::dpi::PhysicalPosition<f64>, window: &winit::window::Window, _world: &mut World, resources: &mut Resources) {
    let time = resources.get::<Time>().unwrap();
    let game_state = resources.get::<GameState>().unwrap();
    //let atlas = resources.get::<Atlas>().unwrap();
    let settings = resources.get::<GameSettings>().unwrap();

    if !game_state.paused {
        let delta = time.delta;
        let mut camera = resources.get_mut::<Camera>().unwrap();
        //let position = entry.get_component_mut::<Position>().unwrap();
        let mut input_state = resources.get_mut::<InputState>().unwrap();
        handle_mouse_motion(
            input_state.last_cursor_pos.0 - input.x,
            input_state.last_cursor_pos.1 - input.y,
            &mut camera.position,
            delta,
            &settings
        );
        let size = window.inner_size();
        let middle = 
        winit::dpi::PhysicalPosition {
            x: size.width / 2,
            y: size.height / 2,
        };
        window.set_cursor_position(middle).unwrap();
        input_state.last_cursor_pos = (middle.x.into(), middle.y.into());
    }
}

// TODO: Move to a separate module
fn handle_mouse_motion (xrel: f64, yrel: f64, position: &mut na::Isometry3<f32>, delta: f32, settings: &GameSettings) {

    let xoffset = xrel as f32 * delta * settings.mouse_sensitivity;
    let yoffset = yrel as f32 * delta * settings.mouse_sensitivity;

    let zrot = na::UnitQuaternion::from_axis_angle(
        &na::Vector3::z_axis(), 
        xoffset,
    );
    let xrot = na::UnitQuaternion::from_axis_angle(
        &na::Vector3::x_axis(), 
        yoffset,
    );

    // Note: By changing the order of multiplications here, we can make the camera
    // do all rotations around it's own relative axes (including the z axis),
    // which would make it a full 3D-space camera. This actually isn't good 
    // in FPS games, where the player never has to "roll (rotate around relative x)"
    // the camera. To fix this, we rotate around the z axis last, so it's always
    // the world's absolute z axis.
    // To make a space-type camera, the z rotation should be performed first.
    // Note 2: When multiplying transformations, the order is actually done backwards
    // (xrot is the first rotation performed, because it's the last one in the multiplication)

    position.rotation = 
        zrot
        * position.rotation
        * xrot
}
