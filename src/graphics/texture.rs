use gl;
use anyhow::Result;
use std::os::raw;
use crate::asset_loader::AssetLoader;

pub struct Texture {
    pub id: gl::types::GLuint,
    pub width: i32,
    pub height: i32,
}

impl Texture {
    pub fn from_res(gl: &gl::Gl, res: &AssetLoader, name: &str) -> Result<Texture> {
        let data = res.load_rgb_image(name)?;
        println!("name: {}, pixel: {:?}", name, data.get_pixel(0, 0));

        Ok(Texture::from_data(gl, &data, data.width() as i32, data.height() as i32))
    }

    pub fn from_data(gl: &gl::Gl, data: &image::RgbImage, width: i32, height: i32) -> Texture {
        let mut texture = 0;
        unsafe {
            gl.GenTextures(1, &mut texture);
            gl.BindTexture(gl::TEXTURE_2D, texture);
            //TODO: glTexParametri
            gl.TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width, height, 0, gl::RGB, gl::UNSIGNED_BYTE,
                          data.as_ptr() as *const raw::c_void);
            gl.GenerateMipmap(gl::TEXTURE_2D);
        }
        Texture {
            id: texture,
            width: width,
            height: height
        }
    }
    
    ///Set a texture unit as active before binding a texture
    ///There is a minimum of 16 texture units, defined in order
    ///from GL_TEXTURE0 to GL_TEXTURE15
    pub fn activate_texture_unit(gl: &gl::Gl, texture_unit: u32) {
        unsafe {
            gl.ActiveTexture(gl::TEXTURE0 + texture_unit);
        }
    }

    pub fn bind(&self, gl: &gl::Gl) {
        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn unbind(&self, gl: &gl::Gl) {
        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn bind_at(&self, gl: &gl::Gl, index: u32) {
        Self::activate_texture_unit(&gl, index);
        self.bind(&gl);
    }

    pub unsafe fn destroy(&mut self, gl: &gl::Gl) {
        gl.DeleteTextures(1, &self.id);
    }
}