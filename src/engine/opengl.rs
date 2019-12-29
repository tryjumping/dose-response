use crate::{
    color::{Color, ColorAlpha},
    engine::DisplayInfo,
};

use std::{ffi::CString, mem, os, ptr};

use gl::types::*;

/// The OpenGl context of our rendering pipeline. Contains the
/// shaders, textures, vao and vbos, etc.
#[derive(Default)]
pub struct OpenGlApp {
    pub program: GLuint,
    pub vertex_shader: GLuint,
    pub fragment_shader: GLuint,
    pub vao: GLuint,
    pub vbo: GLuint,
    pub texture: GLuint,
}

impl OpenGlApp {
    #[allow(unsafe_code)]
    pub fn new(vertex_source: &str, fragment_source: &str) -> Self {
        let mut app: OpenGlApp = Default::default();

        app.vertex_shader = Self::compile_shader(vertex_source, gl::VERTEX_SHADER);
        app.fragment_shader = Self::compile_shader(fragment_source, gl::FRAGMENT_SHADER);
        app.program = Self::link_program(app.vertex_shader, app.fragment_shader);

        unsafe {
            gl::GenVertexArrays(1, &mut app.vao);
            check_gl_error("GenVertexArrays");

            gl::GenBuffers(1, &mut app.vbo);
            check_gl_error("GenBuffers");

            gl::GenTextures(1, &mut app.texture);
            check_gl_error("GenTextures");
        }

        app
    }

    #[allow(unsafe_code)]
    pub fn initialise(&self, image_size: (u32, u32), image_data: &[u8]) {
        let (image_width, image_height) = image_size;
        // NOTE(shadower): as far as I can tell (though the opengl
        // docs could a little more explicit) the data is copied in
        // the `texImage2D` call afterwards so it is okay to pass a
        // reference here. The pointer will not be referenced
        // afterwards.
        let image_data_ptr: *const u8 = image_data.as_ptr();
        unsafe {
            gl::Enable(gl::BLEND);
            check_gl_error("Enable");
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            check_gl_error("BlendFunc");

            // Create Vertex Array Object
            gl::BindVertexArray(self.vao);
            check_gl_error("BindVertexArray");

            // Use shader program
            gl::UseProgram(self.program);
            check_gl_error("UseProgram");
            let out_color_cstr = CString::new("out_color").unwrap();
            gl::BindFragDataLocation(self.program, 0, out_color_cstr.as_ptr());
            check_gl_error("BindFragDataLocation");

            // Bind the texture
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            check_gl_error("BindTexture");
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            check_gl_error("TexParameteri MIN FILTER");
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            check_gl_error("TexParameteri MAG FILTER");
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                image_width as i32,
                image_height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image_data_ptr as *const os::raw::c_void,
            );
            check_gl_error("TexImage2D");
        }
    }

    #[allow(unsafe_code)]
    pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
        let shader;
        unsafe {
            shader = gl::CreateShader(ty);
            // Attempt to compile the shader
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            // Get the compile status
            let mut status = i32::from(gl::FALSE);
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != i32::from(gl::TRUE) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    ::std::str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8")
                );
            }
        }
        shader
    }

    #[allow(unsafe_code)]
    pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            // Get the link status
            let mut status = i32::from(gl::FALSE);
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != i32::from(gl::TRUE) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    ::std::str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
                );
            }
            program
        }
    }

    // TODO: make this a method on GlApp!!!
    #[allow(unsafe_code, too_many_arguments)]
    pub fn render(
        &self,
        clear_color: Color,
        display_info: DisplayInfo,
        texture_size_px: [f32; 2],
        vertex_buffer: &[f32],
    ) {
        let program = self.program;
        let texture = self.texture;
        let vbo = self.vbo;
        unsafe {
            gl::Viewport(
                0,
                0,
                display_info.physical_window_size[0] as i32,
                display_info.physical_window_size[1] as i32,
            );
            check_gl_error("Viewport");

            let rgba: ColorAlpha = clear_color.into();
            let glcolor: [f32; 4] = rgba.into();
            gl::ClearColor(glcolor[0], glcolor[1], glcolor[2], 1.0);
            check_gl_error("ClearColor");
            gl::Clear(gl::COLOR_BUFFER_BIT);
            check_gl_error("Clear");

            // Copy data to the vertex buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            check_gl_error("BindBuffer");
            // TODO: look at BufferSubData here -- that should reuse the allocation
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_buffer.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                vertex_buffer.as_ptr() as *const os::raw::c_void,
                gl::DYNAMIC_DRAW,
            );
            check_gl_error("BufferData");

            // Specify the layout of the vertex data
            // NOTE: this must happen only after the BufferData call
            let stride =
                crate::engine::VERTEX_COMPONENT_COUNT as i32 * mem::size_of::<GLfloat>() as i32;
            let pos_px_cstr = CString::new("pos_px").unwrap();
            let pos_attr = gl::GetAttribLocation(program, pos_px_cstr.as_ptr());
            check_gl_error("GetAttribLocation pos_px");
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            check_gl_error("EnableVertexAttribArray pos_px");
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                stride,
                ptr::null(),
            );
            check_gl_error("VertexAttribPointer pos_xp");

            let tile_pos_px_cstr = CString::new("tile_pos_px").unwrap();
            let tex_coord_attr = gl::GetAttribLocation(program, tile_pos_px_cstr.as_ptr());
            check_gl_error("GetAttribLocation tile_pos_px");
            gl::EnableVertexAttribArray(tex_coord_attr as GLuint);
            check_gl_error("EnableVertexAttribArray tile_pos_px");
            gl::VertexAttribPointer(
                tex_coord_attr as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                stride,
                (2 * mem::size_of::<GLfloat>()) as *const GLvoid,
            );
            check_gl_error("VertexAttribPointer tile_pos_px");

            let color_cstr = CString::new("color").unwrap();
            let color_attr = gl::GetAttribLocation(program, color_cstr.as_ptr());
            check_gl_error("GetAttribLocation color");
            gl::EnableVertexAttribArray(color_attr as GLuint);
            check_gl_error("EnableVertexAttribArray color");
            gl::VertexAttribPointer(
                color_attr as GLuint,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                stride,
                (4 * mem::size_of::<GLfloat>()) as *const GLvoid,
            );
            check_gl_error("VertexAttribPointer color");

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            check_gl_error("BindTexture");
            let texture_index = 0; // NOTE: hardcoded -- we only have 1 texture.
            let tex_cstr = CString::new("tex").unwrap();
            gl::Uniform1i(
                gl::GetUniformLocation(program, tex_cstr.as_ptr()),
                texture_index,
            );

            let native_display_px_cstr = CString::new("native_display_px").unwrap();
            gl::Uniform2f(
                gl::GetUniformLocation(program, native_display_px_cstr.as_ptr()),
                display_info.native_display_px[0],
                display_info.native_display_px[1],
            );

            let display_px_cstr = CString::new("display_px").unwrap();
            gl::Uniform2f(
                gl::GetUniformLocation(program, display_px_cstr.as_ptr()),
                display_info.display_px[0],
                display_info.display_px[1],
            );

            let extra_px_cstr = CString::new("extra_px").unwrap();
            gl::Uniform2f(
                gl::GetUniformLocation(program, extra_px_cstr.as_ptr()),
                display_info.extra_px[0],
                display_info.extra_px[1],
            );

            let texture_size_px_cstr = CString::new("texture_size_px").unwrap();
            gl::Uniform2f(
                gl::GetUniformLocation(program, texture_size_px_cstr.as_ptr()),
                texture_size_px[0],
                texture_size_px[1],
            );

            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                (vertex_buffer.len() / crate::engine::VERTEX_COMPONENT_COUNT) as i32,
            );
            check_gl_error("DrawArrays");
        }
    }
}

impl Drop for OpenGlApp {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteTextures(1, &self.texture);
        }
    }
}

#[allow(unsafe_code)]
fn check_gl_error(source: &str) {
    let err = unsafe { gl::GetError() };
    if err != gl::NO_ERROR {
        log::error!("GL error [{}]: {:?}", source, err);
    }
}
