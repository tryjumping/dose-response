use crate::{
    color::Color,
    engine::{opengl::OpenGlApp, Display, DisplayInfo, Settings},
    point::Point,
};

use image::RgbaImage;

pub struct LoopState {
    pub settings: Settings,
    pub game_display_size: Point,
    pub display: Display,
    pub image: RgbaImage,
    pub default_background: Color,
}

impl LoopState {
    pub fn initialise(
        settings: Settings,
        game_display_size: Point,
        default_background: Color,
    ) -> Self {
        log::debug!(
            "Requested display in tiles: {} x {}",
            game_display_size.x,
            game_display_size.y
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

        let result = Self {
            settings,
            game_display_size,
            display,
            image,
            default_background,
        };

        let desired = result.desired_window_size();
        log::debug!("Desired window size: {} x {}", desired.0, desired.1,);

        result
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

    pub fn render(
        &self,
        gl: &OpenGlApp,
        default_background: Color,
        display_info: DisplayInfo,
        vertex_buffer: &[f32],
    ) {
        let texture_size_px = {
            let (width, height) = self.image.dimensions();
            [width as f32, height as f32]
        };
        gl.render(
            default_background,
            display_info,
            texture_size_px,
            vertex_buffer,
        );
    }
}
