use nalgebra as na;
use crate::game_state::GameState;
use sdl2::mouse::MouseButton;
use crate::camera::Camera;

///Empty struct, we don't need any fields
pub struct MouseInput {
    left_btn: bool,
    right_btn: bool,
}

impl MouseInput {
    pub fn new() -> Self {
        MouseInput { 
            left_btn: false,
            right_btn: false,
        } 
    }

    pub fn handle_mouse_motion(&mut self, xrel: i32, yrel: i32, game_state: &mut GameState, sensitivity: f32, delta: f32) {
        if !game_state.in_menu {
            let xoffset = xrel as f32 * delta * sensitivity;
            let yoffset = yrel as f32 * delta * sensitivity;
            let around_x = na::UnitQuaternion::from_axis_angle(
                &na::Vector3::x_axis(), -yoffset);
            let around_y = na::UnitQuaternion::from_axis_angle(
                &na::Vector3::y_axis(), -xoffset);

            game_state.active_scene.atlas.camera.set_position(
                na::Isometry3::from_parts(
                    game_state.active_scene.atlas.position.translation,
                    around_y
                    * game_state.active_scene.atlas.camera.get_position().rotation
                    * around_x));
            game_state.active_scene.atlas.position = 
                game_state.active_scene.atlas.position * around_y;
        }
    }

    pub fn handle_button_down(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.left_btn = true,
            MouseButton::Right => self.right_btn = true,
            _ => (),
        }
    }
    
    pub fn handle_button_up(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.left_btn = false,
            MouseButton::Right => self.right_btn = false,
            _ => (),
        }
    }       
}
