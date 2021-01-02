use crate::graphics::{color, mesh_pass::Vertex};

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
