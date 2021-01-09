extern crate nalgebra;

use nalgebra as na;

pub struct Camera {
    projection: na::Perspective3<f32>,
}
impl Camera {
    pub fn new(aspect: f32, fov: f32, znear: f32, zfar: f32) -> Self {
        let proj = na::Perspective3::new(aspect, fov, znear, zfar);
        Camera { projection: proj }
    }

    pub fn update_aspect(&mut self, aspect: f32) {
        self.projection.set_aspect(aspect);
    }

    pub fn get_view_matrix(&self, position: &na::Isometry3<f32>) -> na::Matrix4<f32> {
        let eye = position.translation.vector.into();

        // Important note: those axes are colinear
        // with their world-space equivalents only when the camera hasn't
        // been rotated in any way; For example if we want to prevent
        // the camera from rolling (rotating around it's local y (front)),
        // we should make sure nothing applies y rotation to it
        // outside of this function. Nothing ever should be changed here.

        // The target can be an arbitrary point in the direction the camera is pointing
        // y axis = front
        let target = position * na::Point3::new(0.0, 1.0, 0.0);

        // z axis = up
        let up = position * na::Vector3::new(0.0, 0.0, 1.0);

        na::Matrix::look_at_rh(&eye, &target, &up)
    }

    pub fn get_projection_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner()
    }

    pub fn get_view_projection_matrix(&self, position: &na::Isometry3<f32>) -> na::Matrix4<f32> {
        self.projection.into_inner() * self.get_view_matrix(position)
    }
}
