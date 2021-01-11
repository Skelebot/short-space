use crate::{
    graphics::{color, mesh_pass::Vertex},
    spacetime,
};

pub struct MeshData {
    pub parts: Vec<MeshPartData>,
}

pub struct MeshPartData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: MaterialData,
}

pub struct MaterialData {
    pub specular_coefficient: f32,
    pub color_ambient: color::Rgb,
    pub color_diffuse: color::Rgb,
    pub color_specular: color::Rgb,
    pub color_emissive: color::Rgb,
    pub alpha: f32,
    pub lighting: bool,
    // TODO: Add all the other maps
    pub diffuse_map: Option<image::RgbaImage>,
}

impl Default for MaterialData {
    fn default() -> Self {
        MaterialData {
            specular_coefficient: 1.0,
            color_ambient: color::Rgb::new(1.0, 0.0, 1.0),
            color_diffuse: color::Rgb::new(1.0, 0.0, 1.0),
            color_specular: color::Rgb::new(0.0, 0.0, 0.0),
            color_emissive: color::Rgb::new(0.0, 0.0, 0.0),
            lighting: true,
            alpha: 1.0,
            diffuse_map: None,
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Scale {
    All(f32),
    Xyz(f32, f32, f32),
}

impl Into<spacetime::Scale> for Scale {
    fn into(self) -> spacetime::Scale {
        match self {
            Scale::All(n) => na::Vector3::repeat(n),
            Scale::Xyz(x, y, z) => na::Vector3::new(x, y, z),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(crate) enum Rotation {
    /// Roll, Pitch, Yaw
    Euler(f32, f32, f32),
    /// Axis [XYZ], angle
    Axis(Axis, f32),
}

impl Into<na::UnitQuaternion<f32>> for Rotation {
    fn into(self) -> na::UnitQuaternion<f32> {
        match self {
            Rotation::Euler(roll, pitch, yaw) => na::UnitQuaternion::from_euler_angles(
                roll.to_radians(),
                pitch.to_radians(),
                yaw.to_radians(),
            ),
            Rotation::Axis(axis, angle) => na::UnitQuaternion::from_axis_angle(
                &match axis {
                    Axis::X => na::Vector::x_axis(),
                    Axis::Y => na::Vector::y_axis(),
                    Axis::Z => na::Vector::z_axis(),
                },
                angle.to_radians(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(crate) struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation: Option<Rotation>,
}

impl Into<na::Isometry3<f32>> for Position {
    fn into(self) -> na::Isometry3<f32> {
        let translation = na::Translation3::new(self.x, self.y, self.z);
        let rotation = match self.rotation {
            Some(r) => r.into(),
            None => na::UnitQuaternion::identity(),
        };
        na::Isometry3::from_parts(translation, rotation)
    }
}

impl Into<spacetime::Position> for Position {
    fn into(self) -> spacetime::Position {
        let i: na::Isometry3<f32> = self.into();
        i.into()
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Model {
    pub pos: Position,
    pub scale: Option<Scale>,
    pub obj: String,
    pub parent: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Scene {
    pub objects: Vec<Model>,
}
