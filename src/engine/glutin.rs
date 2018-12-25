use crate::{
    color::Color,
    engine::{
        self, Drawcall, Mouse, OpenGlApp, RunningState, Settings, TextMetrics, UpdateFn, Vertex,
    },
    point::Point,
    state::State,
    util,
};

use std::mem;

use glutin::{dpi::*, ElementState, GlContext};

pub struct Metrics {
    tile_width_px: i32,
}

impl TextMetrics for Metrics {
    fn tile_width_px(&self) -> i32 {
        self.tile_width_px
    }
}

#[allow(cyclomatic_complexity, unsafe_code)]
pub fn main_loop(
    display_size: Point,
    default_background: Color,
    window_title: &str,
    mut state: Box<State>,
    update: UpdateFn,
) {
    // Force the DPI factor to be 1.0
    // https://docs.rs/glium/0.22.0/glium/glutin/dpi/index.html
    //
    // NOTE: without this, the window size and contents will be scaled
    // by some heuristic the OS will do. For now, that means blurry
    // fonts and so on. I think once we add support for multiple font
    // sizes, this can be handled gracefully. Until then though, let's
    // just force 1.0. The players can always resize the window
    // manually.
    //
    // Apparently, the only way to set the DPI factor is via this
    // environment variable.
    //
    // This PR might fix it?
    // https://github.com/tomaka/winit/pull/606/files
    std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");

    // Force winit unix backend to X11.
    //
    // Right now, this produces better results on Wayland (Fedora 28).
    // Ideally, we should remove this once winit looks better. We're
    // using `winit 0.18`, the latest release as of writing this.
    //
    // Here are the current issues under wayland:
    // 1. The window decorations look different from the rest of the system
    // 2. The full screen just maximises the window -- the decorations are still visible.
    //    - NOTE: we can use `window.set_decorations(false)` to fix it
    //    - still, feels like we shouldn't have to
    //
    // Both are fixed with the line below:
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let tilesize = super::TILESIZE;
    let (desired_window_width, desired_window_height) = (
        display_size.x as u32 * tilesize as u32,
        display_size.y as u32 * tilesize as u32,
    );

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title(window_title)
        .with_dimensions(LogicalSize::new(
            desired_window_width.into(),
            desired_window_height.into(),
        ));
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
    }

    unsafe {
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
    }

    let image = {
        use std::io::Cursor;
        let data = &include_bytes!(concat!(env!("OUT_DIR"), "/font.png"))[..];
        image::load(Cursor::new(data), image::PNG)
            .unwrap()
            .to_rgba()
    };

    let image_width = image.width();
    let image_height = image.height();

    let vs_source = include_str!("../shader_150.glslv");
    let fs_source = include_str!("../shader_150.glslf");
    let opengl_app = OpenGlApp::new(vs_source, fs_source);
    opengl_app.initialise(image_width, image_height, image.into_raw().as_ptr());

    let mut mouse = Mouse::new();
    let mut settings = Settings { fullscreen: false };
    let window_size_px = Point::new(desired_window_width as i32, desired_window_height as i32);

    let mut display = engine::Display::new(
        display_size,
        Point::from_i32(display_size.y / 2),
        tilesize as i32,
    );
    let mut drawcalls: Vec<Drawcall> = Vec::with_capacity(engine::DRAWCALL_CAPACITY);
    assert_eq!(mem::size_of::<Vertex>(), engine::VERTEX_COMPONENT_COUNT * 4);
    let mut vertex_buffer: Vec<f32> = Vec::with_capacity(engine::VERTEX_BUFFER_CAPACITY);
    let mut overall_max_drawcall_count = 0;
    let mut keys = vec![];

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            log::debug!("{:?}", event);
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,

                    glutin::WindowEvent::Resized(logical_size) => {
                        let dpi_factor = gl_window.get_hidpi_factor();
                        gl_window.resize(logical_size.to_physical(dpi_factor));
                    }

                    glutin::WindowEvent::CursorMoved { position, .. } => {
                        let x = util::clamp(0, position.x as i32, window_size_px.x - 1);
                        let y = util::clamp(0, position.y as i32, window_size_px.y - 1);
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

                    glutin::WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button,
                        ..
                    } => {
                        use glutin::MouseButton::*;
                        match button {
                            Left => {
                                mouse.left_is_down = true;
                            }
                            Right => {
                                mouse.right_is_down = true;
                            }
                            _ => {}
                        }
                    }

                    glutin::WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button,
                        ..
                    } => {
                        use glutin::MouseButton::*;
                        match button {
                            Left => {
                                mouse.left_clicked = true;
                                mouse.left_is_down = false;
                            }
                            Right => {
                                mouse.right_clicked = true;
                                mouse.right_is_down = false;
                            }
                            _ => {}
                        }
                    }

                    _ => (),
                },
                _ => (),
            }
        });

        // TODO: calculate this properly!
        let dt = std::time::Duration::from_millis(16);
        let fps = 60;

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

        mouse.left_clicked = false;
        mouse.right_clicked = false;
        keys.clear();

        drawcalls.clear();
        display.push_drawcalls(&mut drawcalls);

        if drawcalls.len() > overall_max_drawcall_count {
            overall_max_drawcall_count = drawcalls.len();
        }

        if drawcalls.len() > engine::DRAWCALL_CAPACITY {
            log::warn!(
                "Warning: drawcall count exceeded initial capacity {}. Current count: {}.",
                engine::DRAWCALL_CAPACITY,
                drawcalls.len(),
            );
        }

        let display_info = engine::calculate_display_info(
            [window_size_px.x as f32, window_size_px.y as f32],
            display_size,
            tilesize,
        );

        vertex_buffer.clear();
        engine::build_vertices(
            &drawcalls,
            &mut vertex_buffer,
            display_info.native_display_px,
        );

        if vertex_buffer.len() > engine::VERTEX_BUFFER_CAPACITY {
            log::warn!(
                "Warning: vertex count exceeded initial capacity {}. Current count: {} ",
                engine::VERTEX_BUFFER_CAPACITY,
                vertex_buffer.len(),
            );
        }

        engine::opengl_render(
            opengl_app.program,
            opengl_app.texture,
            default_background,
            opengl_app.vbo,
            display_info,
            [image_width as f32, image_height as f32],
            &vertex_buffer,
        );
        gl_window.swap_buffers().unwrap();
    }
}
