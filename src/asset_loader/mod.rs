extern crate image;

pub mod data;

use std::path::{Path, PathBuf};
use anyhow::{Result, Error, anyhow, format_err};
use data::MaterialData;

use crate::graphics::mesh::Vertex;

use wavefront_obj as wobj;
use wobj::{mtl::{Material, MtlSet}, obj::{Primitive, VTNIndex}};

use self::data::MeshPartData;

pub struct AssetLoader {
    root_path: PathBuf,
}

impl AssetLoader {
    pub fn from_relative_exe_path(rel_path: &Path) -> Result<AssetLoader, Error> {
        let exe_file_name = std::env::current_exe()?;

        let exe_path = exe_file_name.parent()
            .ok_or(anyhow!("Could not find executable's parent directory"))?;

        Ok(AssetLoader {
            root_path: exe_path.join(rel_path),
        })
    }

    pub fn load_texture(&self, path: impl AsRef<Path>) -> Result<image::DynamicImage> {
        let img = image::open(self.root_path.join(&path))
            .map_err(|err| 
                anyhow!(err).context(format_err!("Failed to open image: {:?}", path.as_ref())))?;
        Ok(img)
    }

    fn load_map_img(&self, path: impl AsRef<Path>) -> Result<image::RgbaImage> {
        let img = self.load_texture(path)?
            // (?): fixes incorrect texture coords when loading obj models
            .flipv();

        // Convert the image to Rgba
        Ok(match img {
            image::DynamicImage::ImageRgba8(img) => img,
            img => img.to_rgba()
        })
    }

    pub fn load_bytes(&self, path: impl AsRef<Path>) -> Vec<u8> {
        std::fs::read(self.root_path.join(&path)).expect(&format!("Could not find shader: {:?}", self.root_path.join(&path)))
    }

    pub fn load_material_set(&self, path: impl AsRef<Path>) -> Result<MtlSet> {
        wobj::mtl::parse(
            &std::fs::read_to_string(
                self.root_path.join(&path)
            )?
        ).map_err(|err| 
            anyhow!(err)
                .context(
                    format_err!(
                        "Error while parsing material set from file: {:?}",
                        self.root_path.join(path)
        )))
    }

    pub fn load_obj_set(&self, path: impl AsRef<Path>) -> Result<Vec<data::MeshData>> {
        let obj_path = self.root_path.join(&path);
        let obj_parent = obj_path.parent().unwrap();
        let obj_file = std::fs::read_to_string(&obj_path)
            .map_err(|err| anyhow!(err)
                .context(format_err!("Mesh not found: {:?}", &obj_path)))?;

        // A set of objects; a single wavefront OBJ file can contain multiple objects
        let object_set = wobj::obj::parse(&obj_file.as_str()).map_err(
            |err| 
            anyhow!(err)
                .context(
                    format_err!(
                        "Error while parsing object set from file: {:?}", obj_path
                ))
        )?;

        // The set of materials for objects; if None, the objects do not have materials
        let material_set = if let Some(mtl_lib_path) = object_set.material_library {
            self.load_material_set(obj_parent.join(mtl_lib_path))
        } else {
            Err(format_err!(
                "Expected the model: {:?} to have at least one material",
                self.root_path.join(&path)
            ))
        }?;

        let mut objects = Vec::new();

        for object in &object_set.objects {
            debug!("Loading model: {}", object.name);
            //let vertices = object.vertices;
            //let normals = object.normals;
            //let tex_vertices = object.tex_vertices;

            let mut mesh_parts: Vec<MeshPartData> = Vec::with_capacity(object.geometry.len());
            // For every geometry in an object
            // TODO: Group geometries by material
            for geometry in object.geometry.iter() {
                // Create new mesh part
                let material: Option<&Material> = match &geometry.material_name {
                    Some(name) => material_set.materials.iter().find(|m| &m.name == name),
                    None => None,
                };

                let mat_data = material.map_or(
                    MaterialData::default(), 
                    |mat| 
                    MaterialData {
                        specular_coefficient: mat.specular_coefficient as f32,
                        // A quick hack to get things working
                        lighting: mat.specular_coefficient != 0.0,
                        color_ambient: [
                            mat.color_ambient.r as f32,
                            mat.color_ambient.g as f32,
                            mat.color_ambient.b as f32,
                        ],
                        color_diffuse: [
                            mat.color_diffuse.r as f32,
                            mat.color_diffuse.g as f32,
                            mat.color_diffuse.b as f32,
                        ],
                        color_specular: [
                            mat.color_specular.r as f32,
                            mat.color_specular.g as f32,
                            mat.color_specular.b as f32,
                        ],
                        color_emissive: 
                            mat.color_emissive.map_or(
                                [0.0, 0.0, 0.0],
                                |color| [
                                        color.r as f32,
                                        color.g as f32,
                                        color.b as f32,
                        ]),
                        alpha: mat.alpha as f32,
                        // TODO: Replace unwraps here with printing the error and returning None,
                        // allowing the game to run even if some textures couldn't load, but also
                        // leaving a trace that something went wrong
                        // ambient_map: mat.ambient_map.map(|path| self.load_map_img(&path).unwrap()),
                        diffuse_map: mat.diffuse_map.as_ref().map(|path| self.load_map_img(obj_parent.join(&path)).unwrap()),
                    }
                );

                // It may seem weird, but apparently wgpu requires model indices to be u16
                // If we create a index buffer with u32s it doesn't render correctly
                let mut mesh_indices: Vec<u16> = Vec::new();
                let mut mesh_vertices: Vec<Vertex> = Vec::new();

                // Inserts a wobj::obj::Vertex to the Vecs above
                let mut insert_vertex = |vtni: VTNIndex| {
                    if let (posi, Some(uvi), Some(ni)) = vtni {
                        let vertex = Vertex {
                            pos: match object.vertices[posi] { wobj::obj::Vertex {x, y, z} => [x as f32, y as f32, z as f32]},
                            uv: match object.tex_vertices[uvi] { wobj::obj::TVertex {u, v, ..} => [u as f32, v as f32]},
                            normal: match object.normals[ni] { wobj::obj::Normal {x, y, z} => [x as f32, y as f32, z as f32]},
                        };
                        if let Some((i, _)) = mesh_vertices.iter().enumerate().find(|(_, v)| v == &&vertex) {
                            mesh_indices.push(i as u16);
                        } else {
                          // Push a new index
                          mesh_indices.push(mesh_vertices.len() as u16);
                          mesh_vertices.push(vertex);
                        }
                    }
                };

                for shape in geometry.shapes.iter() {
                    match shape.primitive {
                            Primitive::Point(vtni) => insert_vertex(vtni),
                            Primitive::Line(vtni0, vtni1) => {
                                insert_vertex(vtni0);
                                insert_vertex(vtni1);
                            },
                            Primitive::Triangle(vtni0, vtni1, vtni2) => {
                                insert_vertex(vtni0);
                                insert_vertex(vtni1);
                                insert_vertex(vtni2);
                            }
                    }
                }
                mesh_parts.push(
                    MeshPartData {
                        vertices: mesh_vertices,
                        indices: mesh_indices,
                        material: mat_data,
                    }
                )
            }
            objects.push(data::MeshData {
                parts: mesh_parts
            });
        }
        Ok(objects)
    }
}