use nalgebra as na;
use crate::model::Model;

pub struct Entity {
    pub position: na::Point3<f32>,
    pub rotation: na::UnitQuaternion<f32>,
    model: Option<Model>,
}

impl Entity {
    pub fn new(position: na::Point3<f32>, rotation: na::UnitQuaternion<f32>, model: Option<Model>) -> Entity{
        Entity {
            position: position,
            rotation: rotation,
            model: model,
        }
    } 
    pub fn render(
        &self,
        gl: &gl::Gl,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        camera_pos: &na::Point3<f32>,
    ) {
        match &self.model {
            Some(model) => {
                let transformation = na::Isometry3::from_parts(
                    na::Translation3::from(self.position.coords),
                    self.rotation);
                model.render(gl, view_matrix, proj_matrix, camera_pos, &transformation);
            },
            None => ()
        }
    }
}

