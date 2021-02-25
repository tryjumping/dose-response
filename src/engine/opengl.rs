#![allow(unsafe_code)]

use crate::{
    color::{Color, ColorAlpha},
    engine::DisplayInfo,
};

use std::{ffi::CString, mem, os, ptr};

use image::RgbaImage;

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
    pub glyphmap: GLuint,
    pub glyphmap_size_px: [f32; 2],
    pub tilemap: GLuint,
    pub tilemap_size_px: [f32; 2],
    pub eguimap: GLuint,
    pub eguimap_size_px: [f32; 2],
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

            gl::GenTextures(1, &mut app.glyphmap);
            check_gl_error("GenTextures glyph texture");

            gl::GenTextures(1, &mut app.tilemap);
            check_gl_error("GenTextures tilemap texture");

            gl::GenTextures(1, &mut app.eguimap);
            check_gl_error("GenTextures eguimap texture");
        }

        app
    }

    #[allow(unsafe_code)]
    pub fn initialise(&mut self, glyphmap: &RgbaImage, tilemap: &RgbaImage) {
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            check_gl_error("Enable SCISSOR_TEST");
            gl::Enable(gl::BLEND);
            check_gl_error("Enable BLEND");
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
        }

        self.glyphmap_size_px = [glyphmap.width() as f32, glyphmap.height() as f32];
        self.tilemap_size_px = [tilemap.width() as f32, tilemap.height() as f32];

        self.upload_texture(self.glyphmap, "glyphmap", glyphmap);
        self.upload_texture(self.tilemap, "tilemap", tilemap);
    }

    #[allow(unsafe_code)]
    pub fn upload_texture(&mut self, id: GLuint, name: &str, texture: &RgbaImage) {
        let (width, height) = texture.dimensions();
        // NOTE(shadower): as far as I can tell (though the opengl
        // docs could a little more explicit) the data is copied in
        // the `texImage2D` call afterwards so it is okay to pass a
        // reference here. The pointer will not be referenced
        // afterwards.
        let data_ptr: *const u8 = texture.as_ptr();
        unsafe {
            // Bind the texture
            gl::BindTexture(gl::TEXTURE_2D, id);
            check_gl_error(&format!("BindTexture {}", name));
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            check_gl_error(&format!("TexParameteri MIN FILTER {}", name));
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            check_gl_error(&format!("TexParameteri MAG FILTER {}", name));
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data_ptr as *const os::raw::c_void,
            );
            check_gl_error(&format!("TexImage2D {}", name));
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

    #[allow(unsafe_code)]
    pub fn render(&self, clear_color: Color, display_info: DisplayInfo, vertex_buffer: &[f32]) {
        let program = self.program;
        let vbo = self.vbo;
        unsafe {
            // NOTE: this ignores the `extra_px` value. Which means
            // the viewport size will allways have the same aspect
            // ratio as `display_px`. Specifically, it's `display_px *
            // DPI`.
            //
            // We could center the viewport by replacing the zeros
            // here with `extra_px * DPI / 2`. That would offset the
            // viewport's "top left corner". But we don't have the DPI
            // value here and I frankly don't care enough to bring it
            // here.
            gl::Viewport(
                0,
                0,
                display_info.viewport_size[0] as i32,
                display_info.viewport_size[1] as i32,
            );
            check_gl_error("Viewport");
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

            let rgba: ColorAlpha = clear_color.into();
            let glcolor: [f32; 4] = rgba.into();
            gl::ClearColor(glcolor[0], glcolor[1], glcolor[2], 1.0);
            check_gl_error("ClearColor");
            gl::Clear(gl::COLOR_BUFFER_BIT);
            check_gl_error("Clear");

            // Specify the layout of the vertex data
            // NOTE: this must happen only after the BufferData call
            let stride =
                crate::engine::VERTEX_COMPONENT_COUNT as i32 * mem::size_of::<GLfloat>() as i32;

            let texture_id_cstr = CString::new("texture_id").unwrap();
            let texture_id_attr = gl::GetAttribLocation(program, texture_id_cstr.as_ptr());
            check_gl_error("GetAttribLocation texture_id");
            gl::EnableVertexAttribArray(texture_id_attr as GLuint);
            check_gl_error("EnableVertexAttribArray texture_id");
            gl::VertexAttribPointer(
                texture_id_attr as GLuint,
                1,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                stride,
                ptr::null(),
            );

            assert_eq!(mem::size_of::<GLfloat>(), mem::size_of::<GLuint>());

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.glyphmap);
            check_gl_error("BindTexture glyphmap");
            let texture_index = 1;
            let glyphmap_cstr = CString::new("glyphmap").unwrap();
            gl::Uniform1i(
                gl::GetUniformLocation(program, glyphmap_cstr.as_ptr()),
                texture_index,
            );

            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, self.tilemap);
            check_gl_error("BindTexture tilemap");
            let texture_index = 2;
            let tilemap_cstr = CString::new("tilemap").unwrap();
            gl::Uniform1i(
                gl::GetUniformLocation(program, tilemap_cstr.as_ptr()),
                texture_index,
            );

            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, self.eguimap);
            check_gl_error("BindTexture eguimap");
            let texture_index = 3;
            let eguimap_cstr = CString::new("eguimap").unwrap();
            gl::Uniform1i(
                gl::GetUniformLocation(program, eguimap_cstr.as_ptr()),
                texture_index,
            );

            let display_px_cstr = CString::new("display_px").unwrap();
            gl::Uniform2f(
                gl::GetUniformLocation(program, display_px_cstr.as_ptr()),
                display_info.window_size_px[0],
                display_info.window_size_px[1],
            );
            check_gl_error("Uniform2f display_px");

            let glyphmap_size_px_cstr = CString::new("glyphmap_size_px").unwrap();
            gl::Uniform2f(
                gl::GetUniformLocation(program, glyphmap_size_px_cstr.as_ptr()),
                self.glyphmap_size_px[0],
                self.glyphmap_size_px[1],
            );
            check_gl_error("Uniform2f glyphmap_size_px");

            let tilemap_size_px_cstr = CString::new("tilemap_size_px").unwrap();
            gl::Uniform2f(
                gl::GetUniformLocation(program, tilemap_size_px_cstr.as_ptr()),
                self.tilemap_size_px[0],
                self.tilemap_size_px[1],
            );
            check_gl_error("Uniform2f tilemap_size_px");

            let eguimap_size_px_cstr = CString::new("eguimap_size_px").unwrap();
            gl::Uniform2f(
                gl::GetUniformLocation(program, eguimap_size_px_cstr.as_ptr()),
                self.eguimap_size_px[0],
                self.eguimap_size_px[1],
            );
            check_gl_error("Uniform2f eguimap_size_px");

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
                (1 * mem::size_of::<GLfloat>()) as *const GLvoid,
            );
            check_gl_error("VertexAttribPointer pos_px");

            let tile_pos_cstr = CString::new("tile_pos").unwrap();
            let tex_coord_attr = gl::GetAttribLocation(program, tile_pos_cstr.as_ptr());
            check_gl_error("GetAttribLocation tile_pos");
            gl::EnableVertexAttribArray(tex_coord_attr as GLuint);
            check_gl_error("EnableVertexAttribArray tile_pos");
            gl::VertexAttribPointer(
                tex_coord_attr as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                stride,
                (3 * mem::size_of::<GLfloat>()) as *const GLvoid,
            );
            check_gl_error("VertexAttribPointer tile_pos");

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
                (5 * mem::size_of::<GLfloat>()) as *const GLvoid,
            );
            check_gl_error("VertexAttribPointer color");
        }
    }

    /// Render vertices within a specific range and filter out pixels
    /// that don't fit inside the `clip_rect`.
    ///
    /// `clip_rect` rectangle within which to render points. In pixel size.
    ///
    /// `vertex_range` a (index, count) tuple inte a *vertex array*.
    /// `index` must be in multiples of three because the vertex array
    /// represents triangles and it's composed of three vertices per
    /// triangle. `count` is a number of vertices to draw.
    ///
    /// Example: a vertex array of 10 triangles will have 30 vertices.
    /// To draw all of them, use: (0, 30). To draw the first five, use: (0, 5*3) = (0, 15).
    /// To draw the last five, use: (15, 15).
    pub fn render_clipped_vertices(
        &self,
        display_info: DisplayInfo,
        clip_rect: [f32; 4],
        vertex_range: (i32, i32),
    ) {
        use egui::math::clamp;
        unsafe {
            // NOTE: use gl::Scissor to only render the pixels within
            // clip_rect. This makes the shader simpler compared to
            // discarding the pixels there.
            let screen_size_width = display_info.viewport_size[0];
            let screen_size_height = display_info.viewport_size[1];

            let pixels_per_point = display_info.dpi;
            let clip_min_x = pixels_per_point * clip_rect[0];
            let clip_min_y = pixels_per_point * clip_rect[1];
            let clip_max_x = pixels_per_point * clip_rect[2];
            let clip_max_y = pixels_per_point * clip_rect[3];
            let clip_min_x = clamp(clip_min_x, 0.0..=screen_size_width);
            let clip_min_y = clamp(clip_min_y, 0.0..=screen_size_height);
            let clip_max_x = clamp(clip_max_x, clip_min_x..=screen_size_width);
            let clip_max_y = clamp(clip_max_y, clip_min_y..=screen_size_height);
            let clip_min_x = clip_min_x.round() as i32;
            let clip_min_y = clip_min_y.round() as i32;
            let clip_max_x = clip_max_x.round() as i32;
            let clip_max_y = clip_max_y.round() as i32;

            // scissor Y coordinate is from the bottom
            gl::Scissor(
                clip_min_x,
                screen_size_height as i32 - clip_max_y,
                clip_max_x - clip_min_x,
                clip_max_y - clip_min_y,
            );

            gl::DrawArrays(gl::TRIANGLES, vertex_range.0, vertex_range.1);
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
            gl::DeleteTextures(1, &self.glyphmap);
            gl::DeleteTextures(1, &self.tilemap);
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
