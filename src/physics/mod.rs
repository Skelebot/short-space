mod velocity;
pub use velocity::Velocity;
use std::ops::{Deref, DerefMut};

pub struct Collider {
    inner: nc::shape::ShapeHandle<f32>
}
impl Deref for Collider {
    type Target = dyn nc::shape::Shape<f32>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl From<nc::shape::ShapeHandle<f32>> for Collider {
    fn from(shape: nc::shape::ShapeHandle<f32>) -> Self {
        Self { inner: shape }
    }
}

pub struct Scale {
    inner: na::Vector3<f32>
}

impl From<na::Vector3<f32>> for Scale {
    fn from(vec: na::Vector3<f32>) -> Self {
        Self { inner: vec }
    }
}

impl Deref for Scale {
    type Target = na::Vector3<f32>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A wrapper for nalgebra's Isometry to be used as a component for physical entities
pub struct Position {
    inner: na::Isometry3<f32>,
}

impl From<na::Isometry3<f32>> for Position {
    fn from(iso: na::Isometry3<f32>) -> Self {
        Self { inner: iso }
    }
}

impl Deref for Position {
    type Target = na::Isometry3<f32>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct PhysicsSettings {
    pub gravity: na::Vector3<f32>,
    pub air_friction: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        PhysicsSettings {
            gravity: na::Vector3::new(0.0, -0.01, 0.0),
            air_friction: 0.01,
        }
    }
}