use gl;
use failure;
use render_gl::{self, data, buffer};
use resources::Resources;
use nalgebra as na;

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
    #[location = 1]
    clr: data::u2_u10_u10_u10_rev_float,
    #[location = 2]
    tex_coord: data::f32_f32,
}

pub struct Triangle {
    program: render_gl::Program,
    _vbo: buffer::ArrayBuffer,  // _ to disable warning about not used vbo
    vao: buffer::VertexArray,
    texture: render_gl::Texture,
}

impl Triangle {
    pub fn new(res: &Resources, gl: &gl::Gl) -> Result<Triangle, failure::Error> {
        //set up shader program
        let program = render_gl::Program::from_res(gl, res, "shaders/triangle")?;
        //set up textures
        let texture = render_gl::Texture::from_res(&gl, &res, "textures/wall.jpg")?;
        
        //set up vbo
        let vertices: Vec<Vertex> = vec![
            Vertex {
                pos: (0.5, -0.5, 0.0).into(),
                clr: (1.0, 0.0, 0.0, 1.0).into(),
                tex_coord: (1.0, 0.0).into(),
            }, // bottom right
            Vertex {
                pos: (-0.5, -0.5, 0.0).into(),
                clr: (0.0, 1.0, 0.0, 1.0).into(),
                tex_coord: (0.0, 0.0).into(),
            }, // bottom left
            Vertex {
                pos: (0.0,  0.5, 0.0).into(),
                clr: (0.0, 0.0, 1.0, 1.0).into(),
                tex_coord: (0.5, 1.0).into(),
            }  // top
        ];
        let vbo = buffer::ArrayBuffer::new(gl);
        vbo.bind();
        vbo.static_draw_data(&vertices);
        vbo.unbind();

        //set up vao
        let vao = buffer::VertexArray::new(gl);

        vao.bind();
        vbo.bind();
        Vertex::vertex_attrib_pointers(gl);
        vbo.unbind();
        vao.unbind();

        Ok(Triangle {
            program,
            _vbo: vbo,
            vao,
            texture: texture,
        })
    }

    pub fn render(&self, gl: &gl::Gl, vp_matrix: &na::Matrix4<f32>) {
        self.program.set_used();
        let loc = self.program.get_uniform_location("mvp_matrix").unwrap();
        self.program.set_uniform_matrix_4fv(loc, vp_matrix);

        self.vao.bind();
        self.texture.bind();

        unsafe {
            gl.DrawArrays(
                gl::TRIANGLES, // mode
                0, // starting index in the enabled arrays
                3 // number of indices to be rendered
            );
        }
    }
}
