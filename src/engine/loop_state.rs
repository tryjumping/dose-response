use crate::{
    audio::Audio,
    color::Color,
    engine::{self, opengl::OpenGlApp, Display, DisplayInfo, Drawcall, Mouse, TextMetrics, Vertex},
    gamepad::{self, Gamepad},
    keys::Key,
    palette::Palette,
    point::Point,
    settings::{Settings, Store as SettingsStore},
    state::State,
};

use std::{convert::TryInto, sync::Arc, time::Duration};

use egui::{self, epaint::image::FontImage, Event, RawInput};

use gilrs::Gilrs;

use image::{Rgba, RgbaImage};

pub enum FullscreenAction {
    SwitchToFullscreen,
    SwitchToWindowed,
}

pub enum ResizeWindowAction {
    NewSize((u32, u32)),
    NoChange,
}

pub enum UpdateResult {
    QuitRequested,
    KeepGoing,
}

pub struct Metrics {
    pub tile_width_px: i32,
    pub text_width_px: i32,
}

impl TextMetrics for Metrics {
    fn tile_width_px(&self) -> i32 {
        self.tile_width_px
    }

    fn text_width_px(&self) -> i32 {
        self.text_width_px
    }
}

pub fn build_texture_from_egui(font_image: FontImage) -> RgbaImage {
    let width = font_image.size[0].try_into().unwrap_or(0);
    let height = font_image.size[1].try_into().unwrap_or(0);

    // NOTE: the pixels vec generated by egui is a list of u8
    // values each representing an alpha value for the given
    // pixel in the map.
    //
    // We convert it to the Rgba format that every texture in
    // the game uses to make our rendering code more uniform
    // and easier to debug.
    let mut texture = RgbaImage::new(width, height);

    for (index, pixel) in font_image.srgba_pixels(None).enumerate() {
        let pixel = Rgba([pixel.r(), pixel.g(), pixel.b(), pixel.a()]);
        texture.put_pixel(index as u32 % width, index as u32 / width, pixel);
    }

    texture
}

/// Apply a (potentially) partial `ImageDelta` to an existing `FontImage`.
///
/// When we start the game, egui will give us the full font atlas
/// texture. However, when we do things like update the image size,
/// we'll get a delta -- a portion of the image we need to update.
///
/// So this takes the current texture and applies the delta at the
/// requested position.
pub fn egui_font_image_apply_delta(
    source: FontImage,
    pos: Option<[usize; 2]>,
    delta: FontImage,
) -> FontImage {
    match pos {
        Some(pos) => {
            // Ensure that the vec size matches the image size before we do any direct mem manipulations
            debug_assert_eq!(delta.pixels.len(), delta.size[0] * delta.size[1]);

            // Ensure that the delta image is smaller than the source image
            debug_assert!(source.size[0] >= delta.size[0]);
            debug_assert!(source.size[1] >= delta.size[1]);

            // Ensure that the delta image fits all within the source image when positioned
            debug_assert!(source.size[0] >= pos[0] + delta.size[0]);
            debug_assert!(source.size[1] >= pos[1] + delta.size[1]);

            let mut updated_image = source;

            for (delta_index, &alpha) in delta.pixels.iter().enumerate() {
                let pos_in_delta: [usize; 2] =
                    [delta_index % delta.size[0], delta_index / delta.size[0]];

                let updated_x = pos[0] + pos_in_delta[0];
                let updated_y = pos[1] + pos_in_delta[1];

                let updated_index = updated_y * updated_image.size[0] + updated_x;

                updated_image.pixels[updated_index] = alpha;
            }

            updated_image
        }
        None => delta,
    }
}

pub fn egui_load_font(ctx: &egui::Context) {
    let font_definitions = {
        let family = egui::FontFamily::Monospace;
        let font_name = String::from("Mononoki");

        let mut def = egui::FontDefinitions::default();
        def.font_data.insert(
            font_name.clone(),
            egui::FontData::from_static(include_bytes!("../../fonts/mononoki-Regular.ttf")).into(),
        );

        if let Some(fam) = def.families.get_mut(&family) {
            fam.insert(0, font_name);
        }

        def
    };
    ctx.set_fonts(font_definitions);
}

pub fn egui_style(palette: &Palette, font_size: f32) -> egui::Style {
    let mut style = egui::Style::default();
    // NOTE: this applies to check/radio boxes as well, not just regular buttons:
    style.spacing.button_padding = [7.0, 3.0].into();

    // TODO: and have it respond to the palette changes.

    style.visuals.widgets.inactive.bg_fill = palette.gui_button_background.into();
    style.visuals.widgets.inactive.weak_bg_fill = palette.gui_button_background.into();

    style.visuals.widgets.active.bg_fill = palette.gui_button_highlighted_background.into();
    style.visuals.widgets.active.weak_bg_fill = palette.gui_button_highlighted_background.into();

    style.visuals.widgets.active.bg_stroke.color = palette.gui_button_highlighted_stroke.into();
    style.visuals.widgets.active.bg_stroke.width = 3.0;

    style.visuals.widgets.hovered.bg_fill = palette.gui_button_highlighted_background.into();
    style.visuals.widgets.hovered.weak_bg_fill = palette.gui_button_highlighted_background.into();

    style.spacing.scroll = egui::style::ScrollStyle::solid();

    for font_id in style.text_styles.values_mut() {
        *font_id = egui::FontId::monospace(font_size);
    }

    style
}

pub struct LoopState {
    pub settings: Settings,
    pub previous_settings: Settings,
    pub display: Display,
    pub audio: Audio,
    pub dpi: f64,
    pub glyphmap: RgbaImage,
    pub tilemap: RgbaImage,
    pub egui_context: egui::Context,
    pub gilrs: Option<Gilrs>,
    pub default_background: Color,
    pub drawcalls: Vec<Drawcall>,
    pub overall_max_drawcall_count: usize,
    pub vertex_buffer: Vec<f32>,
    pub font_texture: FontImage,
    pub game_state: Box<State>,
    pub mouse: Mouse,
    pub keys: Vec<Key>,
    pub gamepad: Gamepad,
    pub fps_clock: Duration,
    pub switched_from_fullscreen: bool,
    pub frames_in_current_second: i32,
    pub fps: i32,

    // NOTE: This will wrap after running continuously for over 64
    // years at 60 FPS. 32 bits are just fine.
    pub current_frame_id: i32,
}

impl LoopState {
    pub fn initialise(
        settings: Settings,
        default_background: Color,
        game_state: Box<State>,
        egui_context: egui::Context,
        stream_handle: Option<&rodio::OutputStreamHandle>,
    ) -> Self {
        // TODO: do this for every Display creatio / window resize
        let window_size_px =
            Point::new(settings.window_width as i32, settings.window_height as i32);

        assert_eq!(
            std::mem::size_of::<Vertex>(),
            engine::VERTEX_COMPONENT_COUNT * 4
        );

        let display = Display::new(window_size_px, settings.tile_size, settings.text_size);

        log::debug!(
            "Requested display in tiles: {} x {}",
            display.display_size.x,
            display.display_size.y
        );

        let glyphmap = {
            let data = &include_bytes!(concat!(env!("OUT_DIR"), "/glyph.png"))[..];
            image::load_from_memory_with_format(data, image::ImageFormat::Png)
                .unwrap_or_else(|e| {
                    log::error!("Error loading the glyph tilemap image: {}", e);
                    log::warn!("Generating an empty 32x32 pixels image as a fallback.");
                    image::DynamicImage::new_rgba8(32, 32)
                })
                .to_rgba8()
        };
        log::debug!("Loaded glyph tilemap.");

        let mut tilemap = {
            // NOTE: including a manually-edited tileset based on Bountiful Bits
            let data = &include_bytes!("../../assets/tiles.png")[..];
            image::load_from_memory_with_format(data, image::ImageFormat::Png)
                .unwrap_or_else(|e| {
                    log::error!("Error loading the graphics tilemap image: {}", e);
                    log::warn!("Generating an empty 32x32 pixels image as a fallback.");
                    image::DynamicImage::new_rgba8(32, 32)
                })
                .to_rgba8()
        };
        log::debug!("Loaded the graphics tilemap.");
        // Normalise the tilemap colours.
        //
        // The current tilemap has alpha, but it also sets explicit
        // colours. This doesn't work with our colour schemes and the
        // way we do the High effect by overriding some of the
        // colours. That all expects the original colour to be white
        // so what we do here is turn every nonzero pixel to white.
        for pixel in tilemap.pixels_mut() {
            use image::Pixel;
            pixel.apply_with_alpha(|channel| if channel == 0 { 0 } else { 255 }, |alpha| alpha);
        }
        log::debug!("Normalised the graphics tilemap colours.");
        let tilemap = tilemap; // Disable `mut`

        egui_load_font(&egui_context);

        let style = egui_style(&game_state.palette, settings.text_size as f32);
        egui_context.set_style(Arc::new(style));

        let gilrs = match Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(err) => {
                log::error!("Could not initialise gamepad. Error: {:?}", err);
                None
            }
        };

        // Always start from a windowed mode. This will force the
        // fullscreen switch in the first frame if requested in the
        // settings we've loaded.
        //
        // This is necessary because some backends don't support
        // fullscreen on window creation. And TBH, this is easier on us
        // because it means there's only one fullscreen-handling pathway.
        let previous_settings = Settings {
            fullscreen: false,
            ..settings.clone()
        };
        log::debug!(
            "Desired window size: {} x {}",
            window_size_px.x,
            window_size_px.y
        );
        assert_eq!(window_size_px, display.screen_size_px);
        assert_eq!(window_size_px.x, settings.window_width as i32);
        assert_eq!(window_size_px.y, settings.window_height as i32);
        Self {
            settings,
            previous_settings,
            display,
            audio: Audio::new(stream_handle),
            dpi: 1.0,
            glyphmap,
            tilemap,
            egui_context,
            gilrs,
            default_background,
            drawcalls: Vec::with_capacity(engine::DRAWCALL_CAPACITY),
            overall_max_drawcall_count: 0,
            vertex_buffer: Vec::with_capacity(engine::VERTEX_BUFFER_CAPACITY),
            font_texture: Default::default(),
            game_state,
            mouse: Mouse::new(),
            keys: vec![],
            gamepad: Gamepad::new(),
            fps_clock: Duration::new(0, 0),
            switched_from_fullscreen: false,
            frames_in_current_second: 0,
            fps: 0,
            current_frame_id: 0,
        }
    }

    pub fn opengl_app(&self) -> OpenGlApp {
        let vs_source = include_str!("../shader_150.glslv");
        let fs_source = include_str!("../shader_150.glslf");
        let mut opengl_app = OpenGlApp::new(vs_source, fs_source);
        log::debug!("Created opengl app.");

        opengl_app.initialise(&self.glyphmap, &self.tilemap);
        log::debug!("Initialised opengl app.");
        opengl_app
    }

    pub fn desired_window_size_px(&self) -> (u32, u32) {
        // let result = self.display.size_without_padding() * self.settings.tile_size;
        // (result.x as u32, result.y as u32)

        // NOTE: instead of resizing the window based on the current
        // tilesize, use the value from the settings:
        (self.settings.window_width, self.settings.window_height)
    }

    pub fn update_fps(&mut self, dt: Duration) {
        self.fps_clock += dt;
        self.frames_in_current_second += 1;
        self.current_frame_id += 1;
        if self.fps_clock.as_millis() > 1000 {
            self.fps = self.frames_in_current_second;
            self.frames_in_current_second = 1;
            self.fps_clock = Duration::new(0, 0);
        }
    }

    pub fn update_game(
        &mut self,
        dt: Duration,
        settings_store: &mut dyn SettingsStore,
    ) -> UpdateResult {
        use crate::game::RunningState;
        let tile_width_px = self.settings.tile_size;
        let text_width_px = self.settings.text_size;

        let loop_state_dpi = self.dpi as f32;
        if self.egui_context.pixels_per_point() != loop_state_dpi {
            log::info!(
                "Updating egui's dpi from: {} to {}",
                self.egui_context.pixels_per_point(),
                loop_state_dpi
            );
            self.egui_context.set_pixels_per_point(loop_state_dpi);
        }

        self.game_state.keyboard_scroll_delta = [0.0, 0.0];

        if let Some(gilrs) = self.gilrs.as_mut() {
            gamepad::process_gamepad_events(gilrs, &mut self.gamepad, dt)
        }

        let previous_palette = self.settings.palette();

        self.previous_settings = self.settings.clone();

        let mut update_result = crate::game::update(
            &mut self.game_state,
            &self.egui_context,
            dt,
            self.fps,
            &self.keys,
            self.mouse,
            &mut self.gamepad,
            &mut self.settings,
            &Metrics {
                tile_width_px,
                text_width_px,
            },
            settings_store,
            &mut self.display,
            &mut self.audio,
        );

        let skipping = std::matches!(update_result, RunningState::Skip);
        if skipping {
            log::trace!("Skipping no-op frames...");
        }
        while std::matches!(update_result, RunningState::Skip) {
            update_result = crate::game::update(
                &mut self.game_state,
                &self.egui_context,
                dt,
                self.fps,
                &[],
                Mouse::new(),
                &mut self.gamepad,
                &mut self.settings,
                &Metrics {
                    tile_width_px,
                    text_width_px,
                },
                settings_store,
                &mut self.display,
                &mut self.audio,
            );
        }
        if skipping {
            log::trace!("Finished the frame skip");
        }

        if previous_palette != self.settings.palette() {
            // The palette has changed, we need to update the egui style
            let style = egui_style(&self.settings.palette(), self.settings.text_size as f32);
            self.egui_context.set_style(Arc::new(style));
        }

        match update_result {
            RunningState::Running => {}
            RunningState::NewGame(new_state) => {
                self.game_state = new_state;
            }
            RunningState::Stopped => return UpdateResult::QuitRequested,
            RunningState::Skip => unreachable!(),
        }

        self.reset_inputs();

        UpdateResult::KeepGoing
    }

    pub fn egui_raw_input(&self) -> RawInput {
        let text_size = self.settings.text_size as f32;
        let mouse = self.game_state.mouse;
        let mouse_pos = [mouse.screen_pos.x as f32, mouse.screen_pos.y as f32].into();
        let mut events = vec![Event::PointerMoved(mouse_pos)];

        let scroll_delta = if mouse.scroll_delta == [0.0, 0.0] {
            self.game_state.keyboard_scroll_delta
        } else {
            [
                mouse.scroll_delta[0] * text_size,
                mouse.scroll_delta[1] * text_size,
            ]
        };

        events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: scroll_delta.into(),
            modifiers: egui::Modifiers::NONE,
        });

        if mouse.left_clicked {
            events.push(Event::PointerButton {
                pos: mouse_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            });
            events.push(Event::PointerButton {
                pos: mouse_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            });
        }
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Default::default(),
                [
                    self.settings.window_width as f32,
                    self.settings.window_height as f32,
                ]
                .into(),
            )),
            events,

            // TODO: I think we'll have to call `Context::set_zoom_factor` to handle DPI from now on
            // todo go here
            //pixels_per_point: self.dpi,
            ..Default::default()
        }
    }

    /// The inputs are in LOGICAL pixels.
    pub fn handle_window_size_changed(&mut self, new_width: i32, new_height: i32) {
        log::info!("Window resized to: {} x {}", new_width, new_height);
        let new_window_size_px = Point::new(new_width, new_height);
        if self.display.screen_size_px != new_window_size_px {
            self.settings.window_width = new_width as u32;
            self.settings.window_height = new_height as u32;
            self.display = Display::new(
                new_window_size_px,
                self.settings.tile_size,
                self.settings.text_size,
            );
        }
    }

    pub fn change_tilesize_px(&mut self, new_tilesize_px: i32) {
        if crate::engine::AVAILABLE_TILE_SIZES.contains(&new_tilesize_px) {
            log::info!(
                "Changing tilesize from {} to {}",
                self.display.tile_size,
                new_tilesize_px
            );
            self.settings.tile_size = new_tilesize_px;
            // Recreate the display, because the tile count is now different:
            self.display = Display::new(
                self.display.screen_size_px,
                self.settings.tile_size,
                self.settings.text_size,
            );
        } else {
            log::warn!(
                "Trying to switch to a tilesize that's not available: {}. Only these ones exist: {:?}",
                new_tilesize_px,
                crate::engine::AVAILABLE_TILE_SIZES
            );
        }
    }

    pub fn change_text_size_px(&mut self, new_text_size_px: i32) {
        if crate::engine::AVAILABLE_TEXT_SIZES.contains(&new_text_size_px) {
            log::info!(
                "Changing text from {} to {}",
                self.display.text_size,
                new_text_size_px
            );
            self.settings.text_size = new_text_size_px;
            self.display = Display::new(
                self.display.screen_size_px,
                self.settings.tile_size,
                self.settings.text_size,
            );

            // Update the current egui font size:
            let style = egui_style(&self.game_state.palette, self.settings.text_size as f32);
            self.egui_context.set_style(Arc::new(style));
        } else {
            log::warn!(
                "Trying to switch to a text size that's not available: {}. Only these ones exist: {:?}",
                new_text_size_px,
                crate::engine::AVAILABLE_TEXT_SIZES
            );
        }
    }

    pub fn display_info(&self, dpi: f64) -> DisplayInfo {
        engine::calculate_display_info(
            [
                self.display.screen_size_px.x as f32,
                self.display.screen_size_px.y as f32,
            ],
            self.display.size_without_padding(),
            self.settings.tile_size,
            dpi as f32,
        )
    }

    pub fn reset_inputs(&mut self) {
        self.mouse.left_clicked = false;
        self.mouse.right_clicked = false;
        self.mouse.scroll_delta = [0.0, 0.0];
        self.keys.clear();
    }

    pub fn update_mouse_position(&mut self, dpi: f64, window_px_x: i32, window_px_y: i32) {
        let display_info = self.display_info(dpi);

        let x = window_px_x.clamp(0, display_info.window_size_px[0] as i32 - 1);
        let y = window_px_y.clamp(0, display_info.window_size_px[1] as i32 - 1);

        self.mouse.screen_pos = Point { x, y };

        let tile_width = display_info.display_px[0] as i32 / self.display.size_without_padding().x;
        let mouse_tile_x = x / tile_width;

        let tile_height = display_info.display_px[1] as i32 / self.display.size_without_padding().y;
        let mouse_tile_y = y / tile_height;

        self.mouse.tile_pos = Point {
            x: mouse_tile_x,
            y: mouse_tile_y,
        };
    }

    pub fn push_drawcalls_to_display(&mut self) {
        let current_capacity = self.drawcalls.capacity();
        self.drawcalls.clear();
        self.display
            .push_drawcalls(self.settings.visual_style, &mut self.drawcalls);

        if self.drawcalls.len() > self.overall_max_drawcall_count {
            self.overall_max_drawcall_count = self.drawcalls.len();
        }

        if self.drawcalls.len() > current_capacity {
            log::warn!(
                "Warning: drawcall count exceeded current capacity {}. Current count: {}.",
                current_capacity,
                self.drawcalls.len(),
            );
        }
    }

    pub fn render(&self, gl: &OpenGlApp, dpi: f64, batches: &[([f32; 4], i32, i32)]) {
        let display_info = self.display_info(dpi);
        gl.render(self.default_background, display_info, &self.vertex_buffer);

        for &(clip_rect, vertex_index, vertex_count) in batches {
            gl.render_clipped_vertices(display_info, clip_rect, (vertex_index, vertex_count));
        }
    }

    pub fn process_vertices_and_render(
        &mut self,
        opengl_app: &mut OpenGlApp,
        extra_vertices: &[Vertex],
        dpi: f64,
        extra_batches: &[([f32; 4], i32, i32)],
    ) {
        // NOTE: Check if the Egui texture has changed and needs rebuilding
        // NOTE: the `ctx.texture()` call will panic if we hadn't
        // called `begin_frame`. But that absolutely should have
        // happened by now.

        self.push_drawcalls_to_display();

        let current_vertex_buffer_capacity = self.vertex_buffer.capacity();
        self.vertex_buffer.clear();
        let display_info = self.display_info(dpi);
        let display_px = display_info.display_px;
        engine::build_vertices(&self.drawcalls, &mut self.vertex_buffer, display_px);

        let vertex_store: &mut dyn engine::VertexStore = &mut self.vertex_buffer;

        let noclip_rect = [
            0.0,
            0.0,
            display_info.window_size_px[0],
            display_info.window_size_px[1],
        ];

        let mut batches = vec![];
        let noclip_vertex_count = vertex_store.count() as i32;
        batches.push((noclip_rect, 0, noclip_vertex_count));
        for &(clip, index, count) in extra_batches {
            batches.push((clip, index + noclip_vertex_count, count));
        }

        for &vertex in extra_vertices {
            vertex_store.push(vertex);
        }

        if self.vertex_buffer.len() > current_vertex_buffer_capacity {
            log::warn!(
                "Warning: vertex count exceeded current capacity {}. Current count: {} ",
                current_vertex_buffer_capacity,
                self.vertex_buffer.len(),
            );
        }

        self.render(opengl_app, dpi, &batches);
    }

    pub fn fullscreen_action(&mut self) -> Option<FullscreenAction> {
        if self.previous_settings.fullscreen == self.settings.fullscreen {
            None
        } else {
            if self.settings.fullscreen {
                log::info!("[{}] Switching to fullscreen", self.current_frame_id);
                Some(FullscreenAction::SwitchToFullscreen)
            } else {
                log::info!("[{}] Switching fullscreen off", self.current_frame_id);
                self.switched_from_fullscreen = true;
                Some(FullscreenAction::SwitchToWindowed)
            }
        }
    }

    pub fn check_window_size_needs_updating(&mut self) -> ResizeWindowAction {
        if self.previous_settings.tile_size != self.settings.tile_size {
            self.change_tilesize_px(self.settings.tile_size);
            // NOTE: we're no longer resizing window on tilesize change
            // if !self.settings.fullscreen {
            //     return ResizeWindowAction::NewSize(self.desired_window_size_px());
            // }
        }
        if self.previous_settings.text_size != self.settings.text_size {
            self.change_text_size_px(self.settings.text_size);
        }
        ResizeWindowAction::NoChange
    }
}
