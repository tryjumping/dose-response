use crate::{
    color::Color,
    engine::{
        self,
        loop_state::{LoopState, ResizeWindowAction, UpdateResult},
    },
    keys::{Key, KeyCode},
    point::Point,
    settings::Store as SettingsStore,
    state::State,
};

use std::time::Instant;

use glutin::{
    dpi::{LogicalPosition, LogicalSize},
    ElementState, KeyboardInput, MonitorId, VirtualKeyCode as BackendKey,
};

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

        // NOTE: these keys trigger on the numpad when NumLock is off.
        // We will translate them back to the appropriate numpad keys:
        BackendKey::Home => Some(KeyCode::NumPad7),
        BackendKey::End => Some(KeyCode::NumPad1),
        BackendKey::PageUp => Some(KeyCode::NumPad9),
        BackendKey::PageDown => Some(KeyCode::NumPad3),

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

#[allow(cyclomatic_complexity, unsafe_code)]
pub fn main_loop<S>(
    initial_game_display_size: Point,
    initial_default_background: Color,
    window_title: &str,
    mut settings_store: S,
    initial_state: Box<State>,
) where
    S: SettingsStore,
{
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

    let mut loop_state = LoopState::initialise(
        settings_store.load(),
        initial_game_display_size,
        initial_default_background,
        initial_state,
    );

    let mut events_loop = glutin::EventsLoop::new();
    log::debug!("Created events loop: {:?}", events_loop);
    let window = glutin::WindowBuilder::new()
        .with_title(window_title)
        .with_dimensions(loop_state.desired_window_size_px().into());
    log::debug!("Created window builder: {:?}", window);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window, &events_loop);
    log::debug!("Created context: {:?}", context);
    let context = match context {
        Ok(context) => context,
        Err(error) => {
            log::error!("Could not create context: {:?}", error);
            panic!("Aborting!");
        }
    };

    log::debug!("Making context current.");
    let context = unsafe {
        match context.make_current() {
            Ok(context) => context,
            Err(error) => {
                log::error!("Could not make context current: {:?}", error);
                panic!("Aborting!");
            }
        }
    };

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
    log::debug!("Loaded OpenGL symbols.");

    let opengl_app = loop_state.opengl_app();

    let dpi = context.window().get_hidpi_factor();
    log::info!("Window HIDPI factor: {:?}", dpi);
    match context.window().get_inner_size() {
        Some(logical_size) => {
            log::info!("Window inner size (logical): {:?}", logical_size);
            log::info!(
                "Window inner size (physical): {:?}",
                logical_size.to_physical(dpi)
            );
        }
        None => log::warn!("Window inner size is `None`."),
    }

    match context.window().get_outer_size() {
        Some(logical_size) => {
            log::info!("Window outer size (logical): {:?}", logical_size);
            log::info!(
                "Window outer size (physical): {:?}",
                logical_size.to_physical(dpi)
            );
        }
        None => log::warn!("Window outer size is `None`."),
    }

    // We'll just assume the monitors won't change throughout the game.
    let monitors: Vec<_> = events_loop.get_available_monitors().collect();
    log::debug!("Got all available monitors: {:?}", monitors);

    let mut window_pos = {
        match context.window().get_position() {
            Some(LogicalPosition { x, y }) => Point::new(x as i32, y as i32),
            None => Default::default(),
        }
    };
    log::info!("Window pos: {:?}", window_pos);
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
    log::info!(
        "Current monitor: {:?}, pos: {:?}, size: {:?}",
        current_monitor.as_ref().map(|m| m.get_name()),
        current_monitor.as_ref().map(|m| m.get_position()),
        current_monitor.as_ref().map(|m| m.get_dimensions())
    );

    let mut previous_frame_start_time = Instant::now();

    let mut running = true;
    while running {
        let frame_start_time = Instant::now();
        let dt = frame_start_time.duration_since(previous_frame_start_time);
        previous_frame_start_time = frame_start_time;

        loop_state.update_fps(dt);

        events_loop.poll_events(|event| {
            log::debug!("{:?}", event);
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,

                    glutin::WindowEvent::Resized(size) => {
                        let LogicalSize { width, height } = size;
                        log::info!("WindowEvent::Resized: {:?}", size);

                        if let Some(monitor_id) = context.window().get_fullscreen() {
                            log::warn!(
                                "Asked to resize on fullscreen: target size: {:?}, monitor ID: {:?}. Ignoring this request.",
                                size,
                                monitor_id);
                        }

                        context.resize(size.to_physical(context.window().get_hidpi_factor()));
                        loop_state.handle_window_size_changed(width as i32, height as i32);
                    }

                    glutin::WindowEvent::Moved(new_pos) => {
                        if loop_state.settings.fullscreen || loop_state.switched_from_fullscreen {
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
                                loop_state.current_frame_id,
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
                            let key = Key {
                                code,
                                alt: modifiers.alt,
                                ctrl: modifiers.ctrl,
                                shift: modifiers.shift,
                                logo: modifiers.logo,
                            };
                            log::debug!("Detected key {:?}", key);
                            loop_state.keys.push(key);
                        }
                    }

                    glutin::WindowEvent::ReceivedCharacter(chr) => {
                        log::debug!("Received character: {:?}", chr);
                        if chr == '?' {
                            let key = Key {
                                code: KeyCode::QuestionMark,
                                alt: false,
                                ctrl: false,
                                shift: false,
                                logo: false,
                            };
                            log::debug!("Detected key {:?}", key);
                            loop_state.keys.push(key);
                        }
                    }

                    glutin::WindowEvent::CursorMoved { position, .. } => {
                        loop_state.update_mouse_position(dpi, position.x as i32, position.y as i32);
                    }

                    glutin::WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button,
                        ..
                    } => {
                        use glutin::MouseButton::*;
                        match button {
                            Left => {
                                loop_state.mouse.left_is_down = true;
                            }
                            Right => {
                                loop_state.mouse.right_is_down = true;
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

                    _ => (),
                },
                _ => (),
            }
        });

        match loop_state.update_game(dt, &mut settings_store) {
            UpdateResult::QuitRequested => break,
            UpdateResult::KeepGoing => {}
        }

        if cfg!(feature = "fullscreen") {
            use engine::loop_state::FullscreenAction::*;
            match loop_state.fullscreen_action() {
                Some(SwitchToFullscreen) => {
                    if let Some(ref monitor) = current_monitor {
                        pre_fullscreen_window_pos = window_pos;
                        log::info!(
                            "Monitor: {:?}, pos: {:?}, dimensions: {:?}",
                            monitor.get_name(),
                            monitor.get_position(),
                            monitor.get_dimensions()
                        );
                        // TODO: let's see if we need to set
                        // decorations explicitly. Remove this line if
                        // we don't actually need it.
                        //context.window().set_decorations(false);
                        context.window().set_fullscreen(Some(monitor.clone()));
                    } else {
                        log::warn!("`current_monitor` is not set!??");
                    }
                }
                Some(SwitchToWindowed) => {
                    let window = context.window();
                    window.set_fullscreen(None);
                    let pos = window.get_position();
                    log::info!("New window position: {:?}", pos);
                    window.set_decorations(true);
                    loop_state.switched_from_fullscreen = true;
                }
                None => {}
            };
        }

        match loop_state.check_window_size_needs_updating() {
            ResizeWindowAction::NewSize(desired_window_size_px) => {
                log::info!("Updating window to new size: {:?}", desired_window_size_px);
                let window = context.window();
                let size: LogicalSize = desired_window_size_px.into();
                context.resize(size.to_physical(window.get_hidpi_factor()));
            }
            ResizeWindowAction::NoChange => {}
        }

        loop_state.process_vertices_and_render(&opengl_app, dpi);
        context.swap_buffers().unwrap();

        loop_state.previous_settings = loop_state.settings.clone();

        if loop_state.current_frame_id == 1 {
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
            log::info!(
                "Current monitor size: {:?}",
                current_monitor.as_ref().map(|m| m.get_dimensions())
            );
        }

        // If we just switched from fullscreen back to a windowed
        // mode, restore the window position we had before. We do this
        // because the `Moved` event fires with an incorrect value
        // when coming back from full screen.
        //
        // This ensures that we can switch full screen back and fort
        // on a multi monitor setup.
        if loop_state.switched_from_fullscreen {
            window_pos = pre_fullscreen_window_pos;
        }
    }

    log::info!(
        "Drawcall count: {}. Capacity: {}.",
        loop_state.overall_max_drawcall_count,
        engine::DRAWCALL_CAPACITY
    );
}
