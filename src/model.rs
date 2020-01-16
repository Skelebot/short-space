use failure;
use gl;
use nalgebra as na;
use crate::render_gl::{self, buffer, data};
use crate::resources::Resources;
use crate::light::PointLight;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
    #[location = 2]
    normal: data::f32_f32_f32,
    #[location = 3]
    uv: data::f32_f32,
}

pub struct Model {
    program: render_gl::Program,
    texture: Option<render_gl::Texture>,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    camera_pos_location: Option<i32>,
    tex_face_location: Option<i32>,
    _vbo: buffer::ArrayBuffer,
    _ebo: buffer::ElementArrayBuffer,
    index_count: i32,
    vao: buffer::VertexArray
}

impl Model {
    pub fn new(
        res: &Resources,
        gl: &gl::Gl,
        model_path: &str,
        shader_path: &str,
        debug: bool
    ) -> Result<Model, failure::Error> {
        //set up shader program
        let program = render_gl::Program::from_res(gl, res, shader_path)?;
        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let camera_pos_location = program.get_uniform_location("CameraPos");
        let tex_face_location = program.get_uniform_location("TexFace");

        let imported_models = res.load_obj(model_path, debug)?;

        //take first material in obj
        let material = imported_models.materials.into_iter().next();
        let material_index = material.as_ref().map(|_| 0); //it is first or None

        let texture = match material {
            Some(material) => if &material.diffuse_texture == "" {
                None
            } else {
                Some(
                    render_gl::Texture::from_res(
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
                uv: (t[0], -t[1]).into(), }).collect::<Vec<_>>();

        let ebo_data = mesh.indices;

        let vbo = buffer::ArrayBuffer::new(gl);
        vbo.bind();
        vbo.static_draw_data(&vbo_data);
        vbo.unbind();

        let ebo = buffer::ElementArrayBuffer::new(gl);
        ebo.bind();
        ebo.static_draw_data(&ebo_data);
        ebo.unbind();

        let vao = buffer::VertexArray::new(gl);

        vao.bind();
        vbo.bind();
        ebo.bind();
        Vertex::vertex_attrib_pointers(gl);
        vao.unbind();
        vbo.unbind();
        ebo.unbind();

        Ok(Model {
            texture,
            program,
            program_view_location,
            program_projection_location,
            camera_pos_location,
            tex_face_location,
            _vbo: vbo,
            _ebo: ebo,
            index_count: ebo_data.len() as i32,
            vao,
        })
    }

    pub fn render(
        &self,
        gl: &gl::Gl,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        camera_pos: &na::Point3<f32>,
        transformation: &na::Isometry3<f32>,
        lights: &Vec<PointLight>
    ) {
        self.program.set_used();
        //bind lights
        //TODO: add support for more than one light
        let lpos = self.program.get_uniform_location("light.position");
        let lcol = self.program.get_uniform_location("light.color");
        let lstr = self.program.get_uniform_location("light.strength");
        self.program.set_uniform_3f(lpos.unwrap(), &lights[0].position.coords);
        self.program.set_uniform_3f(lcol.unwrap(), &lights[0].color);
        self.program.set_uniform_1f(lstr.unwrap(), lights[0].strength);

        let mut texture_bind = None;
        if let (Some(loc), &Some(ref texture)) = (self.tex_face_location, &self.texture) {
            texture.bind_at(0);
            self.program.set_uniform_1i(loc, 0);
            texture_bind = Some(texture);
        }

        if let Some(loc) = self.program_view_location {
            self.program.set_uniform_matrix_4fv(loc, view_matrix);
        }
        if let Some(loc) = self.program_projection_location {
            self.program.set_uniform_matrix_4fv(loc, proj_matrix);
        }
        if let Some(loc) = self.camera_pos_location {
            self.program.set_uniform_3f(loc, &camera_pos.coords);
        }
        let model_loc = self.program.get_uniform_location("Model").unwrap();
        self.program.set_uniform_matrix_4fv(model_loc, &transformation.to_homogeneous());
        self.vao.bind();

        unsafe {
            gl.DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                std::ptr::null()
            );
        }
        // ensure that if the model does not have a texture, it won't take the last texture bind
        if let Some(tex) = texture_bind {
            tex.unbind();
        }
    }
}

