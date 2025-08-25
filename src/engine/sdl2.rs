use crate::{
    color::Color,
    engine::{
        self,
        loop_state::{self, LoopState, ResizeWindowAction, UpdateResult},
        opengl::OpenGlApp,
    },
    keys::{Key, KeyCode},
    settings::Store as SettingsStore,
    state::State,
};

use std::time::{Duration, Instant};

use egui::{
    Context,
    epaint::{ClippedPrimitive, ClippedShape},
};

use rodio::OutputStream;

use sdl2::{
    EventPump,
    event::{Event, WindowEvent},
    keyboard::{self, Keycode as BackendKey},
    video::Window,
};

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

struct Game<S> {
    loop_state: LoopState,
    event_pump: EventPump,
    dpi: f64,
    window: Window,
    opengl_app: OpenGlApp,
    egui_shapes: Vec<ClippedShape>,
    ui_paint_batches: Vec<ClippedPrimitive>,
    settings_store: S,
}

impl<S: SettingsStore> Game<S> {
    fn update_and_render(&mut self, dt: Duration) -> bool {
        self.loop_state.update_fps(dt);

        for event in self.event_pump.poll_iter() {
            log::debug!("{:?}", event);
            match event {
                Event::Quit { .. } => {
                    return false;
                }

                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    log::info!("Window resized to: {}x{}", width, height);
                    self.loop_state.handle_window_size_changed(width, height);
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
                            logo: keymod
                                .intersects(keyboard::Mod::LGUIMOD | keyboard::Mod::RGUIMOD),
                        };
                        log::debug!("Detected key {:?}", key);
                        self.loop_state.keys.push(key);
                    }
                }

                Event::TextInput { text, .. } => {
                    if text.contains('?') {
                        let key = Key {
                            code: KeyCode::QuestionMark,
                            alt: false,
                            ctrl: false,
                            shift: false,
                            logo: false,
                        };
                        log::debug!("Detected key {:?}", key);
                        self.loop_state.keys.push(key);
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    self.loop_state.update_mouse_position(self.dpi, x, y);
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    match mouse_btn {
                        Left => {
                            self.loop_state.mouse.left_is_down = true;
                        }
                        Right => {
                            self.loop_state.mouse.right_is_down = true;
                        }
                        _ => {}
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    match mouse_btn {
                        Left => {
                            self.loop_state.mouse.left_clicked = true;
                            self.loop_state.mouse.left_is_down = false;
                        }
                        Right => {
                            self.loop_state.mouse.right_clicked = true;
                            self.loop_state.mouse.right_is_down = false;
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        self.loop_state
            .egui_context
            .begin_pass(self.loop_state.egui_raw_input());

        match self.loop_state.update_game(dt, &mut self.settings_store) {
            UpdateResult::QuitRequested => return false,
            UpdateResult::KeepGoing => {}
        }

        if cfg!(feature = "fullscreen") {
            use engine::loop_state::FullscreenAction::*;
            use sdl2::video::FullscreenType::*;
            match self.loop_state.fullscreen_action() {
                Some(SwitchToFullscreen) => {
                    if let Err(err) = self.window.set_fullscreen(Desktop) {
                        log::warn!(
                            "[{}]: Could not switch to fullscreen:",
                            self.loop_state.current_frame_id
                        );
                        log::warn!("{:?}", err);
                    }
                }
                Some(SwitchToWindowed) => {
                    if let Err(err) = self.window.set_fullscreen(Off) {
                        log::warn!(
                            "[{}]: Could not leave fullscreen:",
                            self.loop_state.current_frame_id
                        );
                        log::warn!("{:?}", err);
                    }
                }
                None => {}
            }
        }

        let output = self.loop_state.egui_context.end_pass();

        for command in &output.platform_output.commands {
            if let egui::OutputCommand::OpenUrl(url) = command
                && let Err(err) = webbrowser::open(&url.url)
            {
                log::warn!("Error opening URL {} in the external browser!", url.url);
                log::warn!("{}", err);
            }
        }

        self.egui_shapes = output.shapes;

        if output.textures_delta.set.is_empty() {
            // We don't need to set/update any textures
        } else {
            for (_texture_id, image_delta) in output.textures_delta.set {
                match image_delta.image {
                    egui::epaint::image::ImageData::Color(color_image) => {
                        log::warn!(
                            "Received ImageDelta::Color(ColorImage) of size: {:?}. Ignoring as we're not set up to handle this.",
                            color_image.size
                        );
                    }
                    egui::epaint::image::ImageData::Font(font_image) => {
                        log::warn!(
                            "We need to update the egui texture map FontImage of size: {:?}",
                            font_image.size
                        );
                        let font_image = loop_state::egui_font_image_apply_delta(
                            self.loop_state.font_texture.clone(),
                            image_delta.pos,
                            font_image,
                        );
                        self.loop_state.font_texture = font_image.clone();

                        let egui_texture = loop_state::build_texture_from_egui(font_image);
                        let (width, height) = egui_texture.dimensions();

                        self.opengl_app.eguimap_size_px = [width as f32, height as f32];
                        self.opengl_app.upload_texture(
                            self.opengl_app.eguimap,
                            "egui",
                            &egui_texture,
                        );
                    }
                }
            }
        }

        if output.textures_delta.free.is_empty() {
            // Don't print anything
        } else {
            // NOTE: I don't think we need to free anything.
            // We're just uploading the single egui-based
            // texture.
            log::warn!("Texture IDs to free");
            for texture_id in output.textures_delta.free {
                dbg!(texture_id);
            }
        }

        match self.loop_state.check_window_size_needs_updating() {
            ResizeWindowAction::NewSize((width, height)) => {
                if let Err(err) = self.window.set_size(width, height) {
                    log::warn!(
                        "[{}] Could not resize window:",
                        self.loop_state.current_frame_id
                    );
                    log::warn!("{:?}", err);
                }
                self.loop_state
                    .handle_window_size_changed(width as i32, height as i32);
            }
            ResizeWindowAction::NoChange => {}
        }

        self.ui_paint_batches = self
            .loop_state
            .egui_context
            .tessellate(self.egui_shapes.clone(), self.loop_state.dpi as f32);

        let (ui_vertices, batches) =
            engine::drawcalls_from_egui(&self.opengl_app, &self.ui_paint_batches);

        self.loop_state.process_vertices_and_render(
            &mut self.opengl_app,
            &ui_vertices,
            self.loop_state.dpi,
            &batches,
        );

        self.window.gl_swap_window();

        true
    }
}

pub fn main_loop<S>(
    initial_default_background: Color,
    window_title: &str,
    settings_store: S,
    initial_state: Box<State>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SettingsStore + 'static,
{
    let egui_context = Context::default();

    // NOTE: we need to store the stream to a variable here and then
    // match on a reference to it. Otherwise, it will be dropped and
    // the stream will close.
    let stream_result = OutputStream::try_default();
    let stream_handle = match &stream_result {
        Ok((_stream, stream_handle)) => Some(stream_handle),
        Err(error) => {
            log::error!("Cannot open the audio output stream: {:?}", error);
            None
        }
    };

    let loop_state = LoopState::initialise(
        settings_store.load(),
        initial_default_background,
        initial_state,
        egui_context,
        stream_handle,
    );

    let sdl_context = sdl2::init()?;
    log::info!("sdl2 video context");
    let video_subsystem = sdl_context.video()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_double_buffer(true);
    gl_attr.set_depth_size(0);

    // NOTE: add `.fullscreen_desktop()` to start in fullscreen.
    let window = video_subsystem
        .window(
            window_title,
            loop_state.desired_window_size_px().0,
            loop_state.desired_window_size_px().1,
        )
        .resizable()
        .opengl()
        .position_centered()
        .build()?;

    let _ctx = window.gl_create_context()?;
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    log::debug!("Loaded OpenGL symbols.");

    let opengl_app = loop_state.opengl_app();

    // TODO: we're hardcoding it now because that's what we always did for SDL.
    // There's probably a method to read/handle this proper.
    let dpi = 1.0;

    let event_pump = sdl_context.event_pump()?;

    let mut game = Game {
        loop_state,
        event_pump,
        dpi,
        window,
        opengl_app,
        egui_shapes: vec![],
        ui_paint_batches: vec![],
        settings_store,
    };

    let mut previous_frame_start_time = Instant::now();

    let mut running = true;

    while running {
        let frame_start_time = Instant::now();
        let dt = frame_start_time.duration_since(previous_frame_start_time);
        previous_frame_start_time = frame_start_time;

        running = game.update_and_render(dt);
    }

    log::debug!(
        "Drawcall count: {}. Capacity: {}.",
        game.loop_state.overall_max_drawcall_count,
        engine::DRAWCALL_CAPACITY
    );

    Ok(())
}
