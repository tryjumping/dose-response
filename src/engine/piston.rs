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


fn source_rectangle_from_char(chr: char, tilesize: f64) -> [f64; 4] {
    let (x, y) = match chr {
        ' ' => (0, 0),
        '!' => (1, 0),
        '"' => (2, 0),
        '#' => (3, 0),
        '$' => (4, 0),
        '%' => (5, 0),
        '&' => (6, 0),
        '\'' => (7, 0),
        '(' => (8, 0),
        ')' => (9, 0),
        '*' => (10, 0),
        '+' => (11, 0),
        ',' => (12, 0),
        '-' => (13, 0),
        '.' => (14, 0),
        '/' => (15, 0),
        '0' => (16, 0),
        '1' => (17, 0),
        '2' => (18, 0),
        '3' => (19, 0),
        '4' => (20, 0),
        '5' => (21, 0),
        '6' => (22, 0),
        '7' => (23, 0),
        '8' => (24, 0),
        '9' => (25, 0),
        ':' => (26, 0),
        ';' => (27, 0),
        '<' => (28, 0),
        '=' => (29, 0),
        '>' => (30, 0),
        '?' => (31, 0),

        '@' => (0, 1),
        '[' => (1, 1),
        '\\' => (2, 1),
        ']' => (3, 1),
        '^' => (4, 1),
        '_' => (5, 1),
        '`' => (6, 1),
        '{' => (7, 1),
        '|' => (8, 1),
        '}' => (9, 1),
        '~' => (10, 1),

        // TODO: the graphical characters

        'A' => (0, 3),
        'B' => (1, 3),
        'C' => (2, 3),
        'D' => (3, 3),
        'E' => (4, 3),
        'F' => (5, 3),
        'G' => (6, 3),
        'H' => (7, 3),
        'I' => (8, 3),
        'J' => (9, 3),
        'K' => (10, 3),
        'L' => (11, 3),
        'M' => (12, 3),
        'N' => (13, 3),
        'O' => (14, 3),
        'P' => (15, 3),
        'Q' => (16, 3),
        'R' => (17, 3),
        'S' => (18, 3),
        'T' => (19, 3),
        'U' => (20, 3),
        'V' => (21, 3),
        'W' => (22, 3),
        'X' => (23, 3),
        'Y' => (24, 3),
        'Z' => (25, 3),

        'a' => (0, 4),
        'b' => (1, 4),
        'c' => (2, 4),
        'd' => (3, 4),
        'e' => (4, 4),
        'f' => (5, 4),
        'g' => (6, 4),
        'h' => (7, 4),
        'i' => (8, 4),
        'j' => (9, 4),
        'k' => (10, 4),
        'l' => (11, 4),
        'm' => (12, 4),
        'n' => (13, 4),
        'o' => (14, 4),
        'p' => (15, 4),
        'q' => (16, 4),
        'r' => (17, 4),
        's' => (18, 4),
        't' => (19, 4),
        'u' => (20, 4),
        'v' => (21, 4),
        'w' => (22, 4),
        'x' => (23, 4),
        'y' => (24, 4),
        'z' => (25, 4),

        _ => (0, 0),
    };
    [x as f64 * tilesize, y as f64 * tilesize, tilesize, tilesize]
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
                                let source_rectangle = source_rectangle_from_char(chr, tilesize);
                                image::Image::new_color(
                                    from_color_with_alpha(foreground_color, alpha))
                                    .src_rect(source_rectangle)
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
