use crate::{
    color::Color,
    engine::{
        self, Vertex,
        loop_state::{self, LoopState, ResizeWindowAction, UpdateResult},
        opengl::OpenGlApp,
    },
    formula, keys,
    point::Point,
    settings::{MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, Store as SettingsStore},
    state::State,
};

use std::{
    error::Error,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use sdl3::{event::Event, keyboard::Keycode};

use game_loop::game_loop;

use egui::{ClippedPrimitive, Context};

// fn key_code_from_backend(
//     physical_key: winit::keyboard::PhysicalKey,
//     logical_key: winit::keyboard::Key,
// ) -> Option<keys::KeyCode> {
//     use winit::keyboard::{Key as WinitKey, KeyCode as WinitKeyCode, PhysicalKey};

//     let key_code = match physical_key {
//         PhysicalKey::Code(WinitKeyCode::Enter) => Some(keys::KeyCode::Enter),
//         PhysicalKey::Code(WinitKeyCode::Escape) => Some(keys::KeyCode::Esc),
//         PhysicalKey::Code(WinitKeyCode::Space) => Some(keys::KeyCode::Space),

//         PhysicalKey::Code(WinitKeyCode::Numpad0) => Some(keys::KeyCode::D0),
//         PhysicalKey::Code(WinitKeyCode::Numpad1) => Some(keys::KeyCode::D1),
//         PhysicalKey::Code(WinitKeyCode::Numpad2) => Some(keys::KeyCode::D2),
//         PhysicalKey::Code(WinitKeyCode::Numpad3) => Some(keys::KeyCode::D3),
//         PhysicalKey::Code(WinitKeyCode::Numpad4) => Some(keys::KeyCode::D4),
//         PhysicalKey::Code(WinitKeyCode::Numpad5) => Some(keys::KeyCode::D5),
//         PhysicalKey::Code(WinitKeyCode::Numpad6) => Some(keys::KeyCode::D6),
//         PhysicalKey::Code(WinitKeyCode::Numpad7) => Some(keys::KeyCode::D7),
//         PhysicalKey::Code(WinitKeyCode::Numpad8) => Some(keys::KeyCode::D8),
//         PhysicalKey::Code(WinitKeyCode::Numpad9) => Some(keys::KeyCode::D9),

//         PhysicalKey::Code(WinitKeyCode::F1) => Some(keys::KeyCode::F1),
//         PhysicalKey::Code(WinitKeyCode::F2) => Some(keys::KeyCode::F2),
//         PhysicalKey::Code(WinitKeyCode::F3) => Some(keys::KeyCode::F3),
//         PhysicalKey::Code(WinitKeyCode::F4) => Some(keys::KeyCode::F4),
//         PhysicalKey::Code(WinitKeyCode::F5) => Some(keys::KeyCode::F5),
//         PhysicalKey::Code(WinitKeyCode::F6) => Some(keys::KeyCode::F6),
//         PhysicalKey::Code(WinitKeyCode::F7) => Some(keys::KeyCode::F7),
//         PhysicalKey::Code(WinitKeyCode::F8) => Some(keys::KeyCode::F8),
//         PhysicalKey::Code(WinitKeyCode::F9) => Some(keys::KeyCode::F9),
//         PhysicalKey::Code(WinitKeyCode::F10) => Some(keys::KeyCode::F10),
//         PhysicalKey::Code(WinitKeyCode::F11) => Some(keys::KeyCode::F11),
//         PhysicalKey::Code(WinitKeyCode::F12) => Some(keys::KeyCode::F12),

//         PhysicalKey::Code(WinitKeyCode::ArrowUp) => Some(keys::KeyCode::Up),
//         PhysicalKey::Code(WinitKeyCode::ArrowDown) => Some(keys::KeyCode::Down),
//         PhysicalKey::Code(WinitKeyCode::ArrowLeft) => Some(keys::KeyCode::Left),
//         PhysicalKey::Code(WinitKeyCode::ArrowRight) => Some(keys::KeyCode::Right),

//         // NOTE: these keys trigger on the numpad when NumLock is off.
//         // We will translate them back to the appropriate numpad keys:
//         PhysicalKey::Code(WinitKeyCode::Home) => Some(keys::KeyCode::NumPad7),
//         PhysicalKey::Code(WinitKeyCode::End) => Some(keys::KeyCode::NumPad1),
//         PhysicalKey::Code(WinitKeyCode::PageUp) => Some(keys::KeyCode::NumPad9),
//         PhysicalKey::Code(WinitKeyCode::PageDown) => Some(keys::KeyCode::NumPad3),

//         _ => None,
//     };

//     if key_code.is_some() {
//         return key_code;
//     }

//     if let WinitKey::Character(s) = logical_key {
//         let key_code = match s.as_str() {
//             "a" => Some(keys::KeyCode::A),
//             "b" => Some(keys::KeyCode::B),
//             "c" => Some(keys::KeyCode::C),
//             "d" => Some(keys::KeyCode::D),
//             "e" => Some(keys::KeyCode::E),
//             "f" => Some(keys::KeyCode::F),
//             "g" => Some(keys::KeyCode::G),
//             "h" => Some(keys::KeyCode::H),
//             "i" => Some(keys::KeyCode::I),
//             "j" => Some(keys::KeyCode::J),
//             "k" => Some(keys::KeyCode::K),
//             "l" => Some(keys::KeyCode::L),
//             "m" => Some(keys::KeyCode::M),
//             "n" => Some(keys::KeyCode::N),
//             "o" => Some(keys::KeyCode::O),
//             "p" => Some(keys::KeyCode::P),
//             "q" => Some(keys::KeyCode::Q),
//             "r" => Some(keys::KeyCode::R),
//             "s" => Some(keys::KeyCode::S),
//             "t" => Some(keys::KeyCode::T),
//             "u" => Some(keys::KeyCode::U),
//             "v" => Some(keys::KeyCode::V),
//             "w" => Some(keys::KeyCode::W),
//             "x" => Some(keys::KeyCode::X),
//             "y" => Some(keys::KeyCode::Y),
//             "z" => Some(keys::KeyCode::Z),
//             "?" => Some(keys::KeyCode::QuestionMark),

//             _ => None,
//         };

//         if key_code.is_some() {
//             return key_code;
//         }
//     }

//     None
// }

// #[derive(Debug)]
// struct TriggerUpdateEvent {}

// struct App<S: SettingsStore + 'static> {
//     display_builder: DisplayBuilder,
//     gl_context: Option<PossiblyCurrentContext>,
//     // glutin NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
//     app_state: Option<AppState>,
//     loop_state: LoopState,
//     settings_store: S,
//     opengl_app: Option<OpenGlApp>,
//     monitors: Vec<MonitorHandle>,
//     ui_paint_batches: Vec<ClippedPrimitive>,
//     egui_shapes: Option<Vec<egui::epaint::ClippedShape>>,
//     modifiers: ModifiersState,
//     window_pos: Point,
//     pre_fullscreen_window_pos: Point,
//     exit_state: Result<(), Box<dyn std::error::Error>>,
//     last_tick_time: Instant,
// }

// impl<S: SettingsStore + 'static> App<S> {
//     fn new(display_builder: DisplayBuilder, loop_state: LoopState, settings_store: S) -> Self {
//         Self {
//             display_builder,
//             gl_context: None,
//             app_state: None,
//             loop_state,
//             settings_store,
//             monitors: vec![],
//             opengl_app: None,
//             ui_paint_batches: vec![],
//             egui_shapes: None,
//             modifiers: Default::default(),
//             window_pos: Default::default(),
//             pre_fullscreen_window_pos: Default::default(),
//             exit_state: Ok(()),
//             last_tick_time: Instant::now(),
//         }
//     }

//     fn initialise_window_context(
//         &mut self,
//         event_loop: &ActiveEventLoop,
//     ) -> Result<(), Box<dyn Error>> {
//         use glutin::{
//             config::{Config, ConfigTemplateBuilder, GlConfig},
//             context::{NotCurrentGlContext, PossiblyCurrentGlContext},
//             display::{GetGlDisplay, GlDisplay},
//         };

//         // Find the config with the maximum number of samples.
//         fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
//             #[allow(clippy::expect_used)]
//             configs
//                 .reduce(|accum, config| {
//                     if config.num_samples() > accum.num_samples() {
//                         config
//                     } else {
//                         accum
//                     }
//                 })
//                 // The caller passes this value in and always expects
//                 // to get o Config back. We can't even create an empty
//                 // value here. So if the caller passes in an empty
//                 // iterator, panicking is literally the only thing we
//                 // can do here.
//                 .expect("No `Config` was provided to `DisplayBuilder`")
//         }

//         let (window, gl_config) = {
//             // Glutin: The template will match only the configurations supporting rendering
//             // to windows.
//             let (window, gl_config) = self.display_builder.clone().build(
//                 event_loop,
//                 ConfigTemplateBuilder::new().with_alpha_size(8),
//                 gl_config_picker,
//             )?;

//             let window = window.ok_or("Failed to build a `Window`. Received `None`.")?;

//             log::debug!("Picked a config with {} samples", gl_config.num_samples());

//             // Create gl context.
//             self.gl_context =
//                 Some(create_gl_context(&window, &gl_config)?.treat_as_possibly_current());

//             (window, gl_config)
//         };

//         let attrs = window.build_surface_attributes(Default::default())?;

//         #[allow(unsafe_code)]
//         let gl_surface = unsafe {
//             gl_config
//                 .display()
//                 .create_window_surface(&gl_config, &attrs)?
//         };

//         // Glutin: The context needs to be current for function
//         // loading, which needs a current context on WGL.
//         let gl_context = self
//             .gl_context
//             .as_ref()
//             .ok_or("`Appgl_context` is `None`")?;
//         gl_context.make_current(&gl_surface)?;

//         {
//             use std::ffi::CString;

//             let gl_display = gl_config.display();

//             log::debug!("Loading OpenGL symbols.");
//             gl::load_with(|symbol| match CString::new(symbol) {
//                 Ok(symbol) => gl_display.get_proc_address(symbol.as_c_str()).cast(),
//                 Err(err) => {
//                     log::error!(
//                         "Failed to convert the symbol `{symbol}` to `CString: {}`",
//                         err
//                     );
//                     std::process::exit(1);
//                 }
//             });
//             log::debug!("Loaded OpenGL symbols.");

//             let dpi = window.scale_factor();
//             log::info!("Window HIDPI factor: {:?}", dpi);
//             self.loop_state.dpi = dpi.floor();

//             log::info!("Window inner size (physical): {:?}", window.inner_size());

//             log::info!("Window outer size (physical): {:?}", window.outer_size());
//         }

//         // Load the monitors
//         {
//             // We'll just assume the monitors won't change throughout the game.
//             self.monitors = event_loop.available_monitors().collect();
//             log::debug!("Got all available monitors: {:?}", self.monitors);

//             self.window_pos = window
//                 .outer_position()
//                 .map(|p| Point::new(p.x, p.y))
//                 .unwrap_or_default();
//             log::info!("Window pos: {:?}", self.window_pos);
//             self.pre_fullscreen_window_pos = self.window_pos;

//             log::debug!("All monitors:");
//             for monitor in &self.monitors {
//                 log::debug!(
//                     "* {:?}, pos: {:?}, size: {:?}",
//                     monitor.name(),
//                     monitor.position(),
//                     monitor.size()
//                 );
//             }
//             let current_monitor = get_current_monitor(&self.monitors, self.window_pos);
//             log::info!(
//                 "Current monitor: {:?}, pos: {:?}, size: {:?}",
//                 current_monitor.as_ref().map(MonitorHandle::name),
//                 current_monitor.as_ref().map(MonitorHandle::position),
//                 current_monitor.as_ref().map(MonitorHandle::size)
//             );
//         }

//         // Try setting vsync.
//         if let Err(res) = gl_surface.set_swap_interval(
//             gl_context,
//             SwapInterval::Wait(NonZeroU32::new(1).ok_or("NonZero is in fact zero")?),
//         ) {
//             log::error!("Error setting vsync: {res:?}");
//         }

//         assert!(
//             self.app_state
//                 .replace(AppState { gl_surface, window })
//                 .is_none()
//         );

//         self.opengl_app.replace(self.loop_state.opengl_app());

//         self.last_tick_time = Instant::now();

//         Ok(())
//     }

//     fn exiting(&mut self) {
//         log::info!(
//             "Drawcall count: {}. Capacity: {}.",
//             self.loop_state.overall_max_drawcall_count,
//             engine::DRAWCALL_CAPACITY
//         );

//         // Save any settings on exit.
//         //
//         // NOTE: this is mostly for the window size which doesn't have
//         // actual GUI options in the Settings dialog. Rather, we want
//         // to save whatever the current window size is.
//         self.settings_store.save(&self.loop_state.settings);

//         // Clear the window.
//         self.app_state = None;

//         // glutin: NOTE: The handling below is only needed due to nvidia on Wayland to not crash
//         // on exit due to nvidia driver touching the Wayland display from on
//         // `exit` hook.
//         #[cfg(all(
//             any(windows, unix),
//             not(any(target_os = "macos", target_os = "ios")),
//             not(target_family = "wasm")
//         ))]
//         if let Some(context) = self.gl_context.take() {
//             use glutin::display::GetGlDisplay;
//             let glutin::display::Display::Egl(display) = context.display();
//             #[allow(unsafe_code)]
//             unsafe {
//                 display.terminate();
//             }
//         }
//     }
// }

// impl<S: SettingsStore + 'static> ApplicationHandler<TriggerUpdateEvent> for App<S> {
//     fn window_event(
//         &mut self,
//         event_loop: &ActiveEventLoop,
//         _window_id: winit::window::WindowId,
//         event: WindowEvent,
//     ) {
//         use winit::event::{KeyEvent, WindowEvent};

//         match event {
//             WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
//                 log::info!("WindowEvent::Resized: {:?}", size);

//                 let logical_size: LogicalSize<i32> = size.to_logical(self.loop_state.dpi);

//                 // Glutin: Some platforms like EGL require resizing GL surface to update the size
//                 // Notable platforms here are Wayland and macOS, other don't require it
//                 // and the function is no-op, but it's wise to resize it for portability
//                 // reasons.
//                 if let Some(AppState { gl_surface, window }) = self.app_state.as_ref() {
//                     if let Some(gl_context) = self.gl_context.as_ref() {
//                         gl_surface.resize(
//                             gl_context,
//                             NonZeroU32::new(size.width).unwrap_or(NonZeroU32::MIN),
//                             NonZeroU32::new(size.height).unwrap_or(NonZeroU32::MIN),
//                         );
//                     }

//                     if let Some(monitor_id) = window.fullscreen() {
//                         log::warn!(
//                             "Asked to resize on fullscreen: target size: {:?}, \
//             monitor ID: {:?}. Ignoring this request.",
//                             size,
//                             monitor_id
//                         );
//                     }

//                     self.loop_state
//                         .handle_window_size_changed(logical_size.width, logical_size.height);
//                 }
//             }

//             WindowEvent::Moved(new_pos) => {
//                 if self.loop_state.settings.fullscreen || self.loop_state.switched_from_fullscreen {
//                     // Don't update the window position
//                     //
//                     // Even after we switch from
//                     // fullscreen, the `Moved` event has a
//                     // wrong value that messes things up.
//                     // So we restore the previous position
//                     // manually instead.
//                 } else {
//                     log::trace!(
//                         "[FRAME {}] Window moved to: {:?}",
//                         self.loop_state.current_frame_id,
//                         new_pos
//                     );
//                     self.window_pos.x = new_pos.x;
//                     self.window_pos.y = new_pos.y;
//                     let current_monitor = get_current_monitor(&self.monitors, self.window_pos);
//                     log::trace!(
//                         "Current monitor: {:?}, pos: {:?}, size: {:?}",
//                         current_monitor.as_ref().map(MonitorHandle::name),
//                         current_monitor.as_ref().map(MonitorHandle::position),
//                         current_monitor.as_ref().map(MonitorHandle::size)
//                     );
//                 }
//             }

//             WindowEvent::ModifiersChanged(modifiers) => {
//                 log::trace!("Modifiers changed: {:?}", modifiers);
//                 self.modifiers = modifiers.state();
//             }

//             WindowEvent::KeyboardInput {
//                 event:
//                     KeyEvent {
//                         state: ElementState::Pressed,
//                         logical_key,
//                         physical_key,
//                         ..
//                     },
//                 ..
//             } => {
//                 log::trace!(
//                     "Pressed logical key: {logical_key:?}, physical key: {physical_key:?}, modifiers: {:?}",
//                     self.modifiers
//                 );

//                 if let Some(code) = key_code_from_backend(physical_key, logical_key) {
//                     let key = crate::keys::Key {
//                         code,
//                         alt: self.modifiers.alt_key(),
//                         ctrl: self.modifiers.control_key(),
//                         shift: self.modifiers.shift_key(),
//                         logo: self.modifiers.super_key(),
//                     };
//                     log::trace!("Detected key {:?}", key);
//                     self.loop_state.keys.push(key);
//                 }
//             }

//             WindowEvent::CursorMoved { position, .. } => {
//                 // NOTE: This function expects logical, not physical pixels.
//                 // But the values we get in this event are physical, so we need
//                 // to divide by the DPI to mae them logical.
//                 self.loop_state.update_mouse_position(
//                     self.loop_state.dpi,
//                     (position.x / self.loop_state.dpi) as i32,
//                     (position.y / self.loop_state.dpi) as i32,
//                 );
//             }

//             WindowEvent::MouseWheel { delta, .. } => {
//                 use MouseScrollDelta::*;
//                 match delta {
//                     LineDelta(x, y) => self.loop_state.mouse.scroll_delta = [x, y],
//                     PixelDelta(PhysicalPosition { x: x_px, y: y_px }) => {
//                         let line_height_px = self.loop_state.settings.text_size as f32;
//                         self.loop_state.mouse.scroll_delta =
//                             [x_px as f32 / line_height_px, y_px as f32 / line_height_px]
//                     }
//                 }
//             }

//             WindowEvent::MouseInput {
//                 state: ElementState::Pressed,
//                 button,
//                 ..
//             } => {
//                 use MouseButton::*;
//                 match button {
//                     Left => {
//                         self.loop_state.mouse.left_is_down = true;
//                     }
//                     Right => {
//                         self.loop_state.mouse.right_is_down = true;
//                     }
//                     _ => {}
//                 }
//             }

//             WindowEvent::MouseInput {
//                 state: ElementState::Released,
//                 button,
//                 ..
//             } => {
//                 use MouseButton::*;
//                 match button {
//                     Left => {
//                         self.loop_state.mouse.left_clicked = true;
//                         self.loop_state.mouse.left_is_down = false;
//                     }
//                     Right => {
//                         self.loop_state.mouse.right_clicked = true;
//                         self.loop_state.mouse.right_is_down = false;
//                     }
//                     _ => {}
//                 }
//             }

//             WindowEvent::CloseRequested => event_loop.exit(),

//             WindowEvent::RedrawRequested => {
//                 if let Some(AppState { gl_surface, window }) = self.app_state.as_ref() {
//                     if let Some(gl_context) = self.gl_context.as_ref() {
//                         if let Some(opengl_app) = self.opengl_app.as_mut() {
//                             if let Some(egui_shapes) = self.egui_shapes.take() {
//                                 self.ui_paint_batches = self
//                                     .loop_state
//                                     .egui_context
//                                     .tessellate(egui_shapes, self.loop_state.dpi as f32);

//                                 let (ui_vertices, batches) =
//                                     drawcalls_from_egui(opengl_app, &self.ui_paint_batches);

//                                 self.loop_state.process_vertices_and_render(
//                                     opengl_app,
//                                     &ui_vertices,
//                                     self.loop_state.dpi,
//                                     &batches,
//                                 );

//                                 // NOTE: according to winit docs, this could properly throttle RedrawRequested
//                                 // https://docs.rs/winit/latest/winit/window/struct.Window.html#method.pre_present_notify
//                                 window.pre_present_notify();

//                                 if let Err(err) = gl_surface.swap_buffers(gl_context) {
//                                     log::error!("Swapping buffers failed: {err}");
//                                 }
//                             }
//                         }

//                         if cfg!(feature = "fullscreen") {
//                             use engine::loop_state::FullscreenAction::*;
//                             match self.loop_state.fullscreen_action() {
//                                 Some(SwitchToFullscreen) => {
//                                     let current_monitor =
//                                         get_current_monitor(&self.monitors, self.window_pos);
//                                     if let Some(ref monitor) = current_monitor {
//                                         self.pre_fullscreen_window_pos = self.window_pos;
//                                         log::info!(
//                                             "Monitor: {:?}, pos: {:?}, dimensions: {:?}",
//                                             monitor.name(),
//                                             monitor.position(),
//                                             monitor.size()
//                                         );
//                                         window.set_fullscreen(Some(Fullscreen::Borderless(Some(
//                                             monitor.clone(),
//                                         ))));
//                                     } else {
//                                         log::warn!("`current_monitor` is not set!??");
//                                     }
//                                 }
//                                 Some(SwitchToWindowed) => {
//                                     window.set_fullscreen(None);
//                                     let pos = window.outer_position();
//                                     log::info!("New window position: {:?}", pos);
//                                     window.set_decorations(true);
//                                     self.loop_state.switched_from_fullscreen = true;
//                                 }
//                                 None => {}
//                             };

//                             // If we just switched from fullscreen back to a windowed
//                             // mode, restore the window position we had before. We do this
//                             // because the `Moved` event fires with an incorrect value
//                             // when coming back from full screen.
//                             //
//                             // This ensures that we can switch full screen back and fort
//                             // on a multi monitor setup.
//                             if self.loop_state.switched_from_fullscreen {
//                                 self.window_pos = self.pre_fullscreen_window_pos;
//                             }
//                         }

//                         match self.loop_state.check_window_size_needs_updating() {
//                             ResizeWindowAction::NewSize(desired_window_size_px) => {
//                                 log::info!(
//                                     "Updating window to new size: {:?}",
//                                     desired_window_size_px
//                                 );
//                                 let size: LogicalSize<u32> = desired_window_size_px.into();
//                                 let size = size.to_physical(window.scale_factor());
//                                 gl_surface.resize(
//                                     gl_context,
//                                     NonZeroU32::new(size.width).unwrap_or(NonZeroU32::MIN),
//                                     NonZeroU32::new(size.height).unwrap_or(NonZeroU32::MIN),
//                                 );
//                             }
//                             ResizeWindowAction::NoChange => {}
//                         }
//                     }
//                 }
//             }

//             _e => {
//                 // dbg!(_e);
//             }
//         }
//     }

//     fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: TriggerUpdateEvent) {
//         let frame_start_time = Instant::now();
//         let dt = frame_start_time.duration_since(self.last_tick_time);
//         self.last_tick_time = frame_start_time;

//         self.loop_state
//             .egui_context
//             .begin_pass(self.loop_state.egui_raw_input());

//         self.loop_state.update_fps(dt);

//         match self.loop_state.update_game(dt, &mut self.settings_store) {
//             UpdateResult::QuitRequested => event_loop.exit(),
//             UpdateResult::KeepGoing => {}
//         }

//         let output = self.loop_state.egui_context.end_pass();

//         for command in &output.platform_output.commands {
//             if let egui::OutputCommand::OpenUrl(url) = command {
//                 if let Err(err) = webbrowser::open(&url.url) {
//                     log::warn!("Error opening URL {} in the external browser!", url.url);
//                     log::warn!("{}", err);
//                 }
//             }
//         }

//         self.egui_shapes = Some(output.shapes);

//         if output.textures_delta.set.is_empty() {
//             // We don't need to set/update any textures
//         } else {
//             for (_texture_id, image_delta) in output.textures_delta.set {
//                 match image_delta.image {
//                     egui::epaint::image::ImageData::Color(color_image) => {
//                         log::warn!(
//                             "Received ImageDelta::Color(ColorImage) of size: {:?}. Ignoring as we're not set up to handle this.",
//                             color_image.size
//                         );
//                     }
//                     egui::epaint::image::ImageData::Font(font_image) => {
//                         log::warn!(
//                             "We need to update the egui texture map FontImage of size: {:?}",
//                             font_image.size
//                         );
//                         let font_image = loop_state::egui_font_image_apply_delta(
//                             self.loop_state.font_texture.clone(),
//                             image_delta.pos,
//                             font_image,
//                         );
//                         self.loop_state.font_texture = font_image.clone();

//                         let egui_texture = loop_state::build_texture_from_egui(font_image);
//                         let (width, height) = egui_texture.dimensions();

//                         if let Some(opengl_app) = &mut self.opengl_app {
//                             opengl_app.eguimap_size_px = [width as f32, height as f32];
//                             opengl_app.upload_texture(opengl_app.eguimap, "egui", &egui_texture);
//                         }
//                     }
//                 }
//             }
//         }

//         if output.textures_delta.free.is_empty() {
//             // Don't print anything
//         } else {
//             // NOTE: I don't think we need to free anything.
//             // We're just uploading the single egui-based
//             // texture.
//             log::warn!("Texture IDs to free");
//             for texture_id in output.textures_delta.free {
//                 dbg!(texture_id);
//             }
//         }

//         if let Some(AppState { window, .. }) = self.app_state.as_ref() {
//             window.request_redraw();
//         }
//     }

//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         log::debug!("App::resumed (this should be called only once).");

//         if let Err(err) = self.initialise_window_context(event_loop) {
//             log::error!("Initialising the window context failed");
//             self.exit_state = Err(err);
//             event_loop.exit();
//         };
//     }
// }

// struct AppState {
//     gl_surface: Surface<WindowSurface>,
//     // NOTE: Window should be dropped after all resources created using its
//     // raw-window-handle.
//     window: Window,
// }

// fn create_gl_context(
//     window: &Window,
//     gl_config: &glutin::config::Config,
// ) -> Result<NotCurrentContext, glutin::error::Error> {
//     use glutin::{
//         context::{ContextApi, ContextAttributesBuilder, Version},
//         display::{GetGlDisplay, GlDisplay},
//     };

//     let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

//     // The context creation part.
//     let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

//     // Since glutin by default tries to create OpenGL core context, which may not be
//     // present we should try gles.
//     let fallback_context_attributes = ContextAttributesBuilder::new()
//         .with_context_api(ContextApi::Gles(None))
//         .build(raw_window_handle);

//     // There are also some old devices that support neither modern OpenGL nor GLES.
//     // To support these we can try and create a 2.1 context.
//     let legacy_context_attributes = ContextAttributesBuilder::new()
//         .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
//         .build(raw_window_handle);

//     // Reuse the uncurrented context from a suspended() call if it exists, otherwise
//     // this is the first time resumed() is called, where the context still
//     // has to be created.
//     let gl_display = gl_config.display();

//     #[allow(unsafe_code)]
//     unsafe {
//         gl_display
//             .create_context(gl_config, &context_attributes)
//             .or_else(|_| {
//                 gl_display
//                     .create_context(gl_config, &fallback_context_attributes)
//                     .or_else(|_| gl_display.create_context(gl_config, &legacy_context_attributes))
//             })
//     }
// }

// fn get_current_monitor(monitors: &[MonitorHandle], window_pos: Point) -> Option<MonitorHandle> {
//     for monitor in monitors {
//         let monitor_pos = {
//             let pos = monitor.position();
//             Point::new(pos.x, pos.y)
//         };
//         let monitor_dimensions = {
//             let dim = monitor.size();
//             Point::new(dim.width as i32, dim.height as i32)
//         };

//         let monitor_bottom_left = monitor_pos + monitor_dimensions;
//         if window_pos >= monitor_pos && window_pos < monitor_bottom_left {
//             return Some(monitor.clone());
//         }
//     }

//     monitors.iter().next().cloned()
// }

// // NOTE: convert Egui indexed vertices into ones our
// // engine understands. I.e. naive 3 vertices per
// // triangle with duplication.
// fn drawcalls_from_egui(
//     opengl_app: &OpenGlApp,
//     ui_paint_batches: &Vec<ClippedPrimitive>,
// ) -> (Vec<Vertex>, Vec<([f32; 4], i32, i32)>) {
//     use egui::epaint::Primitive;

//     // TODO: consider doing updating our engine to suport
//     // vertex indices.
//     let mut ui_vertices = vec![];
//     let mut batches = vec![];
//     let mut index = 0;

//     for clipped_primitive in ui_paint_batches {
//         let ClippedPrimitive {
//             clip_rect,
//             primitive,
//         } = clipped_primitive;

//         if let Primitive::Mesh(mesh) = primitive {
//             let texture_id = match mesh.texture_id {
//                 egui::TextureId::Managed(0) => engine::Texture::Egui.into(),
//                 egui::TextureId::Managed(id) => {
//                     log::error!("Unexpected Managed texture ID: {}", id);
//                     engine::Texture::Egui.into()
//                 }
//                 egui::TextureId::User(id) => id as f32,
//             };

//             // NOTE: the shader expects the egui texture (uv)
//             // coordinates to be normalised, but everything
//             // else expects pixel coordinates.
//             //
//             // However, everything that comes out of egui *is
//             // going to be normalised* so we need to
//             // "denormalise" it by multiplying the uv coords
//             // with the size of the texture in pixels.
//             //
//             // For egui we just multiply by 1.0 which has no
//             // effect.
//             let texture_size = match mesh.texture_id {
//                 egui::TextureId::Managed(0) => [1.0, 1.0],
//                 egui::TextureId::Managed(id) => {
//                     log::error!(
//                         "Unexpected TextureId::Managed({})! We should only ever see ID of 0",
//                         id
//                     );
//                     [1.0, 1.0]
//                 }
//                 egui::TextureId::User(engine::TEXTURE_GLYPH) => opengl_app.glyphmap_size_px,
//                 egui::TextureId::User(engine::TEXTURE_TILEMAP) => opengl_app.tilemap_size_px,
//                 id => {
//                     log::error!(
//                         "ERROR[Winit RedrawRequested]: unknown texture ID: `{:?}`",
//                         id
//                     );
//                     [1.0, 1.0]
//                 }
//             };

//             for &index in &mesh.indices {
//                 let egui_vertex = match mesh.vertices.get(index as usize) {
//                     Some(vertex) => vertex,
//                     None => {
//                         log::error!("Can't index into the mesh.vertices");
//                         continue;
//                     }
//                 };
//                 let color = Color {
//                     r: egui_vertex.color.r(),
//                     g: egui_vertex.color.g(),
//                     b: egui_vertex.color.b(),
//                 }
//                 .alpha(egui_vertex.color.a());
//                 let (u, v) = (egui_vertex.uv.x, egui_vertex.uv.y);

//                 let pos = egui_vertex.pos;
//                 let vertex = engine::Vertex {
//                     texture_id,
//                     pos_px: [pos.x, pos.y],
//                     tile_pos: [u * texture_size[0], v * texture_size[1]],
//                     color: color.into(),
//                 };
//                 ui_vertices.push(vertex);
//             }

//             let vertex_count = mesh.indices.len() as i32;
//             batches.push((
//                 [
//                     clip_rect.left_top().x,
//                     clip_rect.left_top().y,
//                     clip_rect.right_bottom().x,
//                     clip_rect.right_bottom().y,
//                 ],
//                 index,
//                 vertex_count,
//             ));
//             index += vertex_count;
//         }
//     }

//     (ui_vertices, batches)
// }

pub fn main_loop<S>(
    initial_default_background: Color,
    window_title: &str,
    settings_store: S,
    initial_state: Box<State>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SettingsStore + 'static,
{
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl3 demo", 800, 600)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas();

    canvas.set_draw_color(sdl3::pixels::Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(sdl3::pixels::Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000_u32 / 60));
    }

    Ok(())
}
