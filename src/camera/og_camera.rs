use nalgebra as na;
use super::MovementDirection;
use super::Camera;

///FPS-Style camera, where front and right are always on the XZ plane
pub struct FpsCamera {
    pub position: na::Point3<f32>,
    target: na::Vector3<f32>,
    front: na::Vector3<f32>,
    right: na::Vector3<f32>,
    up: na::Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    projection: na::Perspective3<f32>,
}

impl FpsCamera {
    pub fn new(
        aspect: f32,
        fov: f32,        
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut cam = FpsCamera {
            position: na::Point3::new(0.0, 0.0, 0.0),
            target: na::Vector3::new(0.0, 0.0, -1.0),
            front: na::Vector3::new(0.0, 0.0, -1.0),
            right: *na::Vector3::x_axis(),
            up: *na::Vector3::y_axis(),
            yaw: 90.0,
            pitch: 0.0,
            projection: na::Perspective3::new(aspect, fov, znear, zfar),
        };
        cam.update_vectors();
        (cam)
    }
}

impl Camera for FpsCamera {

    fn update_aspect(&mut self, aspect: f32) {
        self.projection.set_aspect(aspect);
    }

    fn update_vectors(&mut self) {
        self.front = na::Matrix::normalize(
            &na::Vector3::new(
                self.yaw.to_radians().cos(),
                0.0,
                self.yaw.to_radians().sin()
            )
        );
        self.target = na::Matrix::normalize(
            &na::Vector3::new(
                self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
                self.pitch.to_radians().sin(),
                self.yaw.to_radians().sin() * self.pitch.to_radians().cos()
            )
        );
        self.right = na::Matrix::normalize(&na::Matrix::cross(&self.target, &na::Vector3::y_axis()));
        self.up = na::Matrix::normalize(&na::Matrix::cross(&self.right, &self.target));
    }

    fn get_view_matrix(&self) -> na::Matrix4<f32> {
        let target = na::Point3::new(
            self.position.x + self.target.x,
            self.position.y + self.target.y,
            self.position.z + self.target.z);
        na::Matrix::look_at_rh(&self.position, &target, &self.up)
    }

    fn get_projection_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner()
    }

    fn get_vp_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner() *  self.get_view_matrix()
    }

    ///Rotate camera using relative mouse movement over screen
    fn rotate(&mut self, xoffset: f32, yoffset: f32) {
        self.yaw += xoffset;
        self.pitch -= yoffset;
        self.pitch = *na::partial_clamp(&self.pitch, &-89.0, &89.0).unwrap();
        if self.yaw >= 360.0 { self.yaw = 0.0; }

        self.update_vectors();
    }
    
    fn process_movement(&mut self, direction: MovementDirection, movement_speed: f32) {
        let velocity = movement_speed;
        match direction {
            MovementDirection::FORWARD => self.position.coords += self.front * velocity,
            MovementDirection::BACKWARD => self.position.coords -= self.front * velocity,
            MovementDirection::LEFT => self.position.coords -= self.right * velocity,
            MovementDirection::RIGHT => self.position.coords += self.right * velocity,
        }
    } 

    fn get_position(&self) -> na::Point3<f32> {
        self.position
    }

    fn set_position(&mut self, new_pos: na::Point3<f32>) {
        self.position = new_pos;
    }
}

