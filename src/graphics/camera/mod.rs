extern crate nalgebra;

use nalgebra as na;

pub struct Camera {
    pub position: na::Isometry3<f32>,
    projection: na::Perspective3<f32>,
}
impl Camera {
    pub fn new(
        aspect: f32,
        fov: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let pos = na::Isometry3::from_parts(
            na::Translation3::from(
                na::Vector3::repeat(0.0)
            ),
            na::UnitQuaternion::from_axis_angle(
                &na::Vector3::z_axis(),
                0.0
            )
        );
        let proj = na::Perspective3::new(aspect, fov, znear, zfar);
        Camera {
            position: pos,
            projection: proj,
        }
    }

    pub fn update_aspect(&mut self, aspect: f32) {
        self.projection.set_aspect(aspect);
    }

    pub fn get_view_matrix(&self) -> na::Matrix4<f32> {
        let position: na::Point3<f32> = 
            self.position.translation.vector.into();
        let target: na::Point3<f32> = 
            self.position.translation
            * self.position.rotation
            * na::Point3::new(1.0, 0.0, 0.0);
        let up: na::Vector3<f32> = 
            self.position.translation
            * self.position.rotation
            * na::Vector3::new(0.0, 0.0, 1.0);
        na::Matrix::look_at_rh(&position, &target, &up)
    }

    pub fn get_projection_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner()
    }

    #[allow(dead_code)]
    pub fn get_vp_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner() * self.get_view_matrix()
    }

    pub fn get_position(&self) -> na::Isometry3<f32> {
        self.position
    }

    pub fn set_position(&mut self, position: na::Isometry3<f32>) {
        self.position = position;
    }
}