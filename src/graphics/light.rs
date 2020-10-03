use nalgebra as na;

pub struct PointLight {
    pub position: na::Point3<f32>,
    pub color: na::Vector3<f32>,
    pub strength: f32,
}

impl PointLight {
    pub fn new(position: na::Point3<f32>, color: na::Vector3<f32>, strength: f32) -> PointLight {
        PointLight {
            position: position,
            color: color,
            strength: strength,
        }
    }
}
