use std::path::Path;

use time::Duration;
use color::Color;
use engine::{Draw, UpdateFn, Settings};
use keys::{Key, KeyCode};
use point::Point;

use piston_window::{PistonWindow, WindowSettings};
use piston_window::{Input, Button};
use piston_window::Key as PistonKey;
use piston_window::{Texture, TextureSettings};
use piston_window::{clear, rectangle};
use image::{self, Rgba, Pixel};


fn from_color_with_alpha(color: Color, alpha: f32) -> [f32; 4] {
    [color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, alpha]
}


fn texture_coords_from_char(chr: char, tilesize: u32) -> (u32, u32) {
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
    (x * tilesize, y * tilesize)
}


fn keycode_from_piston(piston_code: PistonKey) -> Option<KeyCode> {
    match piston_code {
        PistonKey::Return => Some(KeyCode::Enter),
        PistonKey::Escape => Some(KeyCode::Esc),
        PistonKey::Space => Some(KeyCode::Space),

        PistonKey::D0 => Some(KeyCode::D0),
        PistonKey::D1 => Some(KeyCode::D1),
        PistonKey::D2 => Some(KeyCode::D2),
        PistonKey::D3 => Some(KeyCode::D3),
        PistonKey::D4 => Some(KeyCode::D4),
        PistonKey::D5 => Some(KeyCode::D5),
        PistonKey::D6 => Some(KeyCode::D6),
        PistonKey::D7 => Some(KeyCode::D7),
        PistonKey::D8 => Some(KeyCode::D8),
        PistonKey::D9 => Some(KeyCode::D9),

        PistonKey::A => Some(KeyCode::A),
        PistonKey::B => Some(KeyCode::B),
        PistonKey::C => Some(KeyCode::C),
        PistonKey::D => Some(KeyCode::D),
        PistonKey::E => Some(KeyCode::E),
        PistonKey::F => Some(KeyCode::F),
        PistonKey::G => Some(KeyCode::G),
        PistonKey::H => Some(KeyCode::H),
        PistonKey::I => Some(KeyCode::I),
        PistonKey::J => Some(KeyCode::J),
        PistonKey::K => Some(KeyCode::K),
        PistonKey::L => Some(KeyCode::L),
        PistonKey::M => Some(KeyCode::M),
        PistonKey::N => Some(KeyCode::N),
        PistonKey::O => Some(KeyCode::O),
        PistonKey::P => Some(KeyCode::P),
        PistonKey::Q => Some(KeyCode::Q),
        PistonKey::R => Some(KeyCode::R),
        PistonKey::S => Some(KeyCode::S),
        PistonKey::T => Some(KeyCode::T),
        PistonKey::U => Some(KeyCode::U),
        PistonKey::V => Some(KeyCode::V),
        PistonKey::W => Some(KeyCode::W),
        PistonKey::X => Some(KeyCode::X),
        PistonKey::Y => Some(KeyCode::Y),
        PistonKey::Z => Some(KeyCode::Z),

        PistonKey::F1 => Some(KeyCode::F1),
        PistonKey::F2 => Some(KeyCode::F2),
        PistonKey::F3 => Some(KeyCode::F3),
        PistonKey::F4 => Some(KeyCode::F4),
        PistonKey::F5 => Some(KeyCode::F5),
        PistonKey::F6 => Some(KeyCode::F6),
        PistonKey::F7 => Some(KeyCode::F7),
        PistonKey::F8 => Some(KeyCode::F8),
        PistonKey::F9 => Some(KeyCode::F9),
        PistonKey::F10 => Some(KeyCode::F10),
        PistonKey::F11 => Some(KeyCode::F11),
        PistonKey::F12 => Some(KeyCode::F12),

        PistonKey::Right => Some(KeyCode::Right),
        PistonKey::Left => Some(KeyCode::Left),
        PistonKey::Down => Some(KeyCode::Down),
        PistonKey::Up => Some(KeyCode::Up),

        PistonKey::NumPad1 => Some(KeyCode::NumPad1),
        PistonKey::NumPad2 => Some(KeyCode::NumPad2),
        PistonKey::NumPad3 => Some(KeyCode::NumPad3),
        PistonKey::NumPad4 => Some(KeyCode::NumPad4),
        PistonKey::NumPad5 => Some(KeyCode::NumPad5),
        PistonKey::NumPad6 => Some(KeyCode::NumPad6),
        PistonKey::NumPad7 => Some(KeyCode::NumPad7),
        PistonKey::NumPad8 => Some(KeyCode::NumPad8),
        PistonKey::NumPad9 => Some(KeyCode::NumPad9),
        PistonKey::NumPad0 => Some(KeyCode::NumPad0),

        _ => None,
    }
}


pub fn main_loop<T>(display_size: Point,
                    default_background: Color,
                    window_title: &str,
                    font_path: &Path,
                    mut state: T,
                    update: UpdateFn<T>)
{
    let tilesize = 16;  // TODO: don't hardcode this value -- calculate it from the tilemap.
    let (screen_width, screen_height) = (display_size.x as u32 * tilesize as u32,
                                         display_size.y as u32 * tilesize as u32);
    let mut window: PistonWindow = WindowSettings::new(window_title,
                                                       (screen_width, screen_height))
        .build()
        .unwrap();

    let tileimage = image::open(font_path).expect(
        &format!("Could not load the font map at: '{}'", font_path.display()));
    let mut surface = image::ImageBuffer::new(screen_width, screen_height);
    let mut surface_texture = Texture::from_image(&mut window.factory, &surface, &TextureSettings::new()).unwrap();

    //let mut factory = window.factory.clone();

    let mut settings = Settings {
        fullscreen: false,
    };
    let mut drawcalls = Vec::with_capacity(8192);
    let mut lctrl_pressed = false;
    let mut rctrl_pressed = false;
    let mut lalt_pressed = false;
    let mut ralt_pressed = false;
    let mut lshift_pressed = false;
    let mut rshift_pressed = false;

    let mut keys = vec![];
    let alpha = 255;
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
                // println!("drawcalls: {}", drawcalls.len());
                keys.clear();

                drawcalls.sort_by(|a, b| {
                    use std::cmp::Ordering::*;
                    use engine::Draw::*;

                    match (a, b) {
                        (&Char(..), &Char(..)) => Equal,
                        (&Background(..), &Background(..)) => Equal,
                        (&Text(..), &Text(..)) => Equal,
                        (&Rectangle(..), &Rectangle(..)) => Equal,
                        (&Fade(..), &Fade(..)) => Equal,

                        (&Fade(..), _) => Less,
                        (_, &Fade(..)) => Greater,

                        (&Background(..), &Char(..)) => Less,
                        (&Background(..), &Text(..)) => Less,
                        (&Background(..), &Rectangle(..)) => Less,

                        (&Char(..), &Background(..)) => Greater,
                        (&Text(..), &Background(..)) => Greater,
                        (&Rectangle(..), &Background(..)) => Greater,

                        _ => Equal,
                    }
                });
            }

            Input::Press(Button::Keyboard(PistonKey::LCtrl)) => {
                lctrl_pressed = true;
            }
            Input::Press(Button::Keyboard(PistonKey::RCtrl)) => {
                rctrl_pressed = true;
            }
            Input::Press(Button::Keyboard(PistonKey::LAlt)) => {
                lalt_pressed = true;
            }
            Input::Press(Button::Keyboard(PistonKey::RAlt)) => {
                ralt_pressed = true;
            }
            Input::Press(Button::Keyboard(PistonKey::LShift)) => {
                lshift_pressed = true;
            }
            Input::Press(Button::Keyboard(PistonKey::RShift)) => {
                rshift_pressed = true;
            }

            Input::Release(Button::Keyboard(PistonKey::LCtrl)) => {
                lctrl_pressed = false;
            }
            Input::Release(Button::Keyboard(PistonKey::RCtrl)) => {
                rctrl_pressed = false;
            }
            Input::Release(Button::Keyboard(PistonKey::LAlt)) => {
                lalt_pressed = false;
            }
            Input::Release(Button::Keyboard(PistonKey::RAlt)) => {
                ralt_pressed = false;
            }
            Input::Release(Button::Keyboard(PistonKey::LShift)) => {
                lshift_pressed = false;
            }
            Input::Release(Button::Keyboard(PistonKey::RShift)) => {
                rshift_pressed = false;
            }

            Input::Press(Button::Keyboard(key_code)) => {
                if let Some(code) = keycode_from_piston(key_code) {
                    keys.push(Key {
                        code: code,
                        alt: lalt_pressed || ralt_pressed,
                        ctrl: lctrl_pressed || rctrl_pressed,
                        shift: lshift_pressed || rshift_pressed,
                    });
                }
            }

            // RenderArgs{ext_dt, width, height, draw_width, draw_height}
            Input::Render(render_args) => {
                use image::GenericImage;
                //println!("ext_dt: {:?}", render_args.ext_dt);

                fn blit_char<I, J>(src_x: u32, src_y: u32,
                                   source: &I,
                                   dst_x: u32, dst_y: u32,
                                   destination: &mut J,
                                   tilesize: u32,
                                   color: Color,
                                   alpha: u8)
                    where I: GenericImage<Pixel=image::Rgba<u8>>,
                          J: GenericImage<Pixel=image::Rgba<u8>>,
                {
                    for x in 0..tilesize {
                        for y in 0..tilesize {
                            let pixel = source.get_pixel(src_x + x, src_y + y).to_rgba();
                            let result = Rgba {
                                data: [
                                    ((color.r as f32 / 255.0) * (pixel.data[0] as f32 / 255.0) * 255.0) as u8,
                                    ((color.g as f32 / 255.0) * (pixel.data[1] as f32 / 255.0) * 255.0) as u8,
                                    ((color.b as f32 / 255.0) * (pixel.data[2] as f32 / 255.0) * 255.0) as u8,
                                    ((alpha as f32 / 255.0) * (pixel.data[3] as f32 / 255.0) * 255.0) as u8,
                                ]
                            };
                            destination.put_pixel(dst_x + x, dst_y + y, result);
                        }
                    }
                }

                fn draw_rect<I>(src_x: u32, src_y: u32,
                                destination: &mut I,
                                tilesize: u32,
                                color: Color,
                                alpha: u8)
                    where I: GenericImage<Pixel=image::Rgba<u8>>,
                {
                    let pixel = Rgba {
                        data: [
                            color.r,
                            color.g,
                            color.b,
                            alpha,
                        ]
                    };
                    for x in 0..tilesize {
                        for y in 0..tilesize {
                            destination.put_pixel(src_x + x, src_y + y, pixel);
                        }
                    }
                }

                // NOTE: in debug mode, this is slowing things down a lot.
                for drawcall in &drawcalls {
                    match drawcall {
                        &Draw::Char(pos, chr, foreground_color) => {
                            let source_rectangle = texture_coords_from_char(chr, tilesize);
                            blit_char(source_rectangle.0, source_rectangle.1,
                                      &tileimage,
                                      pos.x as u32 * tilesize, pos.y as u32 * tilesize,
                                      &mut surface,
                                      tilesize,
                                      foreground_color,
                                      alpha);
                        }

                        &Draw::Background(pos, background_color) => {
                            draw_rect(pos.x as u32 * tilesize, pos.y as u32 * tilesize,
                                      &mut surface, tilesize, background_color, alpha);
                        }

                        &Draw::Text(start_pos, ref text, color) => {
                            for (i, chr) in text.char_indices() {
                                let pos = start_pos + (i as i32, 0);
                                let source_rectangle = texture_coords_from_char(chr, tilesize);
                                blit_char(source_rectangle.0, source_rectangle.1,
                                          &tileimage,
                                          pos.x as u32 * tilesize, pos.y as u32 * tilesize,
                                          &mut surface,
                                          tilesize,
                                          color,
                                          alpha);
                            }
                        }

                        &Draw::Rectangle(top_left, dimensions, color) => {
                            let pixel = Rgba {
                                data: [
                                    color.r,
                                    color.g,
                                    color.b,
                                    alpha,
                                ]
                            };
                            for x in 0..(dimensions.x as u32 * tilesize) {
                                for y in 0..(dimensions.y as u32 * tilesize) {
                                    surface.put_pixel(top_left.x as u32 * tilesize + x,
                                                      top_left.y as u32 * tilesize + y,
                                                      pixel);
                                }
                            }
                        }

                        &Draw::Fade(..) => {
                        }
                    }
                }


                // NOTE: in debug mode, this is slowing things down a lot.
                surface_texture.update(&mut window.encoder, &surface).unwrap();

                window.draw_2d(&event, |c, g| {
                    clear(fade_color, g);

                    // NOTE: Render the default background
                    rectangle(from_color_with_alpha(default_background, alpha as f32 / 255.0),
                              [0.0, 0.0,
                               render_args.draw_width as f64,
                               render_args.draw_height as f64],
                              c.transform,
                              g);

                    ::piston_window::image(&surface_texture, c.transform, g);

                });
            }

            _ => {}
        }
    }

}
