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

    pub fn view(&self, eye: na::Point3<f32>, _yaw_deg: f32, pitch_deg: f32) -> na::Matrix4<f32> {
        static mut LAST_FRONT: Option<na::Vector3<f32>> = None;

        let front = na::UnitQuaternion::from_axis_angle(&na::Vector::x_axis(), pitch_deg)
            * na::Vector3::y();

        unsafe {
            if let Some(last_front) = LAST_FRONT {
                if last_front != front {
                    log::debug!("delta_front: {:?}", last_front - front);
                }
            }
            LAST_FRONT = Some(front);
        }

        //let front = -na::Vector3::new(
        //    yaw_rad.sin() * pitch_rad.cos(),
        //    yaw_rad.cos() * pitch_rad.cos(),
        //    pitch_rad.sin(),
        //);

        let front = eye + front;

        // Important note: those axes are colinear
        // with their world-space equivalents only when the camera hasn't
        // been rotated in any way; For example if we want to prevent
        // the camera from rolling (rotating around it's local y (front)),
        // we should make sure nothing applies y rotation to it
        // outside of this function. Nothing ever should be changed here.

        // y axis = front
        // let target = position * na::Point3::new(0.0, 1.0, 0.0);

        // z axis = up
        let up = na::Vector3::new(0.0, 0.0, 1.0);

        na::Matrix::look_at_rh(&eye, &front, &up)
    }

    pub fn projection(&self) -> na::Matrix4<f32> {
        self.projection.into_inner()
    }
}
