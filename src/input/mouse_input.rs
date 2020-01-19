use crate::game_state::GameState;

///Empty struct, we don't need any fields
pub struct MouseInput;

impl MouseInput {
    pub fn new() -> Self { MouseInput {} }
    pub fn handle_mouse_motion(&mut self, xrel: i32, yrel: i32, game_state: &mut GameState, sensitivity: f32, delta: f32) {
        if !game_state.in_menu {
            let xoffset = xrel as f32 * delta * sensitivity;
            let yoffset = yrel as f32 * delta * sensitivity;
            game_state.active_camera.rotate(xoffset, yoffset);
        }
    }
}
