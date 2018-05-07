use color::{Color, ColorAlpha};
use engine::{self, DisplayInfo, Drawcall, Mouse, Settings, TextMetrics, UpdateFn, Vertex};
use game::RunningState;
use keys::KeyCode;
use point::Point;
use state::State;
use util;

use std::ffi::CString;
use std::mem;
use std::os;
use std::ptr;
use std::time::{Duration, Instant};

use sdl2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{self, Keycode as BackendKey};
use sdl2::video::Window;
use gl;
use gl::types::*;
use image;


// const DESIRED_FPS: u64 = 60;
// const EXPECTED_FRAME_LENGTH: Duration = Duration::from_millis(1000 / DESIRED_FPS);
const VERTEX_BUFFER_CAPACITY: usize = engine::VERTEX_COMPONENT_COUNT * engine::VERTEX_CAPACITY;

pub struct Metrics {
    tile_width_px: i32,
}

impl TextMetrics for Metrics {
    fn tile_width_px(&self) -> i32 {
        self.tile_width_px
    }
}


fn key_code_from_backend(backend_code: BackendKey) -> Option<KeyCode> {
    match backend_code {
        BackendKey::Return => Some(KeyCode::Enter),
        BackendKey::Escape => Some(KeyCode::Esc),
        BackendKey::Space => Some(KeyCode::Space),

        BackendKey::Num0 => Some(KeyCode::D0),
        BackendKey::Num1 => Some(KeyCode::D1),
        BackendKey::Num2 => Some(KeyCode::D2),
        BackendKey::Num3 => Some(KeyCode::D3),
        BackendKey::Num4 => Some(KeyCode::D4),
        BackendKey::Num5 => Some(KeyCode::D5),
        BackendKey::Num6 => Some(KeyCode::D6),
        BackendKey::Num7 => Some(KeyCode::D7),
        BackendKey::Num8 => Some(KeyCode::D8),
        BackendKey::Num9 => Some(KeyCode::D9),

        BackendKey::A => Some(KeyCode::A),
        BackendKey::B => Some(KeyCode::B),
        BackendKey::C => Some(KeyCode::C),
        BackendKey::D => Some(KeyCode::D),
        BackendKey::E => Some(KeyCode::E),
        BackendKey::F => Some(KeyCode::F),
        BackendKey::G => Some(KeyCode::G),
        BackendKey::H => Some(KeyCode::H),
        BackendKey::I => Some(KeyCode::I),
        BackendKey::J => Some(KeyCode::J),
        BackendKey::K => Some(KeyCode::K),
        BackendKey::L => Some(KeyCode::L),
        BackendKey::M => Some(KeyCode::M),
        BackendKey::N => Some(KeyCode::N),
        BackendKey::O => Some(KeyCode::O),
        BackendKey::P => Some(KeyCode::P),
        BackendKey::Q => Some(KeyCode::Q),
        BackendKey::R => Some(KeyCode::R),
        BackendKey::S => Some(KeyCode::S),
        BackendKey::T => Some(KeyCode::T),
        BackendKey::U => Some(KeyCode::U),
        BackendKey::V => Some(KeyCode::V),
        BackendKey::W => Some(KeyCode::W),
        BackendKey::X => Some(KeyCode::X),
        BackendKey::Y => Some(KeyCode::Y),
        BackendKey::Z => Some(KeyCode::Z),

        BackendKey::F1 => Some(KeyCode::F1),
        BackendKey::F2 => Some(KeyCode::F2),
        BackendKey::F3 => Some(KeyCode::F3),
        BackendKey::F4 => Some(KeyCode::F4),
        BackendKey::F5 => Some(KeyCode::F5),
        BackendKey::F6 => Some(KeyCode::F6),
        BackendKey::F7 => Some(KeyCode::F7),
        BackendKey::F8 => Some(KeyCode::F8),
        BackendKey::F9 => Some(KeyCode::F9),
        BackendKey::F10 => Some(KeyCode::F10),
        BackendKey::F11 => Some(KeyCode::F11),
        BackendKey::F12 => Some(KeyCode::F12),

        BackendKey::Right => Some(KeyCode::Right),
        BackendKey::Left => Some(KeyCode::Left),
        BackendKey::Down => Some(KeyCode::Down),
        BackendKey::Up => Some(KeyCode::Up),

        BackendKey::Kp1 => Some(KeyCode::NumPad1),
        BackendKey::Kp2 => Some(KeyCode::NumPad2),
        BackendKey::Kp3 => Some(KeyCode::NumPad3),
        BackendKey::Kp4 => Some(KeyCode::NumPad4),
        BackendKey::Kp5 => Some(KeyCode::NumPad5),
        BackendKey::Kp6 => Some(KeyCode::NumPad6),
        BackendKey::Kp7 => Some(KeyCode::NumPad7),
        BackendKey::Kp8 => Some(KeyCode::NumPad8),
        BackendKey::Kp9 => Some(KeyCode::NumPad9),
        BackendKey::Kp0 => Some(KeyCode::NumPad0),

        _ => None,
    }
}



#[allow(unsafe_code)]
fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
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
                ::std::str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}


#[allow(unsafe_code)]
fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
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
                ::std::str::from_utf8(&buf)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}


#[allow(unsafe_code)]
fn render(window: &mut Window,
          program: GLuint,
          texture: GLuint,
          clear_color: Color,
          vbo: GLuint,
          display_info: DisplayInfo,
          texture_size_px: [f32; 2],
          vertex_buffer: &[f32])
{
    unsafe {
        gl::Viewport(0, 0,
                     display_info.window_size_px[0] as i32, display_info.window_size_px[1] as i32);
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
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertex_buffer.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       vertex_buffer.as_ptr() as *const os::raw::c_void,
                       gl::DYNAMIC_DRAW);
        check_gl_error("BufferData");

        // Specify the layout of the vertex data
        // NOTE: this must happen only after the BufferData call
        let stride = engine::VERTEX_COMPONENT_COUNT as i32 * mem::size_of::<GLfloat>() as i32;
        let pos_attr = gl::GetAttribLocation(program,
                                             CString::new("pos_px").unwrap().as_ptr());
        check_gl_error("GetAttribLocation pos_px");
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        check_gl_error("EnableVertexAttribArray pos_px");
        gl::VertexAttribPointer(pos_attr as GLuint, 2,
                                gl::FLOAT, gl::FALSE as GLboolean,
                                stride,
                                ptr::null());
        check_gl_error("VertexAttribPointer pos_xp");

        let tex_coord_attr = gl::GetAttribLocation(program,
                                                   CString::new("tile_pos_px").unwrap().as_ptr());
        check_gl_error("GetAttribLocation tile_pos_px");
        gl::EnableVertexAttribArray(tex_coord_attr as GLuint);
        check_gl_error("EnableVertexAttribArray tile_pos_px");
        gl::VertexAttribPointer(tex_coord_attr as GLuint, 2,
                                gl::FLOAT, gl::FALSE as GLboolean,
                                stride,
                                (2 * mem::size_of::<GLfloat>()) as *const GLvoid);
        check_gl_error("VertexAttribPointer tile_pos_px");

        let color_attr = gl::GetAttribLocation(program,
                                               CString::new("color").unwrap().as_ptr());
        check_gl_error("GetAttribLocation color");
        gl::EnableVertexAttribArray(color_attr as GLuint);
        check_gl_error("EnableVertexAttribArray color");
        gl::VertexAttribPointer(color_attr as GLuint, 4,
                                gl::FLOAT, gl::FALSE as GLboolean,
                                stride,
                                (4 * mem::size_of::<GLfloat>()) as *const GLvoid);
        check_gl_error("VertexAttribPointer color");


        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        check_gl_error("BindTexture");
        let texture_index = 0;  // NOTE: hardcoded -- we only have 1 texture.
        gl::Uniform1i(gl::GetUniformLocation(program,
                                             CString::new("tex").unwrap().as_ptr()),
                      texture_index);

        gl::Uniform2f(
            gl::GetUniformLocation(program,
                                   CString::new("native_display_px").unwrap().as_ptr()),
            display_info.native_display_px[0], display_info.native_display_px[1]);

        gl::Uniform2f(
            gl::GetUniformLocation(program,
                                   CString::new("display_px").unwrap().as_ptr()),
            display_info.display_px[0], display_info.display_px[1]);

        gl::Uniform2f(
            gl::GetUniformLocation(program,
                                   CString::new("extra_px").unwrap().as_ptr()),
            display_info.extra_px[0], display_info.extra_px[1]);

        gl::Uniform2f(
            gl::GetUniformLocation(program,
                                   CString::new("texture_size_px").unwrap().as_ptr()),
            texture_size_px[0], texture_size_px[1]);

        gl::DrawArrays(gl::TRIANGLES, 0, (vertex_buffer.len() / engine::VERTEX_COMPONENT_COUNT) as i32);
        check_gl_error("DrawArrays");

        window.gl_swap_window();
    }
}


#[allow(unsafe_code)]
fn check_gl_error(source: &str) {
    let err = unsafe{gl::GetError()};
    if err != gl::NO_ERROR {
        println!("GL error [{}]: {:?}", source, err);
    }
}


#[derive(Default)]
struct SdlApp {
    program: GLuint,
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    vao: GLuint,
    vbo: GLuint,
    texture: GLuint,
}

impl SdlApp {
    #[allow(unsafe_code)]
    fn new(vertex_source: &str, fragment_source: &str) -> Self {
        let mut app: SdlApp = Default::default();

        app.vertex_shader = compile_shader(vertex_source, gl::VERTEX_SHADER);
        app.fragment_shader = compile_shader(fragment_source, gl::FRAGMENT_SHADER);
        app.program = link_program(app.vertex_shader, app.fragment_shader);

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
    fn initialise(&self, image_width: u32, image_height: u32, image_data: *const u8) {
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
            gl::BindFragDataLocation(self.program, 0,
                                     CString::new("out_color").unwrap().as_ptr());
            check_gl_error("BindFragDataLocation");

            // Bind the texture
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            check_gl_error("BindTexture");
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            check_gl_error("TexParameteri MIN FILTER");
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            check_gl_error("TexParameteri MAG FILTER");
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32,
                           image_width as i32, image_height as i32, 0, gl::RGBA,
                           gl::UNSIGNED_BYTE, image_data as *const os::raw::c_void);
            check_gl_error("TexImage2D");
        }
    }
}

impl Drop for SdlApp {
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



pub fn main_loop(
    display_size: Point,
    default_background: Color,
    window_title: &str,
    mut state: State,
    update: UpdateFn,
) {
    let tilesize = super::TILESIZE;
    let (desired_window_width, desired_window_height) = (
        display_size.x as u32 * tilesize as u32,
        display_size.y as u32 * tilesize as u32,
    );

    let sdl_context = sdl2::init()
        .expect("SDL context creation failed.");
    let video_subsystem = sdl_context.video()
        .expect("SDL video subsystem creation failed.");

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_double_buffer(true);
    gl_attr.set_depth_size(0);


    // NOTE: add `.fullscreen_desktop()` to start in fullscreen.
    let mut window = video_subsystem.window(window_title, desired_window_width, desired_window_height)
        .opengl()
        .position_centered()
        .build()
        .expect("SDL window creation failed.");

    let _ctx = window.gl_create_context()
        .expect("SDL GL context creation failed.");
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);


    let image = {
        use std::io::Cursor;
        let data = &include_bytes!(concat!(env!("OUT_DIR"), "/font.png"))[..];
        let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
        img
    };

    let image_width = image.width();
    let image_height = image.height();

    let vs_source = include_str!("../shader_150.glslv");
    let fs_source = include_str!("../shader_150.glslf");
    let sdl_app = SdlApp::new(vs_source, fs_source);
    sdl_app.initialise(image_width, image_height, image.into_raw().as_ptr());

    let mut event_pump = sdl_context.event_pump()
        .expect("SDL event pump creation failed.");

    let mut mouse = Mouse::new();
    let mut settings = Settings { fullscreen: false };
    let mut window_size_px = Point::new(desired_window_width as i32, desired_window_height as i32);
    let mut display = engine::Display::new(
        display_size, Point::from_i32(display_size.y / 2), tilesize as i32);
    let mut drawcalls: Vec<Drawcall> = Vec::with_capacity(engine::DRAWCALL_CAPACITY);
    assert_eq!(mem::size_of::<Vertex>(), engine::VERTEX_COMPONENT_COUNT * 4);
    let mut vertex_buffer: Vec<f32> = Vec::with_capacity(VERTEX_BUFFER_CAPACITY);
    let mut overall_max_drawcall_count = 0;
    let mut keys = vec![];
    let mut previous_frame_start_time = Instant::now();
    let mut fps_clock = Duration::from_millis(0);
    let mut frames_in_current_second = 0;
    let mut fps = 0;
    // NOTE: This will wrap after running continuously for over 64
    // years at 60 FPS. 32 bits are just fine.
    let mut current_frame_id: i32 = 0;
    let mut running = true;

    while running {
        let frame_start_time = Instant::now();
        let dt = frame_start_time.duration_since(previous_frame_start_time);
        previous_frame_start_time = frame_start_time;

        // Calculate FPS
        fps_clock = fps_clock + dt;
        frames_in_current_second += 1;
        current_frame_id += 1;
        if util::num_milliseconds(fps_clock) > 1000 {
            fps = frames_in_current_second;
            frames_in_current_second = 1;
            fps_clock = Duration::new(0, 0);
        }

        for event in event_pump.poll_iter() {
            match event {

                Event::Quit {..} => {
                    running = false;
                },

                Event::KeyDown { keycode: Some(backend_code), keymod, ..} => {
                    if let Some(code) = key_code_from_backend(backend_code) {
                        let key = super::Key {
                            code: code,
                            alt: keymod.intersects(keyboard::LALTMOD | keyboard::RALTMOD),
                            ctrl: keymod.intersects(keyboard::LCTRLMOD | keyboard::RCTRLMOD),
                            shift: keymod.intersects(keyboard::LSHIFTMOD | keyboard::RSHIFTMOD),
                        };
                        keys.push(key);
                    }
                }

                Event::TextInput { text, .. } => {
                    if text.contains('?') {
                        let key = super::Key {
                            code: KeyCode::QuestionMark,
                            alt: false,
                            ctrl: false,
                            shift: false,
                        };
                        keys.push(key);
                    }
                }

                Event::MouseMotion {x, y, ..} => {
                    let x = util::clamp(0, x, window_size_px.x - 1);
                    let y = util::clamp(0, y, window_size_px.y - 1);
                    mouse.screen_pos = Point { x, y };

                    let tile_width = window_size_px.x / display_size.x;
                    let mouse_tile_x = x / tile_width;

                    let tile_height = window_size_px.y / display_size.y;
                    let mouse_tile_y = y / tile_height;

                    mouse.tile_pos = Point {
                        x: mouse_tile_x,
                        y: mouse_tile_y,
                    };
                }

                Event::MouseButtonDown {..} => {
                    // NOTE: do nothing. We handle everything in the mouse up event
                }

                Event::MouseButtonUp {mouse_btn, ..} => {
                    use sdl2::mouse::MouseButton::*;
                    match mouse_btn {
                        Left => {
                            mouse.left = true;
                        }
                        Right => {
                            mouse.right = true;
                        }
                        _ => {}
                    }
                }

                Event::Window { win_event: WindowEvent::Resized(width, height), .. } => {
                    println!("Window resized to: {}x{}", width, height);
                    window_size_px = Point::new(width, height);
                }

                _ => {}
            }
        }

        let previous_settings = settings;

        let update_result = update(
            &mut state,
            dt,
            display_size,
            fps,
            &keys,
            mouse,
            &mut settings,
            &Metrics {
                tile_width_px: tilesize as i32,
            },
            &mut display,
        );

        match update_result {
            RunningState::Running => {}
            RunningState::NewGame(new_state) => {
                state = new_state;
            }
            RunningState::Stopped => break,
        }

        mouse.left = false;
        mouse.right = false;
        keys.clear();

        if cfg!(feature = "fullscreen") {
            use sdl2::video::FullscreenType::*;
            if previous_settings.fullscreen != settings.fullscreen {
                if settings.fullscreen {
                    println!("[{}] Switching to (desktop-type) fullscreen", current_frame_id);
                    if let Err(err) = window.set_fullscreen(Desktop) {
                        println!("[{}] WARNING: Could not switch to fullscreen:", current_frame_id);
                        println!("{:?}", err);
                    }
                } else {
                    println!("[{}] Switching fullscreen off", current_frame_id);
                    if let Err(err) = window.set_fullscreen(Off) {
                        println!("[{}] WARNING: Could not leave fullscreen:", current_frame_id);
                        println!("{:?}", err);
                    }
                }
            }
        }

        // println!("Pre-draw duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);


        drawcalls.clear();
        display.push_drawcalls(&mut drawcalls);

        if drawcalls.len() > overall_max_drawcall_count {
            overall_max_drawcall_count = drawcalls.len();
        }

        if drawcalls.len() > engine::DRAWCALL_CAPACITY {
            println!(
                "Warning: drawcall count exceeded initial capacity {}. Current count: {}.",
                engine::DRAWCALL_CAPACITY,
                drawcalls.len(),
            );
        }

        vertex_buffer.clear();
        engine::build_vertices(&drawcalls, &mut vertex_buffer);

        if vertex_buffer.len() > VERTEX_BUFFER_CAPACITY {
            println!(
                "Warning: vertex count exceeded initial capacity {}. Current count: {} ",
                VERTEX_BUFFER_CAPACITY,
                vertex_buffer.len(),
            );
        }

        // println!("Pre-present duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        // NOTE: render

        let display_info = engine::calculate_display_info(
            [window_size_px.x as f32, window_size_px.y as f32],
            display_size,
            tilesize);

        render(&mut window,
               sdl_app.program,
               sdl_app.texture,
               default_background,
               sdl_app.vbo,
               display_info,
               [image_width as f32, image_height as f32],
               &vertex_buffer);


        // println!("Code duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        // if let Some(sleep_duration) = EXPECTED_FRAME_LENGTH.checked_sub(frame_start_time.elapsed()) {
        //     ::std::thread::sleep(sleep_duration);
        // };

        // println!("Total frame duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

    }


    println!("Drawcall count: {}. Capacity: {}.",
             overall_max_drawcall_count, engine::DRAWCALL_CAPACITY);
}
