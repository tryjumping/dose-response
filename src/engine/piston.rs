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
    from_color_with_alpha(color, 1.0)
}

fn from_color_with_alpha(color: Color, alpha: f32) -> [f32; 4] {
    [color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, alpha]
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
    let tilesize = 16.0;  // TODO: don't hardcode this value -- calculate it from the tilemap.


    let mut settings = Settings {
        fullscreen: false,
    };
    let mut drawcalls = Vec::with_capacity(8192);
    let mut keys = vec![];
    let mut alpha = 1.0;
    let fade_color = [1.0, 0.0, 1.0, 1.0];

    while let Some(event) = window.next() {
        // http://docs.piston.rs/piston_window/input/enum.Event.html
        match event {
            Input::Update(update_args) => {
                drawcalls.clear();
                match update(state,
                             Duration::microseconds((update_args.dt * 1_000_000.0) as i64),
                             display_size,
                             1, // TODO: FPS
                             &keys,
                             settings,
                             &mut drawcalls) {
                    Some((new_settings, new_state)) => {
                        state = new_state;
                        settings = new_settings;
                    },
                    None => break,
                };
            }

            Input::Release(Button::Keyboard(PistonKey::Q)) => {
                break;
            }

            // RenderArgs{ext_dt, width, height, draw_width, draw_height}
            Input::Render(render_args) => {
                println!("ext_dt: {:?}", render_args.ext_dt);
                window.draw_2d(&event, |c, g| {
                    clear(fade_color, g);

                    // NOTE: Render the default background
                    rectangle(from_color_with_alpha(default_background, alpha),
                              [0.0, 0.0,
                               render_args.draw_width as f64,
                               render_args.draw_height as f64],
                              c.transform,
                              g);

                    rectangle([0.0, 1.0, 0.0, alpha],
                              [0.0, 0.0, tilesize, tilesize],
                              c.transform,
                              g);

                    rectangle([0.0, 1.0, 0.0, alpha],
                              [32.0, 32.0, tilesize, tilesize],
                              c.transform,
                              g);

                    image::Image::new_color([1.0, 0.0, 1.0, alpha])
                        .src_rect([0.0, 48.0, tilesize, tilesize])
                        .rect([0.0, 0.0, tilesize, tilesize])
                        .draw(&tileset, &c.draw_state, c.transform, g);

                    image::Image::new_color([0.0, 0.0, 1.0, alpha])
                        .src_rect([16.0, 48.0, tilesize, tilesize])
                        .rect([16.0, 0.0, tilesize, tilesize])
                        .draw(&tileset, &c.draw_state, c.transform, g);


                    for drawcall in &drawcalls {
                        match drawcall {
                            &Draw::Char(pos, chr, foreground_color) => {
                                // TODO: get the correct character mapping here!
                                image::Image::new_color(
                                    from_color_with_alpha(foreground_color, alpha))
                                    .src_rect([0.0, 48.0, tilesize, tilesize])
                                    .rect([pos.x as f64 * tilesize, pos.y as f64 * tilesize,
                                           tilesize, tilesize])
                                    .draw(&tileset, &c.draw_state, c.transform, g);
                            }

                            &Draw::Background(..) | &Draw::Text(..) | &Draw::Rectangle(..) | &Draw::Fade(..) => {}
                        }
                    }
                });
            }

            _ => {}
        }
    }

}
