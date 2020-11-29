use crate::graphics::mesh::Vertex;
pub struct ModelData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture_img: image::RgbaImage,
}

pub struct MeshPart {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub material: MeshMaterial,
}

// Re-export illumination
pub use wavefront_obj::mtl::Illumination;
pub struct MeshMaterial {
    specular_coefficient: f32,
    color_ambient: [f32; 3],
    color_diffuse: [f32; 3],
    color_specular: [f32; 3],
    color_emmisive: [f32; 3],
    alpha: f32,
    illumination: Illumination,
}

