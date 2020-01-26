use crate::game_state::GameState;
use sdl2::keyboard::Scancode;
use nalgebra as na;
//use super::nphysics3d as np;
use crate::world::atlas_pmove::pmove;
use crate::settings::GameSettings;

pub struct KeyboardInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub duck: bool,
    pub jump_held: bool,
}

impl KeyboardInput {
    pub fn new() -> Self {
        KeyboardInput {
            forward: false,
            backward: false,
            right: false,
            left: false,
            jump: false,
            duck: false,
            jump_held: false,
        }
    }

    pub fn handle_key_down(&mut self, scancode: &Scancode, game_state: &mut GameState) {
        match scancode {
            Scancode::W => self.forward = true,
            Scancode::S => self.backward = true,
            Scancode::A => self.left = true,
            Scancode::D => self.right = true,
            Scancode::Space => self.jump = true,
            Scancode::LCtrl => self.duck = true,
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
            Scancode::Space => self.jump = false,
            Scancode::LCtrl => self.duck = false,
            _ => ()
        }
    }

    pub fn update(&mut self, game_state: &mut GameState, settings: &GameSettings, delta: f32) {
        let mut atlas = &mut game_state.active_scene.atlas;
        let mut jump_held = self.jump_held;
        pmove(&mut atlas, &self, &mut jump_held, &mut game_state.active_scene.physics, &settings, delta);
        self.jump_held = jump_held;
        //If the player didn't move, exit the function
        //if movement == na::Vector3::repeat(0.0) { return; }
        //let mov_dir = na::Matrix::normalize(&(atlas.position.rotation * movement));
        //calculate the looking direction of the player, normalized, on the XZ plane
        //let look = atlas.position * -na::Vector3::z();
        //let look_dir = na::Matrix::normalize(&na::Vector3::new(look.x, 0.0, look.z));
        //calculate the halfway vector between look_dir and mov_dir - wish_dir
        //let wish_dir = mov_dir;

        //let accel = 10.0;
        //calculate the movement,
        //q2 style
        //let currentspeed = na::Matrix::dot(velocity, &wish_dir);
        //let addspeed = movement_speed - currentspeed;
        //let accelspeed = max(accel*delta*movement_speed, 0.0);
        //addspeed = max(min(addspeed, max_accel * delta), 0.0);
        //let vel: na::Vector3<f32> = wish_dir * accelspeed * 10.0;
        //update atlas
        //atlas.velocity += np::math::Velocity::linear(vel.x, vel.y, vel.z);
        //atlas.position.translation.vector += mov_dir * movement_speed * delta;
        if let Some(debug_arrow) = &mut game_state.active_scene.debug_arrow {
            debug_arrow.position.translation.vector = atlas.position.translation.vector-na::Vector3::new(0.0, 0.5, 0.0)+atlas.velocity.linear;
            debug_arrow.position.rotation = atlas.position.rotation * na::UnitQuaternion::from_axis_angle(
                &na::Vector3::y_axis(), 90.0);
        }
    }
}
