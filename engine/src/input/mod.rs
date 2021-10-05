use nalgebra as na;

use legion::Resources;

use winit::event::VirtualKeyCode;

mod state;
pub use state::InputState;

use crate::{graphics::GraphicsShared, spacetime::Time};

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

// TODO: Consider moving this to InputState and fetching it once in the event loop
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
    let delta = na::Vector2::<f32>::new(delta.0 as f32, delta.1 as f32);
    let mut state = resources.get_mut::<InputState>().unwrap();
    state.mouse_delta = delta;
}

pub fn gen_egui_input(resources: &mut Resources) -> egui::RawInput {
    let state = resources.get::<InputState>().unwrap();
    let graphics = resources.get::<GraphicsShared>().unwrap();
    let time = resources.get::<Time>().unwrap();
    return egui::RawInput {
        screen_rect: Some(egui::Rect {
            min: egui::Pos2 {
                x: 0.0,
                y: 0.0,
            },
            max: egui::Pos2 {
                x: graphics.window.inner_size().width as f32,
                y: graphics.window.inner_size().height as f32,
            }
        }),
        pixels_per_point: Some(graphics.window.scale_factor() as f32),
        time: Some(time.delta.as_secs_f64()),
        modifiers: state.modifiers(),
        events: (),
        ..Default::default()
    }
}