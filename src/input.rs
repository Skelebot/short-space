use camera::camera::Camera;
use camera::camera::MovementDirection;
use gamestate::GameState;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;

pub struct KeyboardInput {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

impl KeyboardInput {
    pub fn new() -> KeyboardInput {
        KeyboardInput {
            forward: false,
            backward: false,
            right: false,
            left: false,
        }
    }

    pub fn handle_input(&mut self, event: &sdl2::event::Event, gamestate: &mut GameState) {
        match *event {
            Event::KeyDown {
                scancode: Some(scancode),
                ..
            } => match scancode {
                Scancode::W => self.forward = true,
                Scancode::S => self.backward = true,
                Scancode::A => self.left = true,
                Scancode::D => self.right = true,
                _ => ()
            },
            Event::KeyUp {
                scancode: Some(scancode),
                ..
            } => match scancode {
                Scancode::W => self.forward = false,
                Scancode::S => self.backward = false,
                Scancode::A => self.left = false,
                Scancode::D => self.right = false,

                Scancode::Escape => gamestate.in_menu = !gamestate.in_menu,
                _ => ()
            },
            _ => ()
        }
    }

    pub fn move_camera(&self, camera: &mut Camera, delta_time: f32, sensitivity: f32) {
        if self.forward  { camera.process_movement(MovementDirection::FORWARD,   delta_time, sensitivity); }
        if self.backward { camera.process_movement(MovementDirection::BACKWARD,  delta_time, sensitivity); }
        if self.left     { camera.process_movement(MovementDirection::LEFT,      delta_time, sensitivity); }
        if self.right    { camera.process_movement(MovementDirection::RIGHT,     delta_time, sensitivity); }
    }
}

//Empty struct because we do not need any fields for now
pub struct MouseInput;

impl MouseInput {
    pub fn new() -> MouseInput { MouseInput{} }
    pub fn handle_input(&mut self, camera: &mut Camera, event: &sdl2::event::Event, delta_time: f32, sensitivity: f32) {
        match *event {
            Event::MouseMotion {
                xrel,
                yrel,
                ..
            } => {
                camera.rotate(xrel as f32, yrel as f32, delta_time, sensitivity);
            }
            _ => ()
        }
    }
}

