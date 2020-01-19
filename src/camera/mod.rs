extern crate nalgebra;

pub use self::spectator_camera::SpectatorCamera;
pub use self::fps_camera::FpsCamera;

use crate::input::MovementDirection;
use nalgebra as na;

pub mod spectator_camera;
pub mod fps_camera;

pub trait Camera {
    fn update_aspect(&mut self, aspect: f32);
    fn update_vectors(&mut self);
    fn get_view_matrix(&self) -> na::Matrix4<f32>;
    fn get_projection_matrix(&self) -> na::Matrix4<f32>;
    fn get_vp_matrix(&self) -> na::Matrix4<f32>;
    fn rotate(&mut self, xoffset: f32, yoffset: f32);
    fn process_movement(&mut self, direction: MovementDirection, movement_speed: f32);
    fn get_position(&self) -> na::Point3<f32>;
}
