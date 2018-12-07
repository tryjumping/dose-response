use crate::{
    color::Color,
    engine::{self, Mouse, Settings, TextMetrics, UpdateFn, Vertex},
    game::RunningState,
    keys::{Key, KeyCode},
    point::Point,
    rect::Rectangle,
    state::State,
    util,
};

use std::time::{Duration, Instant};

use glium::{
    self,
    draw_parameters::DrawParameters,
    glutin::VirtualKeyCode as BackendKey,
    glutin::{Event, EventsLoop, MonitorId, WindowBuilder, WindowEvent},
    program, uniform, Surface,
};

use image;

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

struct Metrics {
    tile_width_px: i32,
}

impl TextMetrics for Metrics {
    fn tile_width_px(&self) -> i32 {
        self.tile_width_px
    }
}

#[allow(unsafe_code)]
mod vertex {
    use super::Vertex;
    glium::implement_vertex!(Vertex, pos_px, tile_pos_px, color);
}

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
    // using `winit 0.16.2` since that's the latest thing supported by
    // the latest glium (0.22). Glium always lags behind, so maybe the
    // best way to test this is to drop it in favour of glutin+gl
    // which tends to use the latest winit pretty quickly.
    //
    // Here are the current issues under wayland:
    // 1. The window decorations look different from the rest of the system
    // 2. The full screen leaves the window's top bar in
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
    let mut window_width = desired_window_width;
    let mut window_height = desired_window_height;

    // GL setup

    let mut events_loop = EventsLoop::new();

    // We'll just assume the monitors won't change throughout the game.
    let monitors: Vec<_> = events_loop.get_available_monitors().collect();

    let desired_dimensions = glium::glutin::dpi::PhysicalSize {
        width: desired_window_width.into(),
        height: desired_window_height.into(),
    };

    let window = WindowBuilder::new()
        .with_title(window_title)
        // NOTE: ensure a DPI 1.0 factor, otherwise the game is not
        // pixel-perfect. Which looks bad.
        .with_dimensions(desired_dimensions.to_logical(1.0));

    let context = glium::glutin::ContextBuilder::new().with_vsync(true);

    let display = glium::Display::new(window, context, &events_loop)
        .expect("dose response ERROR: Could not create the display.");

    let program = program!(&display,
                       150 => {
                           outputs_srgb: true,
                           vertex: include_str!("../shader_150.glslv"),
                           fragment: include_str!("../shader_150.glslf")
                       }
    )
    .unwrap();

    let texture = {
        use std::io::Cursor;
        let data = &include_bytes!(concat!(env!("OUT_DIR"), "/font.png"))[..];
        let image = image::load(Cursor::new(data), image::PNG)
            .unwrap()
            .to_rgba();
        let (w, h) = image.dimensions();
        assert_eq!(w % tilesize, 0);
        assert_eq!(h % tilesize, 0);
        let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), (w, h));
        glium::texture::SrgbTexture2d::new(&display, image).unwrap()
    };

    let (tex_width_px, tex_height_px) =
        (texture.dimensions().0 as f32, texture.dimensions().1 as f32);

    // Main loop
    let mut window_pos = {
        match display.gl_window().get_position() {
            Some(glium::glutin::dpi::LogicalPosition { x, y }) => Point::new(x as i32, y as i32),
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
    let mut settings = Settings { fullscreen: false };
    let mut engine_display = engine::Display::new(
        display_size,
        Point::from_i32(display_size.y / 2),
        tilesize as i32,
    );
    let mut lctrl_pressed = false;
    let mut rctrl_pressed = false;
    let mut lalt_pressed = false;
    let mut ralt_pressed = false;
    let mut lshift_pressed = false;
    let mut rshift_pressed = false;
    let mut drawcalls = Vec::with_capacity(engine::DRAWCALL_CAPACITY);
    let mut vertices: Vec<Vertex> = Vec::with_capacity(engine::VERTEX_CAPACITY);
    let mut keys = vec![];
    let mut previous_frame_time = Instant::now();
    let mut fps_clock = Duration::from_millis(0);
    let mut frame_counter = 0;
    let mut fps = 1;
    let mut running = true;
    // NOTE: This will wrap after running continuously for over 64
    // years at 60 FPS. 32 bits are just fine.
    let mut current_frame: i32 = 0;

    while running {
        let now = Instant::now();
        let dt = now.duration_since(previous_frame_time);
        previous_frame_time = now;

        // Calculate FPS
        fps_clock = fps_clock + dt;
        frame_counter += 1;
        current_frame += 1;
        if util::num_milliseconds(fps_clock) > 1000 {
            fps = frame_counter;
            frame_counter = 1;
            fps_clock = Duration::from_millis(0);
        }

        let previous_settings = settings;

        // NOTE: Skip the first frame -- the window isn't set up
        // properly there.
        if current_frame > 1 {
            engine_display.draw_rectangle(
                Rectangle::from_point_and_size(Point::new(0, 0), display_size),
                default_background,
            );
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
                &mut engine_display,
            );

            match update_result {
                RunningState::Running => {}
                RunningState::NewGame(new_state) => {
                    state = new_state;
                }
                RunningState::Stopped => break,
            }
        }

        mouse.left_clicked = false;
        mouse.right_clicked = false;

        keys.clear();

        let mut switched_from_fullscreen = false;

        if cfg!(feature = "fullscreen") {
            if previous_settings.fullscreen != settings.fullscreen {
                if settings.fullscreen {
                    log::info!("Switching to fullscreen.");
                    display.gl_window().set_decorations(false);
                    if let Some(ref monitor) = current_monitor {
                        pre_fullscreen_window_pos = window_pos;
                        log::debug!(
                            "Monitor: {:?}, pos: {:?}, dimensions: {:?}",
                            monitor.get_name(),
                            monitor.get_position(),
                            monitor.get_dimensions()
                        );
                        display.gl_window().set_fullscreen(Some(monitor.clone()));
                    } else {
                        log::debug!("`current_monitor` is not set!??");
                    }
                } else {
                    log::info!("Switched from fullscreen.");
                    display.gl_window().set_fullscreen(None);
                    let pos = display.gl_window().get_position();
                    log::debug!("New window position: {:?}", pos);
                    display.gl_window().set_decorations(true);
                    switched_from_fullscreen = true;
                }
            }
        }

        let display_info = engine::calculate_display_info(
            [window_width as f32, window_height as f32],
            display_size,
            tilesize,
        );

        // Process drawcalls
        drawcalls.clear();
        engine_display.push_drawcalls(&mut drawcalls);

        vertices.clear();
        engine::build_vertices(&drawcalls, &mut vertices, display_info.native_display_px);

        if vertices.len() > engine::VERTEX_CAPACITY {
            log::warn!(
                "Warning: vertex count exceeded initial capacity {}. Current count: {} ",
                vertices.len(),
                engine::VERTEX_CAPACITY
            );
        }

        let vertex_buffer = glium::VertexBuffer::new(&display, &vertices).unwrap();

        // TODO: Once we support multiple font sizes, we can adjust it
        // here. We could also potentially only allow resizes in steps
        // that would result in crisp text (i.e. no font resizing on
        // the GL side).

        let uniforms = uniform! {
            tex: &texture,
            // TODO: pass this from the block above
            native_display_px: display_info.native_display_px,
            display_px: display_info.display_px,
            extra_px: display_info.extra_px,
            texture_size_px: [tex_width_px, tex_height_px],
        };

        // Render
        let mut target = display.draw();
        target.clear_color_srgb(0.1, 0.0, 0.1, 1.0);
        target
            .draw(
                &vertex_buffer,
                &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &program,
                &uniforms,
                &DrawParameters {
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
        target.finish().unwrap();

        // Process events
        events_loop.poll_events(|ev| {
            //debug!("{:?}", ev);
            match ev {
                Event::WindowEvent {
                    window_id: _,
                    event,
                } => {
                    match event {
                        WindowEvent::CloseRequested => running = false,
                        WindowEvent::Resized(new_size) => {
                            log::debug!(
                                "[FRAME {}] Window resized to: {:?}",
                                current_frame,
                                new_size
                            );
                            window_width = new_size.width as u32;
                            window_height = new_size.height as u32;
                        }
                        WindowEvent::Moved(new_pos) => {
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
                                    current_frame,
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
                        WindowEvent::ReceivedCharacter(chr) => {
                            let code = match chr {
                                '?' => Some(KeyCode::QuestionMark),
                                _ => None,
                            };
                            if let Some(code) = code {
                                keys.push(Key {
                                    code: code,
                                    alt: false,
                                    ctrl: false,
                                    shift: false,
                                });
                            }
                        }
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            input,
                        } => {
                            use glium::glutin::ElementState::*;
                            let pressed = match input.state {
                                Pressed => true,
                                Released => false,
                            };

                            // debug!("KeyboardInput event!");
                            // debug!("{:?}", input);

                            // TODO: this is a temp fix for a
                            // glutin/winit bug where the keypress
                            // release event for the Shift keys has
                            // its `virtual_keycode` set to `None`
                            // instead of `Some(LShift)`. But the
                            // scancodes still work so we'll use them
                            // instead for now.
                            // It's a winit issue:
                            // https://github.com/tomaka/winit/issues/361
                            if input.scancode == 42 && !pressed {
                                lshift_pressed = false;
                            }
                            if input.scancode == 54 && !pressed {
                                rshift_pressed = false;
                            }

                            match input.virtual_keycode {
                                Some(BackendKey::LControl) => {
                                    lctrl_pressed = pressed;
                                }
                                Some(BackendKey::RControl) => {
                                    rctrl_pressed = pressed;
                                }
                                Some(BackendKey::LAlt) => {
                                    lalt_pressed = pressed;
                                }
                                Some(BackendKey::RAlt) => {
                                    ralt_pressed = pressed;
                                }
                                Some(BackendKey::LShift) => {
                                    lshift_pressed = pressed;
                                }
                                Some(BackendKey::RShift) => {
                                    rshift_pressed = pressed;
                                }
                                Some(key_code) => {
                                    if pressed {
                                        if let Some(code) = key_code_from_backend(key_code) {
                                            let key = Key {
                                                code: code,
                                                alt: lalt_pressed
                                                    || ralt_pressed
                                                    || input.modifiers.alt,
                                                ctrl: lctrl_pressed
                                                    || rctrl_pressed
                                                    || input.modifiers.ctrl,
                                                shift: lshift_pressed
                                                    || rshift_pressed
                                                    || input.modifiers.shift,
                                            };
                                            // debug!("Pushing {:?}", key);
                                            keys.push(key);
                                        }
                                    }
                                }
                                None => {
                                    let code = match input.scancode {
                                        79 => Some(KeyCode::NumPad7),
                                        80 => Some(KeyCode::NumPad8),
                                        81 => Some(KeyCode::NumPad9),
                                        83 => Some(KeyCode::NumPad4),
                                        84 => Some(KeyCode::NumPad5),
                                        85 => Some(KeyCode::NumPad6),
                                        87 => Some(KeyCode::NumPad1),
                                        88 => Some(KeyCode::NumPad2),
                                        89 => Some(KeyCode::NumPad3),
                                        _ => None,
                                    };
                                    if pressed {
                                        if let Some(code) = code {
                                            let key = Key {
                                                code: code,
                                                alt: lalt_pressed
                                                    || ralt_pressed
                                                    || input.modifiers.alt,
                                                ctrl: lctrl_pressed
                                                    || rctrl_pressed
                                                    || input.modifiers.ctrl,
                                                shift: lshift_pressed
                                                    || rshift_pressed
                                                    || input.modifiers.shift,
                                            };
                                            // debug!("Pushing {:?}", key);
                                            keys.push(key);
                                        }
                                    }
                                }
                            }
                        }
                        WindowEvent::CursorMoved {
                            position: cursor_pos,
                            ..
                        } => {
                            // debug!("Extra px: {:?}", extra_px);
                            // debug!("Display px: {:?}", display_px);
                            // debug!("Native display px: {:?}", native_display_px);
                            // debug!("screen width/height: {:?}", (screen_width, screen_height));
                            let (x, y) = (cursor_pos.x as i32, cursor_pos.y as i32);

                            let (x, y) = (
                                x - (display_info.extra_px[0] / 2.0) as i32,
                                y - (display_info.extra_px[1] / 2.0) as i32,
                            );
                            let x = util::clamp(0, x, display_info.display_px[0] as i32 - 1);
                            let y = util::clamp(0, y, display_info.display_px[1] as i32 - 1);

                            mouse.screen_pos = Point { x, y };

                            let tile_width = display_info.display_px[0] as i32 / display_size.x;
                            let mouse_tile_x = x / tile_width;

                            let tile_height = display_info.display_px[1] as i32 / display_size.y;
                            let mouse_tile_y = y / tile_height;

                            mouse.tile_pos = Point {
                                x: mouse_tile_x,
                                y: mouse_tile_y,
                            };
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            use glium::glutin::ElementState::*;
                            use glium::glutin::MouseButton::*;

                            match (state, button) {
                                (Pressed, Left) => {
                                    mouse.left_is_down = true;
                                }
                                (Pressed, Right) => {
                                    mouse.right_is_down = true;
                                }
                                (Released, Left) => {
                                    mouse.left_clicked = true;
                                    mouse.left_is_down = false;
                                }
                                (Released, Right) => {
                                    mouse.right_clicked = true;
                                    mouse.right_is_down = false;
                                }
                                _ => {}
                            }
                        }
                        WindowEvent::Focused(false) => {
                            lctrl_pressed = false;
                            rctrl_pressed = false;
                            lalt_pressed = false;
                            ralt_pressed = false;
                            lshift_pressed = false;
                            rshift_pressed = false;
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        });

        if current_frame == 1 {
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

            if desired_window_width != window_width || desired_window_height != window_height {
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
                        display
                            .gl_window()
                            .set_inner_size(glium::glutin::dpi::LogicalSize {
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
    }
}
