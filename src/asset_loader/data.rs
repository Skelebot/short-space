use crate::graphics::mesh::Vertex;
pub struct MeshData {
    pub parts: Vec<MeshPartData>,
}

pub struct MeshPartData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub material: MaterialData,
}

pub struct MaterialData {
    pub specular_coefficient: f32,
    pub color_ambient: [f32; 3],
    pub color_diffuse: [f32; 3],
    pub color_specular: [f32; 3],
    pub color_emissive: [f32; 3],
    pub alpha: f32,
    pub lighting: bool,
    // TODO: Add all the other maps
    pub diffuse_map: Option<image::RgbaImage>,
}

impl Default for MaterialData {
    fn default() -> Self {
        MaterialData {
            specular_coefficient: 1.0,
            color_ambient: [1.0, 0.0, 1.0],
            color_diffuse: [0.0, 0.0, 0.0],
            color_specular: [0.0, 0.0, 0.0],
            color_emissive: [0.0, 0.0, 0.0],
            lighting: true,
            alpha: 1.0,
            diffuse_map: None,
        }
    }
}