use entity::Entity;
use resources::Resources;
use model::Model;
use gl;
use nalgebra as na;
use failure::Error;

pub struct Scene {
    pub entities: Vec<Entity>,
}

impl Scene {
    pub fn new(res: &Resources, gl: &gl::Gl, debug: bool) -> Result<Scene, Error> {
        let dice_model = Model::new(&res, &gl, "models/dice.obj", "shaders/cube", debug)?;
        let dice = Entity::new(
        na::Point3::new(0.0, 0.0, 3.0),
        na::UnitQuaternion::from_euler_angles(0.0, 45.0, 45.0),
        Some(dice_model));
        Ok(Scene {
            entities: vec!(dice),
        })
    }
}


            
