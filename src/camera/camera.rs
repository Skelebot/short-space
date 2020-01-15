use nalgebra as na;

pub enum MovementDirection {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
}

pub struct Camera {
    pub position: na::Point3<f32>,
    pub target: na::Vector3<f32>,
    right: na::Vector3<f32>,
    up: na::Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    projection: na::Perspective3<f32>,
}

impl Camera {
    pub fn new(
        aspect: f32,
        fov: f32,        
        znear: f32,
        zfar: f32,
    ) -> Camera {
        let mut cam = Camera {
            position: na::Point3::new(0.0, 0.0, -2.0),
            target: na::Vector3::new(0.0, 0.0, -1.0),
            right: *na::Vector3::x_axis(),
            up: *na::Vector3::y_axis(),
            yaw: 90.0,
            pitch: 0.0,
            projection: na::Perspective3::new(aspect, fov, znear, zfar),
        };
        cam.update_vectors();
        (cam)
    }

    pub fn update_aspect(&mut self, aspect: f32) {
        self.projection.set_aspect(aspect);
    }
     
    fn update_vectors(&mut self) {
        let front = na::Vector3::new(
            radians(&self.yaw).cos() * radians(&self.pitch).cos(),
            radians(&self.pitch).sin(),
            radians(&self.yaw).sin() * radians(&self.pitch).cos());
        self.target = na::Matrix::normalize(&front);
        self.right = na::Matrix::normalize(&na::Matrix::cross(&self.target, &na::Vector3::y_axis()));
        self.up = na::Matrix::normalize(&na::Matrix::cross(&self.right, &self.target));
    }

    pub fn get_view_matrix(&self) -> na::Matrix4<f32> {
        let target = na::Point3::new(
            self.position.x + self.target.x,
            self.position.y + self.target.y,
            self.position.z + self.target.z);
        na::Matrix::look_at_rh(&self.position, &target, &self.up)
    }

    pub fn get_projection_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner()
    }

    pub fn get_vp_matrix(&self) -> na::Matrix4<f32> {
        self.projection.into_inner() *  self.get_view_matrix()
    }

    ///Rotate camera using relative mouse movement over screen
    pub fn rotate(&mut self, xrel: f32, yrel: f32, delta_time: f32) {
        let sensitivity = 0.00008;
        let xoffset = xrel * sensitivity * delta_time;
        let yoffset = yrel * sensitivity * delta_time;

        self.yaw += xoffset;
        self.pitch -= yoffset;
        self.pitch = *na::partial_clamp(&self.pitch, &-89.0, &89.0).unwrap();
        if self.yaw >= 360.{ self.yaw = 0.0; }

        self.update_vectors();
    }

    pub fn process_movement(&mut self, direction: MovementDirection, delta_time: f32) {
        let movement_speed = 0.000001;
        let velocity = movement_speed * delta_time;
        match direction {
            MovementDirection::FORWARD => self.position.coords += self.target * velocity,
            MovementDirection::BACKWARD => self.position.coords -= self.target * velocity,
            MovementDirection::LEFT => self.position.coords -= self.right * velocity,
            MovementDirection::RIGHT => self.position.coords += self.right * velocity,
        }
    }
}

pub fn radians(degrees: &f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

