extern crate image;

use std::path::{Path, PathBuf};
use anyhow::{Result, Error, anyhow, format_err};
use crate::wgpu_graphics::mesh::{Vertex, ModelData};
use wgpu::util::DeviceExt;

use wavefront_obj as wobj;

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

    /// Load data of a single object from a wavefront OBJ file, discarding objects other than the first one
    /// (if there are multiple objects in that file). Expects the file to contain a material library with
    /// at least one material. Assumes the first material in the material library is assigned to the first
    /// model. Expects the first material in the material library to have a texture. Joins all shapes
    /// in the model into a single model. Discards all other data.
    pub fn load_simple_model(&self, path: impl AsRef<Path>) -> Result<ModelData> {
        let obj_file = std::fs::read_to_string(self.root_path.join(&path))
            .map_err(|err| anyhow!(err)
                .context(format_err!("Model not found: {:?}", self.root_path.join(&path))))?;

        let obj_path = self.root_path.join(&path);
        let obj_parent = obj_path.parent().ok_or(
            format_err!("Could not find parent directory of obj file: {:?}", 
                self.root_path.join(&path)))?;

        info!("path: {:?}", &self.root_path.join(&path).to_str().unwrap());
        // A set of objects; a single wavefront OBJ file can contain multiple objects
        let obj_set = wobj::obj::parse(&obj_file.as_str()).map_err(
            |err| anyhow!(err).context(format_err!(
                    "Error while parsing object set from file: {:?}",
                    self.root_path.join(&path)
                ))
        )?;
        // The set of materials for objects; if None, the objects do not have materials
        let mtl_set = if let Some(mtl_lib_path) = obj_set.material_library {
            let mtl_file = std::fs::read_to_string(self.root_path.join(obj_parent).join(&mtl_lib_path))?;
            wobj::mtl::parse(
                &mtl_file.as_str()
            ).map_err(|err| anyhow!(err).context(format_err!(
                "Error while parsing material set from file: {:?}",
                self.root_path.join(obj_parent).join(mtl_lib_path)
            )))
        } else {
            Err(format_err!(
                "Expected the model: {:?} to have at least one material",
                self.root_path.join(&path)
            ))
        }?;
        if obj_set.objects.len() == 0 { return Err(
            format_err!("No objects found in OBJ file {:?}", 
                self.root_path.join(&path)
            ))
        };
        if mtl_set.materials.len() == 0 { return Err(
            format_err!("No materials found in material set in OBJ file {:?}", 
                self.root_path.join(&path)
            ))
        };

        let object = &obj_set.objects[0];
        info!("Loading model: {}", object.name);

        let material = &mtl_set.materials[0];
        info!("Loading material: {}", material.name);

        let vertices: Vec<[f32; 3]> = object.vertices.iter().map(|v| [v.x as f32, v.y as f32, v.z as f32]).collect();
        let uvs: Vec<[f32; 2]> = object.tex_vertices.iter().map(|uv| [uv.u as f32, uv.v as f32]).collect();
        let normals: Vec<[f32; 3]> = object.normals.iter().map(|n| [n.x as f32, n.y as f32, n.z as f32]).collect();

        let mut model_vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u16>::new();

        for geom in object.geometry.iter() {
            for shape in geom.shapes.iter() {
                use wobj::obj::{Primitive, VTNIndex};
                let mut v_tmp = Vec::<VTNIndex>::new();
                match shape.primitive {
                    Primitive::Point(vtni) => v_tmp.push(vtni),
                    Primitive::Line(vtni0, vtni1) => v_tmp.extend(&[vtni0, vtni1]),
                    Primitive::Triangle(vtni0, vtni1, vtni2) => v_tmp.extend(&[vtni0, vtni1, vtni2]),
                }
                for vtni in v_tmp {
                    indices.push(vtni.0 as u16);
                    if let (Some(uvi), Some(ni)) = (vtni.1, vtni.2) {
                        model_vertices.push(Vertex {
                            pos: vertices[vtni.0],
                            uv: uvs[uvi],
                            normal: normals[ni],
                        })
                    }
                }
            }
        }

        // Remove duplicate Vertexes if their drawing index is the same
        //model_vertices = {
        //    let mut tmp = model_vertices.into_iter().enumerate().map(|(i, v)| (indices[i], v)).collect::<Vec<(u16, Vertex)>>();
        //    tmp.sort_by_key(|(i, _)| *i);
        //    tmp.dedup_by_key(|(i, _)| *i);
        //    tmp.into_iter().map(|(_, v)| v).collect()
        //};

        info!("Lengths: \n vertices: {},\n uvs: {},\n normals: {},\n indices: {},\n model_vertices: {}",
            vertices.len(),
            uvs.len(),
            normals.len(),
            indices.len(),
            model_vertices.len(),
        );

        let texture_path = material.uv_map.as_ref().ok_or(
            format_err!("Expected the material {:?} to have a texture (uv_map)", material.name))?;

        let img = self.load_texture(obj_parent.join(texture_path))?;

        return Ok(ModelData {
            vertices: model_vertices,
            indices: indices,
            texture_img: img,
        })
    }

    pub fn load_texture(&self, path: impl AsRef<Path>) -> Result<image::RgbaImage> {
        Ok(
            match image::open(self.root_path.join(&path))
                .map_err(|err| 
                    anyhow!(err).context(format_err!("Failed to open image: {:?}", path.as_ref())))? 
            {
                image::DynamicImage::ImageRgba8(img) => img,
                img @ _ => img.to_rgba(),
            }
        )
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
            height: img_width,
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
                    rows_per_image: img_width,
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
}