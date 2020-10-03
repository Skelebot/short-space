use nalgebra as na;

//mod keyboard_input;
//mod mouse_input;
//pub use self::keyboard_input::KeyboardInput;
//pub use self::mouse_input::MouseInput;

use anyhow::Error;

/*
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
*/

use crate::graphics::{viewport::Viewport, camera::FpsCamera, camera::Camera};
use crate::time::Time;
use crate::game_state::GameState;
use crate::settings::GameSettings;

use legion::{World, Resources};
use sdl2::event::Event;

pub fn handle_input(_world: &mut World, res: &mut Resources) {
    // Unfortunately, we can't return anyhow::Error from systems or scheduled functions
    let mut event_pump = res.get_mut::<sdl2::EventPump>()
        .ok_or(Error::msg("EventPump not found")).unwrap();
    let sdl = res.get_mut::<sdl2::Sdl>()
        .ok_or(Error::msg("Sdl not found")).unwrap();
    let mut game_state = res.get_mut::<GameState>()
        .ok_or(Error::msg("GameState not found")).unwrap();
    let mut viewport = res.get_mut::<Viewport>()
        .ok_or(Error::msg("Viewport not found")).unwrap();
    let mut camera = res.get_mut::<FpsCamera>()
        .ok_or(Error::msg("FpsCamera not found")).unwrap();
    let gl = res.get::<gl::Gl>()
        .ok_or(Error::msg("Gl not found")).unwrap();
    let delta = res.get::<Time>()
        .ok_or(Error::msg("Time not found")).unwrap().delta;
    let settings = res.get::<GameSettings>()
        .ok_or(Error::msg("Time not found")).unwrap();

    for event in event_pump.poll_iter() {
        match event {
            sdl2::event::Event::Quit { .. } => game_state.should_exit = true,
            sdl2::event::Event::Window {
                win_event: sdl2::event::WindowEvent::Resized(w, h),
                ..
            } => {
                viewport.update_size(w, h);
                camera.update_aspect(viewport.get_aspect());
                viewport.set_used(&gl);
            },
            e => match e {
                Event::MouseMotion { xrel, yrel, .. } => {
                    if !game_state.paused {
                        let xoffset = xrel as f32 * delta * settings.mouse_sensitivity;
                        let yoffset = yrel as f32 * delta * settings.mouse_sensitivity;
                        let around_x = na::UnitQuaternion::from_axis_angle(
                            &na::Vector3::x_axis(), -yoffset);
                        let around_y = na::UnitQuaternion::from_axis_angle(
                            &na::Vector3::y_axis(), -xoffset);

                        let pos = 
                            na::Isometry3::from_parts(
                                camera.position.translation,
                                around_y
                                * camera.get_position().rotation
                                * around_x);

                        camera.set_position(pos);

                        camera.position = 
                            camera.position * around_y;
                    }
                },
                Event::KeyDown { scancode, .. } => {
                    match scancode {
                        Some(sdl2::keyboard::Scancode::Space) => {
                            game_state.paused = true;
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }

    // Release mouse cursor if the game is paused
    sdl.mouse().set_relative_mouse_mode(!game_state.paused);
}
