mod velocity;
pub use velocity::Velocity;

pub struct Collider {
    pub handle: nc::shape::ShapeHandle<f32>,
}

impl From<nc::shape::ShapeHandle<f32>> for Collider {
    fn from(shape: nc::shape::ShapeHandle<f32>) -> Self {
        Self { handle: shape }
    }
}

pub type Scale = na::Vector3<f32>;

/// A wrapper for nalgebra's Isometry to be used as a component for physical entities
pub type Position = na::Isometry3<f32>;

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
