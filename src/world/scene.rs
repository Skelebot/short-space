use super::ncollide3d as nc;
use world::entity::Entity;
use world::entity::{Map, PhysicsEntity, DebugArrow};
use world::atlas::Atlas;
use physics::Physics;
use assets::AssetLoader;
use crate::render_gl::model::Model;
use gl;
use nalgebra as na;
use failure::Error;
use crate::camera::FpsCamera;

pub struct Scene {
    pub atlas: Atlas,
    pub entities: Vec<Box<dyn Entity>>,
    pub debug_arrow: Option<DebugArrow>,
    pub physics: Physics,
}

impl Scene {
    pub fn new(res: &AssetLoader, gl: &gl::Gl, debug: bool, camera: FpsCamera) -> Result<Scene, Error> {
        let mut physics = Physics::new(-20.0);

        let dice = PhysicsEntity::load_entity(
            Model::new(&res, &gl, "models/xyz_cube.obj", "shaders/model", debug)?,
            &mut physics,
            na::Isometry3::from_parts(
                na::Translation::from(na::Point3::new(1.0, 10.0, 3.0).coords),
                na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0)),
            nc::shape::ShapeHandle::new(
                nc::shape::Cuboid::new(
                    na::Vector3::repeat(1.0))),
        );
        
        let map = Map::load_map(
            Model::new(&res, &gl, "models/skatepark.obj", "shaders/model", debug)?,
            &mut physics
        );
        
        let atlas = Atlas::new(
            &mut physics, 
            na::Isometry3::from_parts(
                na::Translation::from(na::Point3::new(0.0, 6.0, -2.0).coords),
                na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0)),
            camera);
        let mut debug_arrow = None;
        if debug { 
            let arrow_model = Model::new(&res, &gl, "models/arrow.obj", "shaders/model", debug)?;
            debug_arrow = Some(DebugArrow::new(
                arrow_model, 
                na::Isometry3::from_parts(
                    na::Translation3::from(na::Vector3::new(0.0, 0.5, 0.0)),
                    na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0)),
            ));
        }

        println!("Scene loaded.");

        Ok(Scene {
            atlas: atlas,
            entities: vec!(Box::new(map), Box::new(dice)),
            debug_arrow: debug_arrow,
            physics: physics,
        })
    }

    pub fn physics_step(&mut self) -> Result<(), Error> {
        //Update physical rigidbodies from entity position/velocity
        for entity in &self.entities {
            entity.update_physics_from_self(&mut self.physics)?;
        }
        self.atlas.update_physics_from_self(&mut self.physics)?;
        
        //Simulate
        self.physics.step();
        
        //Update entity position/velocity from simulated rigidbodies
        for entity in &mut self.entities {
            entity.update_self_from_physics(&self.physics)?;
        }
        self.atlas.update_self_from_physics(&self.physics)?;

        Ok(())
    }
}
