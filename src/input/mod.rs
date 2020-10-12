use nalgebra as na;

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
    // TODO: Use a na::Vector3 (?)
    // Movement on the z axis (ducked/jumping) -1.0 to 1.0
    pub upmove: f32,
    // Movement on the x axis (backward/forward) -1.0 to 1.0
    pub fwdmove: f32,
    // Movement on the y axis (left/right) -1.0 to 1.0
    pub sidemove: f32,
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
                // TODO: Handle mouse button events
                //Event::MouseButtonDown { mouse_btn, .. } => self.mouse_input.handle_button_down(mouse_btn),
                //Event::MouseButtonUp { mouse_btn, .. } => self.mouse_input.handle_button_up(mouse_btn),
                // Handle keyboard events
                Event::KeyDown { scancode, .. } => {
                    match scancode {
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
                            // Toggle the main player's to spectator and back to normal
                            match atlas_player.state {
                                PlayerState::Spectator => atlas_player.state = PlayerState::Normal,
                                _ => atlas_player.state = PlayerState::Spectator,
                            }
                        },
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }

    // TODO: Controller support
    // Update keycodes
    let kbd_state = event_pump.keyboard_state();
    // Set fwdmove
    input_state.fwdmove = 0.0;
    if kbd_state.is_scancode_pressed(Scancode::W) { input_state.fwdmove += 1.0 }
    if kbd_state.is_scancode_pressed(Scancode::S) { input_state.fwdmove -= 1.0 }
    // Set sidemove
    input_state.sidemove = 0.0;
    if kbd_state.is_scancode_pressed(Scancode::D) { input_state.sidemove += 1.0 }
    if kbd_state.is_scancode_pressed(Scancode::A) { input_state.sidemove -= 1.0 }
    // Set upmove
    input_state.upmove = 0.0;
    if kbd_state.is_scancode_pressed(Scancode::Space) { input_state.upmove += 1.0 }
    if kbd_state.is_scancode_pressed(Scancode::LCtrl) { input_state.upmove -= 1.0 }

    println!("state: {:?}", input_state);
    // Release mouse cursor if the game is paused
    sdl.mouse().set_relative_mouse_mode(!game_state.paused);
}

// TODO: Move to a separate module
fn handle_mouse_motion (xrel: i32, yrel: i32, camera: &mut Camera, delta: f32, settings: &mut GameSettings) {

    let xoffset = xrel as f32 * delta * settings.mouse_sensitivity;
    let yoffset = yrel as f32 * delta * settings.mouse_sensitivity;

    let zrot = na::UnitQuaternion::from_axis_angle(
        &-na::Vector3::z_axis(), 
        xoffset,
    );
    let yrot = na::UnitQuaternion::from_axis_angle(
        &na::Vector3::y_axis(), 
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
    // (yrot is the first rotation performed, because it's the last one in the multiplication)

    camera.position.rotation = 
        zrot
        * camera.position.rotation
        * yrot;
}
