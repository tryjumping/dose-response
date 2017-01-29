use std::path::Path;

use time::Duration;
use color::Color;
use engine::{Draw, UpdateFn, Settings};
use keys::{Key, KeyCode};
use point::Point;

use piston_window::{PistonWindow, WindowSettings, Transformed, Glyphs};
use piston_window::{Input, Button, MouseButton, Motion};
use piston_window::Key as PistonKey;
use piston_window::{Texture, Flip, TextureSettings};
use piston_window::{clear, text, image, rectangle};


fn from_color(color: Color) -> [f32; 4] {
    [color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, 1.0]
}


pub fn main_loop<T>(display_size: Point,
                    default_background: Color,
                    window_title: &str,
                    font_path: &Path,
                    mut state: T,
                    update: UpdateFn<T>)
{
    // TODO remove this
    let (screen_width, screen_height) = (1024, 768);
    let mut window: PistonWindow = WindowSettings::new(window_title,
                                                       (screen_width, screen_height))
        .build()
        .unwrap();

    let tileset = Texture::from_path(
        &mut window.factory, font_path, Flip::None, &TextureSettings::new()).expect(
        &format!("Could not load the font map at: '{}'", font_path.display()));

    while let Some(event) = window.next() {
        // http://docs.piston.rs/piston_window/input/enum.Event.html
        match event {
            Input::Update(update_args) => {
                let dt = update_args.dt;
            }

            Input::Release(Button::Keyboard(PistonKey::Q)) => {
                break;
            }

            // RenderArgs{ext_dt, width, height, draw_width, draw_height}
            Input::Render(_render_args) => {
                window.draw_2d(&event, |c, g| {
                    clear(from_color(default_background), g);

                    rectangle([0.0, 1.0, 0.0, 1.0],
                              [0.0, 0.0, 16.0, 16.0],
                              c.transform,
                              g,
                    );

                    rectangle([0.0, 1.0, 0.0, 1.0],
                              [32.0, 32.0, 16.0, 16.0],
                              c.transform,
                              g,
                    );

                    // TODO: this isn't getting blended!
                    // It's because our source images don't set transparent colors.
                    image::Image::new_color([1.0, 0.0, 1.0, 1.0])
                        .src_rect([0.0, 48.0, 16.0, 16.0])
                        .rect([0.0, 0.0, 16.0, 16.0])
                        .draw(&tileset, &c.draw_state, c.transform, g);

                    image::Image::new_color([0.0, 0.0, 1.0, 1.0])
                        .src_rect([16.0, 48.0, 16.0, 16.0])
                        .rect([16.0, 0.0, 16.0, 16.0])
                        .draw(&tileset, &c.draw_state, c.transform, g);
                });
            }

            _ => {}
        }
    }

}
