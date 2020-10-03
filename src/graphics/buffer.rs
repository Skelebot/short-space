use gl;

pub trait BufferType {
    const BUFFER_TYPE: gl::types::GLuint;
}

pub struct BufferTypeArray;
impl BufferType for BufferTypeArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ARRAY_BUFFER;
}

pub struct BufferTypeElementArray;
impl BufferType for BufferTypeElementArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ELEMENT_ARRAY_BUFFER;
}

pub type ArrayBuffer = Buffer<BufferTypeArray>;
pub type ElementArrayBuffer = Buffer<BufferTypeElementArray>;

pub struct Buffer<B> where B: BufferType {
    vbo: gl::types::GLuint,
    _marker: ::std::marker::PhantomData<B>,
}

impl<B> Buffer<B> where B: BufferType {
    pub fn new(gl: &gl::Gl) -> Buffer<B> {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }

        Buffer {
            vbo,
            _marker: ::std::marker::PhantomData,
        }
    }

    pub fn bind(&self, gl: &gl::Gl) {
        unsafe {
            gl.BindBuffer(B::BUFFER_TYPE, self.vbo);
        }
    }

    pub fn unbind(&self, gl: &gl::Gl) {
        unsafe {
            gl.BindBuffer(B::BUFFER_TYPE, 0);
        }
    }

    pub fn static_draw_data<T>(&self, gl: &gl::Gl, data: &[T]) {
        unsafe {
            gl.BufferData(
                B::BUFFER_TYPE, // target
                (data.len() * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
        }
    }

    pub unsafe fn destroy(&mut self, gl: &gl::Gl) {
        gl.DeleteBuffers(1, &self.vbo);
    }
}

///Vertex Array Object wrapper
pub struct VertexArray {
    vao: gl::types::GLuint
}

impl VertexArray {
    pub fn new(gl: &gl::Gl) -> VertexArray {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut vao);
        }

        VertexArray {
            vao
        }
    }

    pub fn bind(&self, gl: &gl::Gl) {
        unsafe {
            gl.BindVertexArray(self.vao);
        }
    }

    pub fn unbind(&self, gl: &gl::Gl) {
        unsafe {
            gl.BindVertexArray(0);
        }
    }

    pub fn destroy(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}