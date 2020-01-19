use crate::game_state::GameState;
use sdl2::keyboard::Scancode;

use super::MovementDirection;

pub struct KeyboardInput {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

impl KeyboardInput {
    pub fn new() -> Self {
        KeyboardInput {
            forward: false,
            backward: false,
            right: false,
            left: false,
        }
    }

    pub fn handle_key_down(&mut self, scancode: &Scancode, game_state: &mut GameState) {
        match scancode {
            Scancode::W => self.forward = true,
            Scancode::S => self.backward = true,
            Scancode::A => self.left = true,
            Scancode::D => self.right = true,
            Scancode::Escape => game_state.in_menu = !game_state.in_menu,
            _ => ()
        }
    }
    pub fn handle_key_up(&mut self, scancode: &Scancode) {
        match scancode {
            Scancode::W => self.forward = false,
            Scancode::S => self.backward = false,
            Scancode::A => self.left = false,
            Scancode::D => self.right = false,
            _ => ()
        }
    }

    pub fn update(&self, game_state: &mut GameState, movement_speed: f32, delta: f32) {
        let sensitivity = delta * movement_speed;
        if self.forward  { game_state.active_camera.process_movement(MovementDirection::FORWARD,  sensitivity); }
        if self.backward { game_state.active_camera.process_movement(MovementDirection::BACKWARD, sensitivity); }
        if self.left     { game_state.active_camera.process_movement(MovementDirection::LEFT,     sensitivity); }
        if self.right    { game_state.active_camera.process_movement(MovementDirection::RIGHT,    sensitivity); }
    }
}
