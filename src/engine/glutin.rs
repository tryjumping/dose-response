use crate::{
    color::Color,
    engine::{OpenGlApp, UpdateFn},
    point::Point,
    state::State,
};

use glutin::{dpi::*, GlContext};

#[allow(cyclomatic_complexity, unsafe_code)]
pub fn main_loop(
    _display_size: Point,
    _default_background: Color,
    _window_title: &str,
    mut _state: Box<State>,
    _update: UpdateFn,
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

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world!")
        .with_dimensions(LogicalSize::new(1024.0, 768.0));
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
    let sdl_app = OpenGlApp::new(vs_source, fs_source);
    sdl_app.initialise(image_width, image_height, image.into_raw().as_ptr());

    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => running = false,
                glutin::WindowEvent::Resized(logical_size) => {
                    let dpi_factor = gl_window.get_hidpi_factor();
                    gl_window.resize(logical_size.to_physical(dpi_factor));
                }
                _ => (),
            },
            _ => (),
        });

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        gl_window.swap_buffers().unwrap();
    }
}
