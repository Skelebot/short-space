use super::nphysics3d as np;
use super::ncollide3d as nc;
use nalgebra as na;
use crate::render_gl::model::Model;
use crate::physics::{Physics, Error};

pub trait Entity {
    fn render(&self, gl: &gl::Gl, 
              view_matrix: &na::Matrix4<f32>,
              proj_matrix: &na::Matrix4<f32>,
              camera_pos: &na::Point3<f32>);
    fn update_physics_from_self(&self, _physics: &mut Physics) -> Result<(), Error> {Ok(())}
    fn update_self_from_physics(&mut self, _physics: &Physics) -> Result<(), Error> {Ok(())}
}

pub struct Map {
    position: na::Isometry3<f32>,
    pub rb_handle: np::object::DefaultBodyHandle,
    pub model: Model,
}

impl Map {
    pub fn load_map(model: Model, physics: &mut Physics) -> Map {
        let pos = na::Isometry3::from_parts(
            na::Translation::from(na::Point3::origin().coords),
            na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0));
        let map_shape = nc::shape::ShapeHandle::new(model.get_trimesh());
        let map_rb_desc = np::object::RigidBodyDesc::new().status(np::object::BodyStatus::Static);
        let map_handle = physics.add_rigid_body(map_rb_desc.build());
        let map_collider = np::object::ColliderDesc::new(map_shape)
            .build(np::object::BodyPartHandle(map_handle.clone(), 0));
        physics.add_collider(map_collider);
        physics.get_rigid_body_mut(map_handle).unwrap().set_user_data(Some(Box::new(10)));
        Map {
            position: pos,
            rb_handle: map_handle,
            model: model,
        }
    }
}           

impl Entity for Map {
    fn render(&self,gl: &gl::Gl,
                  view_matrix: &na::Matrix4<f32>,
                  proj_matrix: &na::Matrix4<f32>,
                  camera_pos: &na::Point3<f32>,) {
        self.model.render(gl, view_matrix, proj_matrix, camera_pos, &self.position);
    }
}

pub struct PhysicsEntity {
    pub position: na::Isometry3<f32>,
    pub velocity: np::math::Velocity<f32>,
    pub rb_handle: np::object::DefaultBodyHandle,
    pub model: Model,
}

impl PhysicsEntity {
    //TODO: Make a builder that returns PhysicsEntityDef
    pub fn load_entity(model: Model, physics: &mut Physics, pos: na::Isometry3<f32>, shape: nc::shape::ShapeHandle<f32>) -> Self {
        let ent_rb_desc = np::object::RigidBodyDesc::new().position(pos).mass(0.2);
        let ent_rb_handle = physics.add_rigid_body(ent_rb_desc.build());
        let ent_collider = np::object::ColliderDesc::new(shape)
            .density(1.0)
            .build(np::object::BodyPartHandle(ent_rb_handle.clone(), 0));
        physics.add_collider(ent_collider);
        PhysicsEntity {
            position: pos,
            velocity: np::math::Velocity::zero(),
            rb_handle: ent_rb_handle,
            model: model,
        }
    }
}

impl Entity for PhysicsEntity {
    fn render(&self,gl: &gl::Gl,
                  view_matrix: &na::Matrix4<f32>,
                  proj_matrix: &na::Matrix4<f32>,
                  camera_pos: &na::Point3<f32>,) {
        self.model.render(gl, view_matrix, proj_matrix, camera_pos, &self.position);
    }

    fn update_self_from_physics(&mut self, physics: &Physics) -> Result<(), Error> {
        let rigidbody = physics.get_rigid_body(self.rb_handle)?;
        self.position = *rigidbody.position();
        self.velocity = *rigidbody.velocity();
        Ok(())
    }
    fn update_physics_from_self(&self, physics: &mut Physics) -> Result<(), Error> {
        let rigidbody = physics.get_rigid_body_mut(self.rb_handle)?;
        rigidbody.set_velocity(self.velocity);
        rigidbody.set_position(self.position);
        Ok(())
    }
}

pub struct DebugArrow {
    pub position: na::Isometry3<f32>,
    pub model: Model,
}

impl DebugArrow {
    pub fn new(model: Model, pos: na::Isometry3<f32>) -> Self {
        DebugArrow {
            position: pos,
            model: model,
        }
    }
}

impl Entity for DebugArrow {
    fn render(&self,gl: &gl::Gl,
                  view_matrix: &na::Matrix4<f32>,
                  proj_matrix: &na::Matrix4<f32>,
                  camera_pos: &na::Point3<f32>,) {
        self.model.render(gl, view_matrix, proj_matrix, camera_pos, &self.position);
    }
}

