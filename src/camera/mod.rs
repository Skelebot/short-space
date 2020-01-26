extern crate nalgebra;

use nalgebra as na;

//TODO: Fix the camera - make it a quaternion camera
//pub mod spectator_camera;
mod fps_camera;
pub use self::fps_camera::FpsCamera;

pub trait Camera {
    fn update_aspect(&mut self, aspect: f32);
    fn get_view_matrix(&self) -> na::Matrix4<f32>;
    fn get_projection_matrix(&self) -> na::Matrix4<f32>;
    fn get_vp_matrix(&self) -> na::Matrix4<f32>;
    fn get_position(&self) -> na::Isometry3<f32>;
    fn set_position(&mut self, position: na::Isometry3<f32>);
}
