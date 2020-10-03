use gl;
use anyhow::Result;
use crate::asset_loader::AssetLoader;
use crate::graphics::{
    texture::Texture,
    shader::Program,
    data, buffer
};

//Position from bottom-left corner
#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
struct Glyphtex {
    #[location = 0]
    pos: data::f32_f32,
    #[location = 1]
    uv: data::f32_f32,
}

struct Glyph {
    pub vao: buffer::VertexArray,
    pub vbo: buffer::ArrayBuffer,
    pub ebo: buffer::ElementArrayBuffer,
    pub index_count: i32,
}

impl Glyph {
    //Positions from bottom left, UV of top left corner
    //TODO: position coordinates in screen pixels, converting the screen pixels to <-1.0; 1.0> with
    //a translation matrix, a function for updating the size using window size, fix the uvs
    pub fn new(gl: &gl::Gl, x_uv: u32, y_uv: u32, w_width: f32, w_height: f32, width: u32, height: u32) -> Glyph {
        let pos_width: f32 = ((width) as f32/w_width as f32) * -1.0;
        let pos_height: f32 = ((height as f32)/w_height as f32) * -1.0;
        let positions: [(f32, f32); 4] = [
            (-1.0, -1.0),               //Bottom left
            (-1.0, -1.0 - pos_height),  //Top left
            (-1.0 - pos_width, -1.0 - pos_height),  //Top right
            (-1.0 - pos_width, -1.0),   //Bottom right
        ];
        let x_uv_f = x_uv as f32;
        let y_uv_f = y_uv as f32;
        let w_f = width as f32;
        let h_f = height as f32;
        let uvs: [(f32, f32); 4] = [
            (x_uv_f/w_width, (y_uv_f + h_f)/w_height),     //Bottom left
            (x_uv_f/w_width, y_uv_f/w_height),           //Top left
            ((x_uv_f + w_f)/w_width, y_uv_f/w_height),     //Top right
            ((x_uv_f + w_f)/w_width, (y_uv_f + h_f)/w_height)//Bottom right
        ];

        let indices: [u32; 6] = [0, 2, 1, 0, 3, 2];
        
        let mut vbo_data: Vec<Glyphtex> = Vec::with_capacity(4);
        for i in 0..4 {
            vbo_data.push(
                Glyphtex {
                    pos: positions[i].into(),
                    uv: uvs[i].into(),
                }
            )
        }
        //DEBUG
        println!("positions: {:?}\n uvs: {:?}", positions, uvs);

        let vbo = buffer::ArrayBuffer::new(gl);
        vbo.bind(&gl);
        vbo.static_draw_data(&gl, &vbo_data);
        vbo.unbind(&gl);

        let ebo = buffer::ElementArrayBuffer::new(gl);
        ebo.bind(&gl);
        ebo.static_draw_data(&gl, &indices.to_vec());
        ebo.unbind(&gl);

        let vao = buffer::VertexArray::new(gl);

        vao.bind(&gl);
        vbo.bind(&gl);
        ebo.bind(&gl);
        Glyphtex::vertex_attrib_pointers(gl);
        vao.unbind(&gl);
        vbo.unbind(&gl);
        ebo.unbind(&gl);

        Glyph {
            vao: vao,
            vbo: vbo,
            ebo: ebo,
            index_count: 6,
        }
    }
}

pub struct BitmapFont {
    options: BitmapFontOptions,
    program: Program,
    glyphs: Vec<Glyph>,
    texture: Texture,
    size_x: f32,
    size_y: f32,
}

impl BitmapFont {
    pub fn new(gl: &gl::Gl, res: &AssetLoader, options: BitmapFontOptions, win_width: u32, win_height: u32) -> Result<BitmapFont> {
        let program = Program::from_res(gl, res, options.shader_path)?;
        let image = res.load_rgb_image(options.image_path)?;

        //TODO: Make it shorter or make it a function
        //FIXME: Fix the error message
        let num_glyphs_x: u32 = image.width()/(options.glyph_width+options.border);
        //DEBUG
        //if ((options.glyph_width + options.border)*num_glyphs_x) + options.border != image.width() {
        //    return Err(failure::format_err!(
        //            "Image width does not match glyph width with border size: (({0} + {1}) * {2}) + {1}!= {3}",
        //            options.glyph_width,
        //            options.border,
        //            num_glyphs_x, image.width()));
        //}
        let num_glyphs_y: u32 = image.height()/(options.glyph_height + options.border);
        //DEBUG
        //if ((options.glyph_height + options.border)*num_glyphs_y) + options.border != image.height() {
        //    return Err(failure::format_err!(
        //            "Image height does not match glyph height with border size: FIXME: {0} / {1} + {2} != {0}",
        //            image.height(),
        //            options.glyph_width,
        //            options.border));
        //}
        let total_num_glyphs = num_glyphs_x * num_glyphs_y;
        let size_x: f32 = ((options.glyph_width) as f32/win_width as f32) * -1.0;
        let size_y: f32 = ((options.glyph_height as f32)/win_height as f32) * -1.0;
        println!("glyph_size: {:?}, {}", (options.glyph_width, win_width), options.glyph_width/win_width);
        let mut glyphs: Vec<Glyph> = Vec::with_capacity(total_num_glyphs as usize);
        for i in 1..total_num_glyphs {
            let row: u32 = (i/num_glyphs_x)+1;
            let mut column: u32 = i%num_glyphs_x;
            if column == 0 { column = 4; }
            let x_uv = (options.border * column) + (options.glyph_width * (column - 1));
            let y_uv = (options.border * row) + (options.glyph_height * (row - 1));
            glyphs.push(
                Glyph::new(gl, x_uv, y_uv, win_width as f32, win_height as f32,
                           options.glyph_width, options.glyph_height)
            );
        }
        let tex = Texture::from_res(gl, res, options.image_path)?;
        Ok(
            BitmapFont {
                options: options,
                program: program,
                glyphs,
                texture: tex,
                size_x,
                size_y,
            }
        )
    }

    ///Makes sure the font is pixel-perfect
    ///Call every time the window is resized
    pub fn update_size(&mut self, win_width: u32, win_height: u32) {
        self.size_x = ((self.options.glyph_width)/win_width) as f32 * -1.0;
        self.size_y = ((self.options.glyph_height)/win_height) as f32 * -1.0;
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.program.set_used(&gl);
        self.texture.bind_at(&gl, 0);
        let tex_loc = self.program.get_uniform_location(&gl, "TexFace").expect("Uniform location not found");
        self.program.set_uniform_1i(&gl, tex_loc, 0);
        self.glyphs[4].vao.bind(&gl);
        unsafe {
            gl.DrawElements(
                gl::TRIANGLES,
                self.glyphs[4].index_count,
                gl::UNSIGNED_INT,
                std::ptr::null()
            );
        }
        
    }
}

pub struct BitmapFontOptions {
    pub glyph_width: u32,
    pub glyph_height: u32,
    pub border: u32,
    pub image_path: &'static str,
    pub shader_path: &'static str,
}

impl BitmapFontOptions {
    pub fn new(g_width: u32, g_height: u32, border: u32, img_path: &'static str, shader_path: &'static str) -> Self {
        BitmapFontOptions {
            glyph_width: g_width,
            glyph_height: g_height,
            border: border,
            image_path: img_path,
            shader_path: shader_path,
        }
    }
}

