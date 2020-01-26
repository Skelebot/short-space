extern crate sdl2;
extern crate nphysics3d;
use sdl2::event::Event;

use crate::game_state::GameState;
use crate::settings::GameSettings;

mod keyboard_input;
mod mouse_input;
pub use self::keyboard_input::KeyboardInput;
pub use self::mouse_input::MouseInput;

pub struct Input {
    keyboard_input: KeyboardInput,
    mouse_input: MouseInput,
    pub mouse_sensitivity: f32,
    pub movement_speed: f32,
}

impl Input {
    pub fn new(mouse_sensitivity: f32, movement_speed: f32) -> Self {
        Input {
            keyboard_input: KeyboardInput::new(),
            mouse_input: MouseInput::new(),
            mouse_sensitivity: mouse_sensitivity,
            movement_speed: movement_speed,
        }
    }
    
    ///Called only when a sdl2 event happens
    pub fn handle_event(&mut self, event: &sdl2::event::Event, game_state: &mut GameState, delta: f32) {
        match *event {
            Event::KeyDown { scancode: Some(scancode), .. } => self.keyboard_input.handle_key_down(&scancode, game_state),
            Event::KeyUp { scancode: Some(scancode), .. } => self.keyboard_input.handle_key_up(&scancode),
            Event::MouseMotion { xrel, yrel, .. }
            => self.mouse_input.handle_mouse_motion(xrel, yrel, game_state, self.mouse_sensitivity, delta),
            Event::MouseButtonDown { mouse_btn, .. } => self.mouse_input.handle_button_down(mouse_btn),
            Event::MouseButtonUp { mouse_btn, .. } => self.mouse_input.handle_button_up(mouse_btn),
            _ => (),
        }
    }

    ///Called every frame, after handle_event() has been called
    pub fn update(&mut self, game_state: &mut GameState, settings: &GameSettings, delta: f32) {
        self.keyboard_input.update(game_state, settings, delta);
    }
}
