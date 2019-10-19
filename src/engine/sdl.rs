use crate::{
    color::Color,
    engine::{self, Drawcall, Mouse, Settings, SettingsStore, TextMetrics, UpdateFn, Vertex},
    game::RunningState,
    keys::KeyCode,
    point::Point,
    state::State,
    util,
};

use std::{
    mem,
    time::{Duration, Instant},
};

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::{self, Keycode as BackendKey},
};

// const DESIRED_FPS: u64 = 60;
// const EXPECTED_FRAME_LENGTH: Duration = Duration::from_millis(1000 / DESIRED_FPS);

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

#[allow(cyclomatic_complexity)]
pub fn main_loop<S>(
    display_size: Point,
    default_background: Color,
    window_title: &str,
    mut settings_store: S,
    mut state: Box<State>,
    update: UpdateFn,
) where
    S: SettingsStore,
{
    let mut settings = settings_store.load();
    let (desired_window_width, desired_window_height) = (
        display_size.x as u32 * settings.tile_size as u32,
        display_size.y as u32 * settings.tile_size as u32,
    );

    let sdl_context = sdl2::init().expect("SDL context creation failed.");
    let video_subsystem = sdl_context
        .video()
        .expect("SDL video subsystem creation failed.");

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_double_buffer(true);
    gl_attr.set_depth_size(0);

    // NOTE: add `.fullscreen_desktop()` to start in fullscreen.
    let mut window = video_subsystem
        .window(window_title, desired_window_width, desired_window_height)
        .resizable()
        .opengl()
        .position_centered()
        .build()
        .expect("SDL window creation failed.");

    let _ctx = window
        .gl_create_context()
        .expect("SDL GL context creation failed.");
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

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
    let sdl_app = engine::opengl::OpenGlApp::new(vs_source, fs_source);
    sdl_app.initialise(image.dimensions(), &image);

    let mut event_pump = sdl_context
        .event_pump()
        .expect("SDL event pump creation failed.");

    let mut mouse = Mouse::new();
    let mut window_size_px = Point::new(desired_window_width as i32, desired_window_height as i32);
    let mut display = engine::Display::new(
        display_size,
        Point::from_i32(display_size.y / 2),
        settings.tile_size,
    );
    let mut drawcalls: Vec<Drawcall> = Vec::with_capacity(engine::DRAWCALL_CAPACITY);
    assert_eq!(mem::size_of::<Vertex>(), engine::VERTEX_COMPONENT_COUNT * 4);
    let mut vertex_buffer: Vec<f32> = Vec::with_capacity(engine::VERTEX_BUFFER_CAPACITY);
    let mut overall_max_drawcall_count = 0;
    let mut keys = vec![];
    let mut previous_frame_start_time = Instant::now();
    // Always stard from a windowed mode. This will force the
    // fullscreen switch in the first frame if requested in the
    // settings we've loaded.
    //
    // This is necessary because some backends don't support
    // fullscreen on window creation. And TBH, this is easier on us
    // because it means there's only one fullscreen-handling pathway.
    let mut previous_settings = Settings {
        fullscreen: false,
        ..settings.clone()
    };
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

        for event in event_pump.poll_iter() {
            log::debug!("{:?}", event);
            match event {
                Event::Quit { .. } => {
                    running = false;
                }

                Event::KeyDown {
                    keycode: Some(backend_code),
                    scancode,
                    keymod,
                    ..
                } => {
                    log::debug!(
                        "KeyDown backend_code: {:?}, scancode: {:?}, keymod bits: {:?}",
                        backend_code,
                        scancode,
                        keymod.bits(),
                    );
                    if let Some(code) = key_code_from_backend(backend_code) {
                        let key = super::Key {
                            code,
                            alt: keymod.intersects(keyboard::Mod::LALTMOD | keyboard::Mod::RALTMOD),
                            ctrl: keymod
                                .intersects(keyboard::Mod::LCTRLMOD | keyboard::Mod::RCTRLMOD),
                            shift: keymod
                                .intersects(keyboard::Mod::LSHIFTMOD | keyboard::Mod::RSHIFTMOD),
                        };
                        log::debug!("Detected key {:?}", key);
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
                        log::debug!("Detected key {:?}", key);
                        keys.push(key);
                    }
                }

                Event::MouseMotion { x, y, .. } => {
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

                Event::MouseButtonDown { mouse_btn, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    match mouse_btn {
                        Left => {
                            mouse.left_is_down = true;
                        }
                        Right => {
                            mouse.right_is_down = true;
                        }
                        _ => {}
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    match mouse_btn {
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

                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    log::info!("Window resized to: {}x{}", width, height);
                    window_size_px = Point::new(width, height);
                }

                _ => {}
            }
        }

        let tile_width_px = settings.tile_size;
        let update_result = update(
            &mut state,
            dt,
            display_size,
            fps,
            &keys,
            mouse,
            &mut settings,
            &Metrics { tile_width_px },
            &mut settings_store,
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
            use sdl2::video::FullscreenType::*;
            if previous_settings.fullscreen != settings.fullscreen {
                if settings.fullscreen {
                    log::info!(
                        "[{}] Switching to (desktop-type) fullscreen",
                        current_frame_id
                    );
                    if let Err(err) = window.set_fullscreen(Desktop) {
                        log::warn!("[{}]: Could not switch to fullscreen:", current_frame_id);
                        log::warn!("{:?}", err);
                    }
                } else {
                    log::info!("[{}] Switching fullscreen off", current_frame_id);
                    if let Err(err) = window.set_fullscreen(Off) {
                        log::warn!("[{}]: Could not leave fullscreen:", current_frame_id);
                        log::warn!("{:?}", err);
                    }
                }
            }
        }

        // debug!("Pre-draw duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

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
            settings.tile_size,
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

        // debug!("Pre-present duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        // NOTE: render

        sdl_app.render(
            default_background,
            display_info,
            [image_width as f32, image_height as f32],
            &vertex_buffer,
        );
        window.gl_swap_window();

        previous_settings = settings.clone();

        // debug!("Code duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        // if let Some(sleep_duration) = EXPECTED_FRAME_LENGTH.checked_sub(frame_start_time.elapsed()) {
        //     ::std::thread::sleep(sleep_duration);
        // };

        // debug!("Total frame duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);
    }

    log::debug!(
        "Drawcall count: {}. Capacity: {}.",
        overall_max_drawcall_count,
        engine::DRAWCALL_CAPACITY
    );
}
