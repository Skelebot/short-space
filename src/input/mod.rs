use nalgebra as na;

//mod keyboard_input;
//mod mouse_input;
//pub use self::keyboard_input::KeyboardInput;
//pub use self::mouse_input::MouseInput;

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
            _ => (),
        }
    }

    ///Called every frame, after handle_event() has been called
    pub fn update(&mut self, game_state: &mut GameState, settings: &GameSettings, delta: f32) {
        self.keyboard_input.update(game_state, settings, delta);
    }
}
*/

use crate::graphics::{Viewport, Camera};
use crate::player::{Player, Atlas, PlayerState};
use crate::time::Time;
use crate::game_state::GameState;
use crate::settings::GameSettings;

use legion::{system, world::SubWorld, world::EntityStore};
use sdl2::event::Event;
use sdl2::keyboard::Scancode;


// TODO: Key bindings (remove hardcoded Scancodes)
/// Tracks which keys are pressed
#[derive(Default, Debug)]
pub struct InputState {
    pub forward: bool, //W
    pub backward: bool,
    pub right: bool,
    pub left: bool,
    pub jump: bool,
}

#[system]
#[write_component(Player)]
pub fn handle_input(
    #[resource] event_pump: &mut sdl2::EventPump,
    #[resource] sdl: &mut sdl2::Sdl,
    #[resource] game_state: &mut GameState,
    #[resource] viewport: &mut Viewport,
    #[resource] camera: &mut Camera,
    #[resource] gl: &gl::Gl,
    #[resource] time: &mut Time,
    #[resource] settings: &mut GameSettings,
    #[resource] input_state: &mut InputState,
    #[resource] atlas: &Atlas,
    world: &mut SubWorld,
) {
    for event in event_pump.poll_iter() {
        match event {
            sdl2::event::Event::Quit { .. } => game_state.should_exit = true,
            // Handle window resizing
            sdl2::event::Event::Window {
                win_event: sdl2::event::WindowEvent::Resized(w, h),
                ..
            } => {
                viewport.update_size(w, h);
                camera.update_aspect(viewport.get_aspect());
                viewport.set_used(&gl);
            },
            e => match e {
                // Handle mouse motion (relative)
                Event::MouseMotion { xrel, yrel, .. } => {
                    if !game_state.paused {
                        let delta = time.delta;
                        handle_mouse_motion(xrel, yrel, camera, delta, settings);
                    }
                },
                //Event::MouseButtonDown { mouse_btn, .. } => self.mouse_input.handle_button_down(mouse_btn),
                //Event::MouseButtonUp { mouse_btn, .. } => self.mouse_input.handle_button_up(mouse_btn),
                // Handle keyboard events
                Event::KeyDown { scancode, .. } => {
                    match scancode {
                        Some(Scancode::W) => input_state.forward = true,
                        Some(Scancode::A) => input_state.right= true,
                        Some(Scancode::D) => input_state.left = true,
                        Some(Scancode::S) => input_state.backward = true,
                        Some(Scancode::Space) => input_state.jump = true,
                        _ => (),
                    }
                },
                // KeyUp can be used for simple not-time-critical single-press keybindings
                Event::KeyUp { scancode, .. } => {
                    match scancode {
                        Some(Scancode::Escape) => {
                            game_state.paused = !game_state.paused;
                        },
                        Some(Scancode::P) => {
                            let mut atlas_player = world.entry_mut(atlas.entity).unwrap();
                            let mut atlas_player = atlas_player.get_component_mut::<Player>().unwrap();
                            // Toggle the main player's state between Spectator and Playing
                            match atlas_player.state {
                                PlayerState::Playing => atlas_player.state = PlayerState::Spectator,
                                PlayerState::Spectator => atlas_player.state = PlayerState::Playing,
                            }
                            game_state.paused = !game_state.paused;
                        },
                        Some(Scancode::W) => input_state.forward = false,
                        Some(Scancode::A) => input_state.right= false,
                        Some(Scancode::D) => input_state.left = false,
                        Some(Scancode::S) => input_state.backward = false,
                        Some(Scancode::Space) => input_state.jump = false,
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

fn handle_mouse_motion (xrel: i32, yrel: i32, camera: &mut Camera, delta: f32, settings: &mut GameSettings) {

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
