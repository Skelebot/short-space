extern crate ncollide3d as nc;
use anyhow::Result;
use gl;
use nalgebra as na;
use crate::graphics::{buffer, data, shader, texture};
use crate::asset_loader::AssetLoader;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
    #[location = 1]
    normal: data::f32_f32_f32,
    #[location = 2]
    uv: data::f32_f32,
}

pub struct Model {
    shader_program: shader::Program,
    texture: Option<texture::Texture>,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    camera_pos_location: Option<i32>,
    tex_face_location: Option<i32>,
    _vbo: buffer::ArrayBuffer,
    _ebo: buffer::ElementArrayBuffer,
    index_count: i32,
    vao: buffer::VertexArray,
    pub mesh: tobj::Mesh,
}

impl Model {
    pub fn new(
        res: &AssetLoader,
        gl: &gl::Gl,
        model_path: &str,
        shader_program: &shader::Program,
        debug: bool
    ) -> Result<Model> {
        let shader_program = shader_program.clone();
        //set up shader program
        let program_view_location = shader_program.get_uniform_location(&gl, "View");
        let program_projection_location = shader_program.get_uniform_location(&gl, "Projection");
        let camera_pos_location = shader_program.get_uniform_location(&gl, "CameraPos");
        let tex_face_location = shader_program.get_uniform_location(&gl, "TexFace");

        let imported_models = res.load_obj(model_path, debug)?;

        //take first material in obj
        let material = imported_models.materials.into_iter().next();
        let material_index = material.as_ref().map(|_| 0); //it is first or None

        let texture = match material {
            Some(material) => if &material.diffuse_texture == "" {
                None
            } else {
                Some(
                    texture::Texture::from_res(
                        gl, res,
                        &[
                        &imported_models.imported_from_resource_path[..],
                        "/",
                        &material.diffuse_texture[..],
                        ].concat(),
                    )?,
                )
            },
            None => None,
        };

        let mesh = imported_models
            .models.into_iter()
            .filter(|model|model.mesh.material_id == material_index)
            .next().expect("expected obj file to contain a mesh")
            .mesh;

        let vbo_data = mesh
            .positions
            .chunks(3)
            .zip(mesh.normals.chunks(3))
            .zip(mesh.texcoords.chunks(2))
            .map(|((p, n), t)| Vertex {
                pos: (p[0], p[1], p[2]).into(),
                normal: (n[0], n[1], n[2]).into(),
                uv: (t[0], -t[1]).into(), })
            .collect::<Vec<_>>();
        

        let ebo_data = mesh.indices.clone();

        let vbo = buffer::ArrayBuffer::new(gl);
        vbo.bind(&gl);
        vbo.static_draw_data(&gl, &vbo_data);
        vbo.unbind(&gl);

        let ebo = buffer::ElementArrayBuffer::new(gl);
        ebo.bind(&gl);
        ebo.static_draw_data(&gl, &ebo_data);
        ebo.unbind(&gl);

        let vao = buffer::VertexArray::new(gl);

        vao.bind(&gl);
        vbo.bind(&gl);
        ebo.bind(&gl);
        Vertex::vertex_attrib_pointers(gl);
        vao.unbind(&gl);
        vbo.unbind(&gl);
        ebo.unbind(&gl);

        Ok(Model {
            texture,
            shader_program,
            program_view_location,
            program_projection_location,
            camera_pos_location,
            tex_face_location,
            _vbo: vbo,
            _ebo: ebo,
            index_count: ebo_data.len() as i32,
            vao,
            mesh
        })
    }

    pub fn render(
        &self,
        gl: &gl::Gl,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        camera_pos: &na::Point3<f32>,
        transformation: &na::Isometry3<f32>,
    ) {
        self.shader_program.set_used(&gl);
        //bind lights
        //TODO: add support for more than one light
        //let lpos = self.program.get_uniform_location("light.position");
        //let lcol = self.program.get_uniform_location("light.color");
        //let lstr = self.program.get_uniform_location("light.strength");
        //self.program.set_uniform_3f(lpos.unwrap(), &lights[0].position.coords);
        //self.program.set_uniform_3f(lcol.unwrap(), &lights[0].color);
        //self.program.set_uniform_1f(lstr.unwrap(), lights[0].strength);

        let mut texture_bind = None;
        if let (Some(loc), &Some(ref texture)) = (self.tex_face_location, &self.texture) {
            texture.bind_at(&gl, 0);
            self.shader_program.set_uniform_1i(&gl, loc, 0);
            texture_bind = Some(texture);
        }

        if let Some(loc) = self.program_view_location {
            self.shader_program.set_uniform_matrix_4fv(&gl, loc, view_matrix);
        }
        if let Some(loc) = self.program_projection_location {
            self.shader_program.set_uniform_matrix_4fv(&gl, loc, proj_matrix);
        }
        if let Some(loc) = self.camera_pos_location {
            self.shader_program.set_uniform_3f(&gl, loc, &camera_pos.coords);
        }
        let model_loc = self.shader_program.get_uniform_location(&gl, "Model").unwrap();
        self.shader_program.set_uniform_matrix_4fv(&gl, model_loc, &transformation.to_homogeneous());
        self.vao.bind(&gl);

        unsafe {
            gl.DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                std::ptr::null()
            );
        }
        // ensure that if the model does not have a texture, it won't take the last texture bound
        if let Some(tex) = texture_bind {
            tex.unbind(&gl);
        }
    }

    pub fn get_trimesh(&self) -> nc::shape::TriMesh<f32> {
        let points = self.mesh.positions.chunks(3)
            .map(|p| na::Point3::new(p[0], p[1], p[2]))
            .collect::<Vec<_>>();
        let indices = self.mesh.indices.chunks(3)
            .map(|p| na::Point3::new(p[0] as usize, p[1] as usize, p[2] as usize))
            .collect::<Vec<_>>();

        nc::shape::TriMesh::new(points, indices, None)
    }

    /// 
    pub unsafe fn destroy(&mut self, gl: &gl::Gl) {
        self.shader_program.destroy(&gl);
        if let Some(ref mut tex) = self.texture { tex.destroy(&gl) }

        self.vao.destroy(&gl);
        self._vbo.destroy(&gl);
    }
}

