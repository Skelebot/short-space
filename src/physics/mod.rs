extern crate nalgebra as na;
extern crate nphysics3d as np;
extern crate ncollide3d as nc;

use self::np::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};
use self::np::object::{DefaultBodySet, DefaultColliderSet, DefaultBodyHandle, DefaultColliderHandle};
use self::np::joint::DefaultJointConstraintSet;
use self::np::force_generator::DefaultForceGeneratorSet;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid rigidbody handle: {:?}", rb_handle)]
    InvalidRBHandle { rb_handle: DefaultBodyHandle },
    #[fail(display = "Invalid collider handle: {:?}", cl_handle)]
    InvalidCLHandle { cl_handle: DefaultColliderHandle },
}

pub struct Physics {
    pub mech_world: DefaultMechanicalWorld<f32>,
    pub geom_world: DefaultGeometricalWorld<f32>,
    pub bodies: DefaultBodySet<f32>,
    pub colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
}

impl Physics {
    pub fn new(gravity: f32) -> Self {
        Physics {
            mech_world: DefaultMechanicalWorld::new(na::Vector3::new(0.0, gravity, 0.0)),
            geom_world: DefaultGeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }

    pub fn add_ground(&mut self) -> DefaultBodyHandle {
        self.bodies.insert(np::object::Ground::new())
    }

    pub fn add_rigid_body(&mut self, rigidbody: np::object::RigidBody<f32>) -> DefaultBodyHandle {
        self.bodies.insert(rigidbody)
    }

    pub fn add_collider(&mut self, collider: np::object::Collider<f32, DefaultBodyHandle>) -> DefaultColliderHandle {
        self.colliders.insert(collider)
    }

    pub fn get_body(&self, handle: DefaultBodyHandle) -> Result<&dyn np::object::Body<f32>, Error> {
        match self.bodies.get(handle) {
            Some(body) => Ok(body),
            None => Err(Error::InvalidRBHandle { rb_handle: handle })
        }
    }

    pub fn get_rigid_body(&self, handle: DefaultBodyHandle) -> Result<&np::object::RigidBody<f32>, Error> {
        match self.bodies.rigid_body(handle) {
            Some(rigid_body) => Ok(rigid_body),
            None => Err(Error::InvalidRBHandle { rb_handle: handle })
        }
    }

    pub fn get_rigid_body_mut(&mut self, handle: DefaultBodyHandle) -> Result<&mut np::object::RigidBody<f32>, Error> {
        match self.bodies.rigid_body_mut(handle) {
            Some(rigid_body) => Ok(rigid_body),
            None => Err(Error::InvalidRBHandle { rb_handle: handle })
        }
    }

    pub fn get_collider(&self, handle: DefaultColliderHandle) -> Result<&np::object::Collider<f32, DefaultBodyHandle>, Error> {
        match self.colliders.get(handle) {
            Some(collider) => Ok(collider),
            None => Err(Error::InvalidCLHandle { cl_handle: handle })
        }
    }

    pub fn get_collider_mut(&mut self, handle: DefaultColliderHandle) -> Result<&mut np::object::Collider<f32, DefaultBodyHandle>, Error> {
        match self.colliders.get_mut(handle) {
            Some(collider) => Ok(collider),
            None => Err(Error::InvalidCLHandle { cl_handle: handle })
        }
    }

    pub fn step(&mut self) {
        self.mech_world.step(
            &mut self.geom_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators
        );
    }
}
