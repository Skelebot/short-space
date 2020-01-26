use nalgebra as na;
use super::Camera;

pub struct FpsCamera {
    pub position: na::Isometry3<f32>,
    projection: na::Perspective3<f32>,
}

impl FpsCamera {
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
                &na::Vector3::y_axis(),
                0.0
            )
        );
        let proj = na::Perspective3::new(aspect, fov, znear, zfar);
        FpsCamera {
            position: pos,
            projection: proj,
        }
    }
}

impl Camera for FpsCamera {
    fn update_aspect(&mut self, aspect: f32) {
        self.projection.set_aspect(aspect);
    }

    fn get_view_matrix(&self) -> na::Matrix4<f32> {
        let position: na::Point3<f32> = 
            na::Point3::from(
            self.position.translation.vector);
        let target: na::Point3<f32> = 
            self.position.translation
            * self.position.rotation
            * na::Translation3::from(-na::Vector3::z())
            * na::Point3::origin();
        na::Matrix::look_at_rh(&position, &target, &na::Vector3::y_axis())
    }

    fn get_projection_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner()
    }

    fn get_vp_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner() * self.get_view_matrix()
    }
}
