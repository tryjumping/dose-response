use crate::{
    color::Color,
    engine::{self, opengl::OpenGlApp, Display, DisplayInfo, Drawcall, Mouse, TextMetrics, Vertex},
    keys::Key,
    point::Point,
    settings::{Settings, Store as SettingsStore},
    state::State,
    util,
};

use std::time::Duration;

use image::RgbaImage;

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
    tile_width_px: i32,
}

impl TextMetrics for Metrics {
    fn tile_width_px(&self) -> i32 {
        self.tile_width_px
    }
}

pub struct LoopState {
    pub settings: Settings,
    pub previous_settings: Settings,
    pub window_size_px: Point,
    pub display: Display,
    pub image: RgbaImage,
    pub default_background: Color,
    pub drawcalls: Vec<Drawcall>,
    pub overall_max_drawcall_count: usize,
    pub vertex_buffer: Vec<f32>,
    pub game_state: Box<State>,
    pub mouse: Mouse,
    pub keys: Vec<Key>,
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
        game_display_size_tiles: Point,
        default_background: Color,
        game_state: Box<State>,
    ) -> Self {
        log::debug!(
            "Requested display in tiles: {} x {}",
            game_display_size_tiles.x,
            game_display_size_tiles.y
        );

        assert_eq!(
            std::mem::size_of::<Vertex>(),
            engine::VERTEX_COMPONENT_COUNT * 4
        );

        let display = Display::new(game_display_size_tiles, settings.tile_size);
        let image = {
            let data = &include_bytes!(concat!(env!("OUT_DIR"), "/font.png"))[..];
            image::load_from_memory_with_format(data, image::PNG)
                .unwrap()
                .to_rgba()
        };
        log::debug!("Loaded font image.");

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
        let window_size_px = game_display_size_tiles * settings.tile_size;
        log::debug!(
            "Desired window size: {} x {}",
            window_size_px.x,
            window_size_px.y
        );
        Self {
            settings,
            previous_settings,
            display,
            image,
            default_background,
            window_size_px,
            drawcalls: Vec::with_capacity(engine::DRAWCALL_CAPACITY),
            overall_max_drawcall_count: 0,
            vertex_buffer: Vec::with_capacity(engine::VERTEX_BUFFER_CAPACITY),
            game_state,
            mouse: Mouse::new(),
            keys: vec![],
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
        let opengl_app = OpenGlApp::new(vs_source, fs_source);
        log::debug!("Created opengl app.");

        let image_size = self.image.dimensions();
        opengl_app.initialise(image_size, &self.image);
        log::debug!("Initialised opengl app.");
        opengl_app
    }

    pub fn desired_window_size_px(&self) -> (u32, u32) {
        let result = self.display.size_without_padding() * self.settings.tile_size;
        (result.x as u32, result.y as u32)
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
        let update_result = crate::game::update(
            &mut self.game_state,
            dt,
            self.fps,
            &self.keys,
            self.mouse,
            &mut self.settings,
            &Metrics { tile_width_px },
            settings_store,
            &mut self.display,
        );

        match update_result {
            RunningState::Running => {}
            RunningState::NewGame(new_state) => {
                self.game_state = new_state;
            }
            RunningState::Stopped => return UpdateResult::QuitRequested,
        }

        self.reset_inputs();

        UpdateResult::KeepGoing
    }

    pub fn handle_window_size_changed(&mut self, new_width: i32, new_height: i32) {
        log::info!("Window resized to: {} x {}", new_width, new_height);
        let new_window_size_px = Point::new(new_width, new_height);
        if self.window_size_px != new_window_size_px {
            self.window_size_px = new_window_size_px;
            let new_display_size_tiles = Point::new(
                new_window_size_px.x / self.settings.tile_size,
                new_window_size_px.y / self.settings.tile_size,
            );
            self.display = Display::new(new_display_size_tiles, self.settings.tile_size);
        }
    }

    pub fn change_tilesize_px(&mut self, new_tilesize_px: i32) {
        if crate::engine::AVAILABLE_FONT_SIZES.contains(&(new_tilesize_px as i32)) {
            log::info!(
                "Changing tilesize from {} to {}",
                self.display.tilesize,
                new_tilesize_px
            );
            self.settings.tile_size = new_tilesize_px;
            // Recreate the display, because the tile count is now different:
            let new_display_size_tiles = Point::new(
                self.window_size_px.x / self.settings.tile_size,
                self.window_size_px.y / self.settings.tile_size,
            );
            self.display = Display::new(new_display_size_tiles, self.settings.tile_size);
        } else {
            log::warn!(
            "Trying to switch to a tilesize that's not available: {}. Only these ones exist: {:?}",
            new_tilesize_px,
            crate::engine::AVAILABLE_FONT_SIZES
            );
        }
    }

    pub fn display_info(&self, dpi: f64) -> DisplayInfo {
        engine::calculate_display_info(
            [self.window_size_px.x as f32, self.window_size_px.y as f32],
            self.display.size_without_padding(),
            self.settings.tile_size,
            dpi as f32,
        )
    }

    pub fn reset_inputs(&mut self) {
        self.mouse.left_clicked = false;
        self.mouse.right_clicked = false;
        self.keys.clear();
    }

    pub fn update_mouse_position(&mut self, dpi: f64, window_px_x: i32, window_px_y: i32) {
        let display_info = self.display_info(dpi);

        let (x, y) = (
            window_px_x - (display_info.extra_px[0] / 2.0) as i32,
            window_px_y - (display_info.extra_px[1] / 2.0) as i32,
        );
        let x = util::clamp(0, x, display_info.display_px[0] as i32 - 1);
        let y = util::clamp(0, y, display_info.display_px[1] as i32 - 1);

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
        self.drawcalls.clear();
        self.display.push_drawcalls(&mut self.drawcalls);

        if self.drawcalls.len() > self.overall_max_drawcall_count {
            self.overall_max_drawcall_count = self.drawcalls.len();
        }

        if self.drawcalls.len() > engine::DRAWCALL_CAPACITY {
            log::warn!(
                "Warning: drawcall count exceeded initial capacity {}. Current count: {}.",
                engine::DRAWCALL_CAPACITY,
                self.drawcalls.len(),
            );
        }
    }

    pub fn render(&self, gl: &OpenGlApp, dpi: f64) {
        let texture_size_px = {
            let (width, height) = self.image.dimensions();
            [width as f32, height as f32]
        };
        gl.render(
            self.default_background,
            self.display_info(dpi),
            texture_size_px,
            &self.vertex_buffer,
        );
    }

    pub fn process_vertices_and_render(&mut self, opengl_app: &OpenGlApp, dpi: f64) {
        self.push_drawcalls_to_display();

        self.vertex_buffer.clear();
        let display_px = self.display_info(dpi).display_px;
        engine::build_vertices(&self.drawcalls, &mut self.vertex_buffer, display_px);
        self.check_vertex_buffer_capacity();

        self.render(&opengl_app, dpi);
    }

    pub fn check_vertex_buffer_capacity(&self) {
        if self.vertex_buffer.len() > engine::VERTEX_BUFFER_CAPACITY {
            log::warn!(
                "Warning: vertex count exceeded initial capacity {}. Current count: {} ",
                engine::VERTEX_BUFFER_CAPACITY,
                self.vertex_buffer.len(),
            );
        }
    }

    pub fn fullscreen_action(&mut self) -> Option<FullscreenAction> {
        if self.previous_settings.fullscreen != self.settings.fullscreen {
            if self.settings.fullscreen {
                log::info!("[{}] Switching to fullscreen", self.current_frame_id);
                Some(FullscreenAction::SwitchToFullscreen)
            } else {
                log::info!("[{}] Switching fullscreen off", self.current_frame_id);
                self.switched_from_fullscreen = true;
                Some(FullscreenAction::SwitchToWindowed)
            }
        } else {
            None
        }
    }

    pub fn check_window_size_needs_updating(&mut self) -> ResizeWindowAction {
        if self.previous_settings.tile_size != self.settings.tile_size {
            self.change_tilesize_px(self.settings.tile_size);
            if !self.settings.fullscreen {
                return ResizeWindowAction::NewSize(self.desired_window_size_px());
            }
        }
        ResizeWindowAction::NoChange
    }
}
