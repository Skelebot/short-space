extern crate image;

pub mod data;

use std::path::{Path, PathBuf};
use anyhow::{Result, Error, anyhow, format_err};
use wgpu::util::DeviceExt;

use crate::graphics::mesh::Vertex;

use wavefront_obj as wobj;
use wobj::{mtl::{Material, MtlSet}, obj::{Primitive, VTNIndex}};

use self::data::MeshPart;
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

    pub fn upload_texture(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        srgb: bool,
        img: image::RgbaImage,
    ) -> wgpu::Texture {
        // The physical size of the texture
        let (img_width, img_height) = (img.width(), img.height());
        let texture_extent = wgpu::Extent3d {
            width: img_width,
            height: img_height,
            depth: 1,
        };
        // The texture binding to copy data to and use as a handle to it
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm },
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        // Temporary buffer to copy data from into the texture
        let tmp_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&img.into_raw()),
                usage: wgpu::BufferUsage::COPY_SRC,
            }
        );
        // Copy img's pixels from the temporary buffer into the texture buffer
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &tmp_buf,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 4 * img_width,
                    rows_per_image: img_height,
                },
            }, 
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            }, 
            texture_extent
        );
        // Return the texture handle
        texture
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

    pub fn load_model(&self, path: impl AsRef<Path>) -> Result<data::ModelData> {
        let obj_path = self.root_path.join(&path);
        let obj_parent = obj_path.parent().unwrap();
        let obj_file = std::fs::read_to_string(&obj_path)
            .map_err(|err| anyhow!(err)
                .context(format_err!("Model not found: {:?}", &obj_path)))?;

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

        for object in object_set.objects {
            debug!("Loading model: {}", object.name);
            let vertices = object.vertices;
            let tex_vertices = object.tex_vertices;

            let mesh_parts: Vec<MeshPart> = Vec::with_capacity(object.geometry.len());
            for geometry in object.geometry {
                let material: Option<&Material> = match geometry.material_name {
                    Some(name) => material_set.materials.iter().find(|m| m.name == name),
                    None => None,
                };

                let mut indices: Vec<u16> = Vec::new();
                let mut vertices: Vec<Vertex> = Vec::new();

                for shape in geometry.shapes {
                    match shape.primitive {
                        Primitive::Point(_) => {}
                        Primitive::Line(_, _) => {}
                        Primitive::Triangle(_, _, _) => {}
                    }
                }
            }
        }


        let material = &mtl_set.materials[0];
        debug!("Loading material: {}", material.name);

        let mut indices: Vec<u16> = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();

        let mut add_vertex = |vtni: &VTNIndex| {
            if let (Some(uvi), Some(ni)) = (vtni.1, vtni.2) {
                let vertex = Vertex {
                    pos: match object.vertices[vtni.0] { wobj::obj::Vertex {x, y, z} => [x as f32, y as f32, z as f32]},
                    uv: match object.tex_vertices[uvi] { wobj::obj::TVertex {u, v, ..} => [u as f32, v as f32]},
                    normal: match object.normals[ni] { wobj::obj::Normal {x, y, z} => [x as f32, y as f32, z as f32]},
                };
                if let Some((i, _)) = vertices.iter().enumerate().find(|(_, v)| v == &&vertex) {
                    indices.push(i as u16);
                } else {
                    // Push a new index
                    indices.push(vertices.len() as u16);
                    vertices.push(vertex);
                }
            }
        };
        for geom in object.geometry.iter() {
            for shape in geom.shapes.iter() {
                match shape.primitive {
                    Primitive::Point(vtni) => add_vertex(&vtni),
                    Primitive::Line(vtni0, vtni1) => {
                        add_vertex(&vtni0);
                        add_vertex(&vtni1);
                    },
                    Primitive::Triangle(vtni0, vtni1, vtni2) => {
                        add_vertex(&vtni0);
                        add_vertex(&vtni1);
                        add_vertex(&vtni2);
                    }
                }
            }
        }

        let texture_path = material.uv_map.as_ref().ok_or(
            format_err!("Expected the material {:?} to have a texture (uv_map)", material.name))?;

        let img = self.load_texture(obj_parent.join(texture_path))?
            // (?): fixes incorrect texture coords when loading obj models
            .flipv();

        // Convert the image to Rgba
        let texture_img = match img {
            image::DynamicImage::ImageRgba8(img) => img,
            img => img.to_rgba()
        };

        return Ok(data::ModelData {
            vertices,
            indices,
            texture_img,
        })
    }
}