use gl;
use crate::asset_loader::{self, AssetLoader};
use std::ffi::{CStr, CString};
use nalgebra as na;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to load resource {name}")]
    ResourceLoad {
        name: String,
        #[source]
        inner: asset_loader::Error,
    },
    #[error("Can not determine shader type for resource {name}")]
    CanNotDetermineShaderTypeForResource { name: String },
    #[error("Failed to compile shader {name}: {message}")]
    CompileError { name: String, message: String },
    #[error("Failed to link program {name}: {message}")]
    LinkError { name: String, message: String },
//    #[fail(display = "Failed to find uniform location: {}", name)]
//    UniformLocationNotFound { name: String },
}

#[derive(Copy, Clone)]
pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_res(gl: &gl::Gl, res: &AssetLoader, name: &str) -> Result<Program, Error> {
        const POSSIBLE_EXT: [&str; 2] = [".vert", ".frag"];

        let resource_names = POSSIBLE_EXT
            .iter()
            .map(|file_extension| format!("{}{}", name, file_extension))
            .collect::<Vec<String>>();

        let shaders = resource_names
            .iter()
            .map(|resource_name| Shader::from_res(gl, res, resource_name))
            .collect::<Result<Vec<Shader>, Error>>()?;

        Program::from_shaders(gl, &shaders[..]).map_err(|message| Error::LinkError {
            name: name.into(),
            message,
        })
    }

    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe {
                gl.AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl.LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl.DetachShader(program_id, shader.id());
            }
        }

        Ok(Program {
            id: program_id,
        })
    }
    #[allow(dead_code)]
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self, gl: &gl::Gl) {
        unsafe {
            gl.UseProgram(self.id);
        }
    }

    pub fn get_uniform_location(&self, gl: &gl::Gl, name: &str) -> Option<i32> {
        // TODO: Return Result<Option, Error> ?
        let cname = CString::new(name).expect("expected uniform name to have no nul bytes");
        
        let location: i32 = unsafe {
            gl.GetUniformLocation(self.id, cname.as_bytes_with_nul().as_ptr() as *const i8)
        };
        if location == -1 {
            return None;
        }
        Some(location)
    }

    // TODO: Generic functions
    #[allow(dead_code)]
    pub fn set_uniform_matrix_4fv(&self, gl: &gl::Gl, location: i32, value: &na::Matrix4<f32>) {
        unsafe {
            gl.UniformMatrix4fv(
                location,
                1,
                gl::FALSE,
                value.as_slice().as_ptr() as *const f32,
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_uniform_3f(&self, gl: &gl::Gl, location: i32, value: &na::Vector3<f32>) {
        unsafe {
            gl.Uniform3f(location, value.x, value.y, value.z);
        }
    }

    #[allow(dead_code)]
    pub fn set_uniform_4f(&self, gl: &gl::Gl, location: i32, value: &na::Vector4<f32>) {
        unsafe {
            gl.Uniform4f(location, value.x, value.y, value.z, value.w);
        }
    }

    #[allow(dead_code)]
    pub fn set_uniform_1i(&self, gl: &gl::Gl, location: i32, index: i32) {
        unsafe {
            gl.Uniform1i(location, index);
        }
    }
    
    #[allow(dead_code)]
    pub fn set_uniform_1f(&self, gl: &gl::Gl, location: i32, value: f32) {
        unsafe{
            gl.Uniform1f(location, value);
        }
    }

    pub unsafe fn destroy(self, gl: &gl::Gl) {
        gl.DeleteProgram(self.id);
    }
}

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_res(gl: &gl::Gl, res: &AssetLoader, name: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] =
            [(".vert", gl::VERTEX_SHADER), (".frag", gl::FRAGMENT_SHADER)];

        let shader_kind = POSSIBLE_EXT
            .iter()
            .find(|&&(file_extension, _)| name.ends_with(file_extension))
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::CanNotDetermineShaderTypeForResource { name: name.into() })?;

        let source = res.load_cstring(name).map_err(|e| Error::ResourceLoad {
            name: name.into(),
            inner: e,
        })?;

        Shader::from_source(gl, &source, shader_kind).map_err(|message| Error::CompileError {
            name: name.into(),
            message,
        })
    }

    pub fn from_source(
        gl: &gl::Gl,
        source: &CStr,
        kind: gl::types::GLenum,
    ) -> Result<Shader, String> {
        let id = shader_from_source(gl, source, kind)?;
        Ok(Shader { id })
    }

    #[allow(dead_code)]
    pub fn from_vert_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::VERTEX_SHADER)
    }

    #[allow(dead_code)]
    pub fn from_frag_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub unsafe fn destroy(self, gl: &gl::Gl) {
        gl.DeleteShader(self.id);
    }
}

fn shader_from_source(
    gl: &gl::Gl,
    source: &CStr,
    kind: gl::types::GLenum,
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl.CreateShader(kind) };
    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)

}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}