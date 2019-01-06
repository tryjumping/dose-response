use crate::{
    color::Color,
    engine::{
        self, Display, Drawcall, Mouse, OpenGlApp, RunningState, Settings, TextMetrics, UpdateFn,
        Vertex,
    },
    keys::KeyCode,
    point::Point,
    state::State,
    util,
};

use std::{
    mem,
    time::{Duration, Instant},
};

use glutin::{
    dpi::{LogicalPosition, LogicalSize},
    ElementState, GlContext, KeyboardInput, MonitorId, VirtualKeyCode as BackendKey,
};

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

        BackendKey::Key0 => Some(KeyCode::D0),
        BackendKey::Key1 => Some(KeyCode::D1),
        BackendKey::Key2 => Some(KeyCode::D2),
        BackendKey::Key3 => Some(KeyCode::D3),
        BackendKey::Key4 => Some(KeyCode::D4),
        BackendKey::Key5 => Some(KeyCode::D5),
        BackendKey::Key6 => Some(KeyCode::D6),
        BackendKey::Key7 => Some(KeyCode::D7),
        BackendKey::Key8 => Some(KeyCode::D8),
        BackendKey::Key9 => Some(KeyCode::D9),

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

        BackendKey::Numpad1 => Some(KeyCode::NumPad1),
        BackendKey::Numpad2 => Some(KeyCode::NumPad2),
        BackendKey::Numpad3 => Some(KeyCode::NumPad3),
        BackendKey::Numpad4 => Some(KeyCode::NumPad4),
        BackendKey::Numpad5 => Some(KeyCode::NumPad5),
        BackendKey::Numpad6 => Some(KeyCode::NumPad6),
        BackendKey::Numpad7 => Some(KeyCode::NumPad7),
        BackendKey::Numpad8 => Some(KeyCode::NumPad8),
        BackendKey::Numpad9 => Some(KeyCode::NumPad9),
        BackendKey::Numpad0 => Some(KeyCode::NumPad0),

        _ => None,
    }
}

fn get_current_monitor(monitors: &[MonitorId], window_pos: Point) -> Option<MonitorId> {
    for monitor in monitors {
        let monitor_pos = {
            let pos = monitor.get_position();
            Point::new(pos.x as i32, pos.y as i32)
        };
        let monitor_dimensions = {
            let dim = monitor.get_dimensions();
            Point::new(dim.width as i32, dim.height as i32)
        };

        let monitor_bottom_left = monitor_pos + monitor_dimensions;
        if window_pos >= monitor_pos && window_pos < monitor_bottom_left {
            return Some(monitor.clone());
        }
    }

    monitors.iter().cloned().next()
}

fn change_tilesize(
    new_tilesize: u32,
    tilesize: &mut u32,
    display: &mut Display,
    settings: &mut Settings,
    desired_window_width: &mut u32,
    desired_window_height: &mut u32,
) {
    if crate::engine::AVAILABLE_FONT_SIZES.contains(&(new_tilesize as i32)) {
        log::info!("Changing tilesize from {} to {}", tilesize, new_tilesize);
        *tilesize = new_tilesize;
        *desired_window_width = display.display_size.x as u32 * new_tilesize;
        *desired_window_height = display.display_size.y as u32 * new_tilesize;
        display.tilesize = new_tilesize as i32;
        settings.font_size = new_tilesize as i32;
    } else {
        log::warn!(
            "Trying to switch to a tilesize that's not available: {}. Only these ones exist: {:?}",
            new_tilesize,
            crate::engine::AVAILABLE_FONT_SIZES
        );
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

    let mut tilesize = super::TILESIZE;
    let mut desired_window_width = display_size.x as u32 * tilesize as u32;
    let mut desired_window_height = display_size.y as u32 * tilesize as u32;

    log::debug!(
        "Requested display in tiles: {} x {}",
        display_size.x,
        display_size.y
    );
    log::debug!(
        "Desired window size: {} x {}",
        desired_window_width,
        desired_window_height
    );

    let mut events_loop = glutin::EventsLoop::new();
    log::debug!("Created events loop: {:?}", events_loop);
    let window = glutin::WindowBuilder::new()
        .with_title(window_title)
        .with_dimensions(LogicalSize::new(
            desired_window_width.into(),
            desired_window_height.into(),
        ))
        .with_resizable(false);
    log::debug!("Created window builder: {:?}", window);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        // See: https://github.com/PistonDevelopers/conrod/pull/1032/
        .with_multisampling(4);;
    log::debug!("Created context.");
    let gl_window = match glutin::GlWindow::new(window, context, &events_loop) {
        Ok(gl_window) => gl_window,
        Err(error) => {
            log::error!("Could not create `glutin::GlWindow`: {:?}", error);
            panic!("Aborting!");
        }
    };

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    }
    log::debug!("Loaded OpenGL symbols.");

    // We'll just assume the monitors won't change throughout the game.
    let monitors: Vec<_> = events_loop.get_available_monitors().collect();
    log::debug!("Got all available monitors: {:?}", monitors);

    let image = {
        use std::io::Cursor;
        let data = &include_bytes!(concat!(env!("OUT_DIR"), "/font.png"))[..];
        image::load(Cursor::new(data), image::PNG)
            .unwrap()
            .to_rgba()
    };
    log::debug!("Loaded font image.");

    let image_width = image.width();
    let image_height = image.height();

    let vs_source = include_str!("../shader_150.glslv");
    let fs_source = include_str!("../shader_150.glslf");
    let opengl_app = OpenGlApp::new(vs_source, fs_source);
    log::debug!("Created opengl app.");
    opengl_app.initialise(image_width, image_height, image.into_raw().as_ptr());
    log::debug!("Initialised opengl app.");

    // Main loop
    let mut window_pos = {
        match gl_window.get_position() {
            Some(LogicalPosition { x, y }) => Point::new(x as i32, y as i32),
            None => Default::default(),
        }
    };
    log::debug!("Window pos: {:?}", window_pos);
    let mut pre_fullscreen_window_pos = window_pos;

    let mut current_monitor = get_current_monitor(&monitors, window_pos);
    log::debug!("All monitors:");
    for monitor in &monitors {
        log::debug!(
            "* {:?}, pos: {:?}, size: {:?}",
            monitor.get_name(),
            monitor.get_position(),
            monitor.get_dimensions()
        );
    }
    log::debug!(
        "Current monitor: {:?}, pos: {:?}, size: {:?}",
        current_monitor.as_ref().map(|m| m.get_name()),
        current_monitor.as_ref().map(|m| m.get_position()),
        current_monitor.as_ref().map(|m| m.get_dimensions())
    );

    let mut mouse = Mouse::new();
    let mut settings = Settings {
        fullscreen: false,
        font_size: tilesize as i32,
    };
    let mut window_size_px = Point::new(desired_window_width as i32, desired_window_height as i32);

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
    let mut previous_frame_start_time = Instant::now();
    let mut switched_from_fullscreen = false;
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
        fps_clock += dt;
        frames_in_current_second += 1;
        current_frame_id += 1;
        if util::num_milliseconds(fps_clock) > 1000 {
            fps = frames_in_current_second;
            frames_in_current_second = 1;
            fps_clock = Duration::new(0, 0);
        }

        events_loop.poll_events(|event| {
            log::debug!("{:?}", event);
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,

                    glutin::WindowEvent::Resized(LogicalSize { width, height }) => {
                        // let dpi_factor = gl_window.get_hidpi_factor();
                        // gl_window.resize(logical_size.to_physical(dpi_factor));
                        let height = height as i32;
                        let width = width as i32;
                        log::info!("Window resized to: {}x{}", width, height);
                        let new_window_size_px = Point::new(width, height);
                        if window_size_px != new_window_size_px {
                            window_size_px = new_window_size_px;

                            // NOTE: Update the tilesize if we get a perfect match
                            if height > 0 && height % crate::DISPLAY_SIZE.y == 0 {
                                let new_tilesize = height / crate::DISPLAY_SIZE.y;
                                change_tilesize(
                                    new_tilesize as u32,
                                    &mut tilesize,
                                    &mut display,
                                    &mut settings,
                                    &mut desired_window_width,
                                    &mut desired_window_height,
                                );
                            };
                        }
                    }

                    glutin::WindowEvent::Moved(new_pos) => {
                        if settings.fullscreen || switched_from_fullscreen {
                            // Don't update the window position
                            //
                            // Even after we switch from
                            // fullscreen, the `Moved` event has a
                            // wrong value that messes things up.
                            // So we restore the previous position
                            // manually instead.
                        } else {
                            log::debug!(
                                "[FRAME {}] Window moved to: {:?}",
                                current_frame_id,
                                new_pos
                            );
                            window_pos.x = new_pos.x as i32;
                            window_pos.y = new_pos.y as i32;
                            current_monitor = get_current_monitor(&monitors, window_pos);
                            log::debug!(
                                "Current monitor: {:?}, pos: {:?}, size: {:?}",
                                current_monitor.as_ref().map(|m| m.get_name()),
                                current_monitor.as_ref().map(|m| m.get_position()),
                                current_monitor.as_ref().map(|m| m.get_dimensions())
                            );
                        }
                    }

                    glutin::WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(backend_code),
                                state: ElementState::Pressed,
                                scancode,
                                modifiers,
                                ..
                            },
                        ..
                    } => {
                        log::debug!(
                            "KeyDown backend_code: {:?}, scancode: {:?}, modifiers: {:?}",
                            backend_code,
                            scancode,
                            modifiers,
                        );
                        if let Some(code) = key_code_from_backend(backend_code) {
                            let key = super::Key {
                                code,
                                alt: modifiers.alt,
                                ctrl: modifiers.ctrl,
                                shift: modifiers.shift,
                            };
                            log::debug!("Detected key {:?}", key);
                            keys.push(key);
                        }
                    }

                    glutin::WindowEvent::ReceivedCharacter(chr) => {
                        log::debug!("Received character: {:?}", chr);
                        if chr == '?' {
                            let key = super::Key {
                                code: KeyCode::QuestionMark,
                                alt: false,
                                ctrl: false,
                                shift: false,
                            };
                            log::debug!("Detected key {:?}", key);
                            keys.push(key);
                        }
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

        mouse.left_clicked = false;
        mouse.right_clicked = false;
        keys.clear();

        if cfg!(feature = "fullscreen") {
            if previous_settings.fullscreen != settings.fullscreen {
                if settings.fullscreen {
                    log::info!("[{}] Switching to fullscreen", current_frame_id);
                    gl_window.set_decorations(false);
                    gl_window.set_resizable(true);
                    if let Some(ref monitor) = current_monitor {
                        pre_fullscreen_window_pos = window_pos;
                        log::debug!(
                            "Monitor: {:?}, pos: {:?}, dimensions: {:?}",
                            monitor.get_name(),
                            monitor.get_position(),
                            monitor.get_dimensions()
                        );
                        gl_window.set_fullscreen(Some(monitor.clone()));
                    } else {
                        log::debug!("`current_monitor` is not set!??");
                    }
                } else {
                    log::info!("[{}] Switching fullscreen off", current_frame_id);
                    gl_window.set_resizable(true);
                    gl_window.set_fullscreen(None);
                    let pos = gl_window.get_position();
                    log::debug!("New window position: {:?}", pos);
                    gl_window.set_decorations(true);
                    switched_from_fullscreen = true;
                }
            }
        }

        if previous_settings.font_size != settings.font_size {
            change_tilesize(
                settings.font_size as u32,
                &mut tilesize,
                &mut display,
                &mut settings,
                &mut desired_window_width,
                &mut desired_window_height,
            );
            gl_window.set_resizable(true);
            if !settings.fullscreen {
                let size: LogicalSize = (desired_window_width, desired_window_height).into();
                gl_window.set_inner_size(size);
                gl_window.resize(size.to_physical(gl_window.get_hidpi_factor()));
            }
        }

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

        if current_frame_id == 1 {
            // NOTE: We should have the proper window position and
            // monitor info at this point but not sooner.

            // NOTE: If the primary monitor is different from the
            // monitor the window actually spawns at (this happens on
            // my dev machine where the primary monitor is in the
            // portrait orientation and therefore more narrow, but the
            // game window normally spawns on my landscape monitor),
            // it gets resized. We can detect it because this event
            // fires on the first frame. So we ask it to resize to the
            // expected size again and leave it at that.
            log::debug!(
                "Current monitor: {:?}",
                current_monitor.as_ref().map(|m| m.get_dimensions())
            );

            if desired_window_width != window_size_px.x as u32
                || desired_window_height != window_size_px.y as u32
            {
                if let Some(ref monitor) = current_monitor {
                    let dim = monitor.get_dimensions();
                    let monitor_width = dim.width as u32;
                    let monitor_height = dim.height as u32;
                    if desired_window_width <= monitor_width
                        && desired_window_height <= monitor_height
                    {
                        log::debug!(
                            "Resetting the window to its expected size: {} x {}.",
                            desired_window_width,
                            desired_window_height
                        );
                        gl_window.set_inner_size(LogicalSize {
                            width: desired_window_width.into(),
                            height: desired_window_height.into(),
                        });
                    } else {
                        log::debug!("TODO: try to resize but maintain aspect ratio.");
                    }
                }
            }
        }

        // If we just switched from fullscreen back to a windowed
        // mode, restore the window position we had before. We do this
        // because the `Moved` event fires with an incorrect value
        // when coming back from full screen.
        //
        // This ensures that we can switch full screen back and fort
        // on a multi monitor setup.
        if switched_from_fullscreen {
            window_pos = pre_fullscreen_window_pos;
        }

        gl_window.set_resizable(false);
    }

    log::debug!(
        "Drawcall count: {}. Capacity: {}.",
        overall_max_drawcall_count,
        engine::DRAWCALL_CAPACITY
    );
}
