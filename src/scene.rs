use entity::Entity;
use resources::Resources;
use model::Model;
use gl;
use nalgebra as na;
use failure::Error;
use light::PointLight;

pub struct Scene {
    pub entities: Vec<Entity>,
    pub lights: Vec<PointLight>
}

impl Scene {
    pub fn new(res: &Resources, gl: &gl::Gl, debug: bool) -> Result<Scene, Error> {
        let dice_model = Model::new(&res, &gl, "models/dice.obj", "shaders/cube", debug)?;
        let dice = Entity::new(
            na::Point3::new(0.0, 0.0, 3.0),
            na::UnitQuaternion::from_euler_angles(0.0, 45.0, 45.0),
            Some(dice_model));
/*        let asteroid_model = Model::new(&res, &gl, "models/asteroid1.obj", "shaders/asteroid", debug)?;
        let asteroid = Entity::new(
            na::Point3::new(2.0, 0.0, 0.0),
            na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            Some(asteroid_model));
*/
        let light = PointLight::new(
            na::Point3::new(0.0, 5.0, 0.0),
            na::Vector3::new(1.0, 1.0, 1.0), 1.0);
        Ok(Scene {
            entities: vec!(dice),
            lights: vec!(light),
        })
    }
}


            
