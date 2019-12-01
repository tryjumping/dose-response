use crate::{
    color::Color,
    engine::{
        self,
        loop_state::{LoopState, ResizeWindowAction, UpdateResult},
        TextMetrics,
    },
    keys::{Key, KeyCode},
    point::Point,
    settings::Store as SettingsStore,
    state::State,
};

use std::time::Instant;

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::{self, Keycode as BackendKey},
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
    initial_game_display_size: Point,
    initial_default_background: Color,
    window_title: &str,
    mut settings_store: S,
    initial_state: Box<State>,
) where
    S: SettingsStore,
{
    let mut loop_state = LoopState::initialise(
        settings_store.load(),
        initial_game_display_size,
        initial_default_background,
        initial_state,
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
        .window(
            window_title,
            loop_state.desired_window_size_px().0,
            loop_state.desired_window_size_px().1,
        )
        .resizable()
        .opengl()
        .position_centered()
        .build()
        .expect("SDL window creation failed.");

    let _ctx = window
        .gl_create_context()
        .expect("SDL GL context creation failed.");
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    log::debug!("Loaded OpenGL symbols.");

    let opengl_app = loop_state.opengl_app();

    // TODO: we're hardcoding it now because that's what we always did for SDL.
    // There's probably a method to read/handle this proper.
    let dpi = 1.0;

    let mut event_pump = sdl_context
        .event_pump()
        .expect("SDL event pump creation failed.");

    let mut previous_frame_start_time = Instant::now();

    let mut running = true;
    while running {
        let frame_start_time = Instant::now();
        let dt = frame_start_time.duration_since(previous_frame_start_time);
        previous_frame_start_time = frame_start_time;

        loop_state.update_fps(dt);

        for event in event_pump.poll_iter() {
            log::debug!("{:?}", event);
            match event {
                Event::Quit { .. } => {
                    running = false;
                }

                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    log::info!("Window resized to: {}x{}", width, height);
                    loop_state.handle_window_size_changed(width, height);
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
                        let key = Key {
                            code,
                            alt: keymod.intersects(keyboard::Mod::LALTMOD | keyboard::Mod::RALTMOD),
                            ctrl: keymod
                                .intersects(keyboard::Mod::LCTRLMOD | keyboard::Mod::RCTRLMOD),
                            shift: keymod
                                .intersects(keyboard::Mod::LSHIFTMOD | keyboard::Mod::RSHIFTMOD),
                        };
                        log::debug!("Detected key {:?}", key);
                        loop_state.keys.push(key);
                    }
                }

                Event::TextInput { text, .. } => {
                    if text.contains('?') {
                        let key = Key {
                            code: KeyCode::QuestionMark,
                            alt: false,
                            ctrl: false,
                            shift: false,
                        };
                        log::debug!("Detected key {:?}", key);
                        loop_state.keys.push(key);
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    loop_state.update_mouse_position(dpi, x, y);
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    match mouse_btn {
                        Left => {
                            loop_state.mouse.left_is_down = true;
                        }
                        Right => {
                            loop_state.mouse.right_is_down = true;
                        }
                        _ => {}
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    match mouse_btn {
                        Left => {
                            loop_state.mouse.left_clicked = true;
                            loop_state.mouse.left_is_down = false;
                        }
                        Right => {
                            loop_state.mouse.right_clicked = true;
                            loop_state.mouse.right_is_down = false;
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        match loop_state.update_game(dt, &mut settings_store) {
            UpdateResult::QuitRequested => break,
            UpdateResult::KeepGoing => {}
        }

        if cfg!(feature = "fullscreen") {
            use engine::loop_state::FullscreenAction::*;
            use sdl2::video::FullscreenType::*;
            match loop_state.fullscreen_action() {
                Some(SwitchToFullscreen) => {
                    if let Err(err) = window.set_fullscreen(Desktop) {
                        log::warn!(
                            "[{}]: Could not switch to fullscreen:",
                            loop_state.current_frame_id
                        );
                        log::warn!("{:?}", err);
                    }
                }
                Some(SwitchToWindowed) => {
                    if let Err(err) = window.set_fullscreen(Off) {
                        log::warn!(
                            "[{}]: Could not leave fullscreen:",
                            loop_state.current_frame_id
                        );
                        log::warn!("{:?}", err);
                    }
                }
                None => {}
            }
        }

        match loop_state.check_window_size_needs_updating() {
            ResizeWindowAction::NewSize((width, height)) => {
                if let Err(err) = window.set_size(width, height) {
                    log::warn!("[{}] Could not resize window:", loop_state.current_frame_id);
                    log::warn!("{:?}", err);
                }
                loop_state.handle_window_size_changed(width as i32, height as i32);
            }
            ResizeWindowAction::NoChange => {}
        }

        loop_state.process_vertices_and_render(&opengl_app, dpi);
        window.gl_swap_window();

        loop_state.previous_settings = loop_state.settings.clone();
    }

    log::debug!(
        "Drawcall count: {}. Capacity: {}.",
        loop_state.overall_max_drawcall_count,
        engine::DRAWCALL_CAPACITY
    );
}
