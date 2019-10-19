use crate::{
    color::Color,
    engine::{self, opengl::OpenGlApp, Display, DisplayInfo, Drawcall, Mouse, Settings, Vertex},
    keys::Key,
    point::Point,
    state::State,
};

use image::RgbaImage;

pub struct LoopState {
    pub settings: Settings,
    pub previous_settings: Settings,
    pub game_display_size: Point,
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
}

impl LoopState {
    pub fn initialise(
        settings: Settings,
        game_display_size: Point,
        default_background: Color,
        game_state: Box<State>,
    ) -> Self {
        log::debug!(
            "Requested display in tiles: {} x {}",
            game_display_size.x,
            game_display_size.y
        );

        assert_eq!(
            std::mem::size_of::<Vertex>(),
            engine::VERTEX_COMPONENT_COUNT * 4
        );

        let padding = Point::from_i32(game_display_size.y / 2);
        let display = Display::new(game_display_size, padding, settings.tile_size);
        let image = {
            let data = &include_bytes!(concat!(env!("OUT_DIR"), "/font.png"))[..];
            image::load_from_memory_with_format(data, image::PNG)
                .unwrap()
                .to_rgba()
        };
        log::debug!("Loaded font image.");

        // Always stard from a windowed mode. This will force the
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
        let window_size_px = game_display_size * settings.tile_size;
        log::debug!(
            "Desired window size: {} x {}",
            window_size_px.x,
            window_size_px.y
        );
        Self {
            settings,
            previous_settings,
            game_display_size,
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

    // TODO: add units!!
    pub fn desired_window_size(&self) -> (u32, u32) {
        (
            self.game_display_size.x as u32 * self.settings.tile_size as u32,
            self.game_display_size.y as u32 * self.settings.tile_size as u32,
        )
    }

    pub fn resize_window(&mut self, new_width: i32, new_height: i32) {
        log::info!("Window resized to: {} x {}", new_width, new_height);
        let new_window_size_px = Point::new(new_width, new_height);
        if self.window_size_px != new_window_size_px {
            self.window_size_px = new_window_size_px;

            // NOTE: Update the tilesize if we get a perfect match
            if new_height > 0 && new_height % crate::DISPLAY_SIZE.y == 0 {
                let new_tilesize = new_height / crate::DISPLAY_SIZE.y;
                self.change_tilesize(new_tilesize);
            };
        }
    }

    // TODO: units? I think it's pixels. Make sure that's in the name
    pub fn change_tilesize(&mut self, new_tilesize: i32) {
        if crate::engine::AVAILABLE_FONT_SIZES.contains(&(new_tilesize as i32)) {
            log::info!(
                "Changing tilesize from {} to {}",
                self.display.tilesize,
                new_tilesize
            );
            self.display.tilesize = new_tilesize;
            self.settings.tile_size = new_tilesize;
        } else {
            log::warn!(
            "Trying to switch to a tilesize that's not available: {}. Only these ones exist: {:?}",
            new_tilesize,
            crate::engine::AVAILABLE_FONT_SIZES
            );
        }
    }

    pub fn display_info(&self, dpi: f64) -> DisplayInfo {
        // TODO(shadower): is this the right way to use the `dpi`? I'm
        // guessing we should just be honest about `window_size_px`
        // everywhere.
        engine::calculate_display_info(
            [
                self.window_size_px.x as f32 * dpi as f32,
                self.window_size_px.y as f32 * dpi as f32,
            ],
            self.game_display_size,
            self.settings.tile_size,
        )
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

    pub fn check_vertex_buffer_capacity(&self) {
        if self.vertex_buffer.len() > engine::VERTEX_BUFFER_CAPACITY {
            log::warn!(
                "Warning: vertex count exceeded initial capacity {}. Current count: {} ",
                engine::VERTEX_BUFFER_CAPACITY,
                self.vertex_buffer.len(),
            );
        }
    }
}
