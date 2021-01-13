use nalgebra as na;

use legion::Resources;

use winit::event::VirtualKeyCode;

mod state;
pub use state::InputState;

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

// TODO: Consider moving all of this to InputState and fetching it once in the event loop
pub fn prepare(resources: &mut Resources) {
    let mut state = resources.get_mut::<InputState>().unwrap();
    state.mouse_delta = na::zero();
}

pub fn handle_keyboard_input(input: winit::event::KeyboardInput, resources: &mut Resources) {
    if let Some(vkeycode) = input.virtual_keycode {
        let mut input_state = resources.get_mut::<InputState>().unwrap();

        input_state.handle_key_event(&vkeycode, &input.state);
    }
}

pub fn handle_mouse_movement(delta: (f64, f64), resources: &mut Resources) {
    let delta = na::Vector2::<f32>::new(-delta.0 as f32, -delta.1 as f32);
    let mut state = resources.get_mut::<InputState>().unwrap();
    state.mouse_delta = delta;
}
