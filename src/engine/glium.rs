use std::path::Path;
use time::{Duration, PreciseTime};

use glium::{self, DisplayBuild, Surface};
use glium::draw_parameters::DrawParameters;
use glium::glutin::{Event, WindowBuilder};
use glium::glutin::ElementState as PressState;
use glium::glutin::VirtualKeyCode as BackendKey;
use image;
use image::GenericImage;

use color::Color;
use engine::{Draw, UpdateFn, Settings};
use keys::{Key, KeyCode};
use point::Point;


fn gl_color(color: Color, alpha: f32) -> [f32; 4] {
    [color.r as f32, color.g as f32, color.b as f32, alpha]
}


fn key_code_from_backend(backend_code: BackendKey) -> Option<KeyCode> {
    match backend_code {
        BackendKey::Return => Some(KeyCode::Enter),
        BackendKey::Escape => Some(KeyCode::Esc),
        BackendKey::Space => Some(KeyCode::Space),

        BackendKey::Key0 => Some(KeyCode::D0),
        BackendKey::Key1 => Some(KeyCode::D1),
        BackendKey::Key2 => Some(KeyCode::D2),
        BackendKey::Key3 => Some(KeyCode::D3),
        BackendKey::Key4 => Some(KeyCode::D4),
        BackendKey::Key5 => Some(KeyCode::D5),
        BackendKey::Key6 => Some(KeyCode::D6),
        BackendKey::Key7 => Some(KeyCode::D7),
        BackendKey::Key8 => Some(KeyCode::D8),
        BackendKey::Key9 => Some(KeyCode::D9),

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

        BackendKey::Numpad1 => Some(KeyCode::NumPad1),
        BackendKey::Numpad2 => Some(KeyCode::NumPad2),
        BackendKey::Numpad3 => Some(KeyCode::NumPad3),
        BackendKey::Numpad4 => Some(KeyCode::NumPad4),
        BackendKey::Numpad5 => Some(KeyCode::NumPad5),
        BackendKey::Numpad6 => Some(KeyCode::NumPad6),
        BackendKey::Numpad7 => Some(KeyCode::NumPad7),
        BackendKey::Numpad8 => Some(KeyCode::NumPad8),
        BackendKey::Numpad9 => Some(KeyCode::NumPad9),
        BackendKey::Numpad0 => Some(KeyCode::NumPad0),

        _ => None,
    }
}

fn texture_coords_from_char(chr: char) -> (f32, f32) {
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
    (x as f32, y as f32)
}


#[derive(Copy, Clone, Debug)]
struct Vertex {
    /// Position in the tile coordinates.
    ///
    /// Note that this doesn't have to be an integer, so you can
    /// implement smooth positioning by using a fractional value.
    tile_position: [f32; 2],

    /// Index into the texture map. [0, 0] is the top-left corner, the
    /// map's width and height depends on the number of textures in it.
    ///
    /// If a map has 16 textures in a 4x4 square, the top-left index
    /// is [0, 0] and the bottom-right is [3, 3].
    tilemap_index: [f32; 2],

    /// Colour of the glyph. The glyphs are greyscale, so this is how
    /// we set the final colour.
    color: [f32; 4],
}

implement_vertex!(Vertex, tile_position, tilemap_index, color);


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

    // GL setup

    let display = WindowBuilder::new()
        .with_vsync()
        .with_title(window_title)
        .with_dimensions(screen_width, screen_height)
        .build_glium()
        .expect("dose response ERROR: Could not create the window.");

    let program = glium::Program::from_source(
        &display,
        include_str!("../shader_150.glslv"),
        include_str!("../shader_150.glslf"),
        None).unwrap();

    let texture = {
        let image = image::open(font_path).unwrap().to_rgba();
        let (w, h) = image.dimensions();
        assert_eq!(w % tilesize, 0);
        assert_eq!(h % tilesize, 0);
        let image = glium::texture::RawImage2d::from_raw_rgba(
            image.into_raw(), (w, h));
        glium::texture::SrgbTexture2d::new(&display, image).unwrap()
    };

    let (tex_width_px, tex_height_px) = texture.dimensions();
    let texture_tile_count_x = tex_width_px as f32 / tilesize as f32;
    let texture_tile_count_y = tex_height_px as f32 / tilesize as f32;


    let uniforms = uniform! {
        tex: &texture,
        world_dimensions: [display_size.x as f32, display_size.y as f32],
        texture_gl_dimensions: [1.0 / texture_tile_count_x, 1.0 / texture_tile_count_y],
    };


    // Main loop

    let mut settings = Settings {
        fullscreen: false,
    };
    let mut drawcalls = Vec::with_capacity(4000);
    let mut lctrl_pressed = false;
    let mut rctrl_pressed = false;
    let mut lalt_pressed = false;
    let mut ralt_pressed = false;
    let mut lshift_pressed = false;
    let mut rshift_pressed = false;
    let mut vertices = Vec::with_capacity(drawcalls.len() * 6);
    let mut keys = vec![];
    let mut alpha = 1.0;
    let mut previous_frame_time = PreciseTime::now();

    loop {
        let now = PreciseTime::now();
        let dt = previous_frame_time.to(now);
        previous_frame_time = now;

        drawcalls.clear();
        match update(state,
                     dt,
                     display_size,
                     1,  // TODO: FPS
                     &keys,
                     settings,
                     &mut drawcalls) {
            Some((new_settings, new_state)) => {
                state = new_state;
                settings = new_settings;
            },
            None => break,
        };
        keys.clear();

        // Process drawcalls
        vertices.clear();
        for drawcall in &drawcalls {
            match drawcall {
                &Draw::Char(pos, chr, foreground_color) => {
                    let (pos_x, pos_y) = (pos.x as f32, pos.y as f32);
                    let (tilemap_x, tilemap_y) = texture_coords_from_char(chr);
                    let color = gl_color(foreground_color, alpha);

                    vertices.push(Vertex { tile_position: [pos_x,   pos_y],
                                           tilemap_index:  [tilemap_x, tilemap_y],
                                           color: color  });
                    vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y],
                                           tilemap_index:  [tilemap_x + 1.0, tilemap_y],
                                           color: color });
                    vertices.push(Vertex { tile_position: [pos_x,   pos_y + 1.0],
                                           tilemap_index:  [tilemap_x, tilemap_y + 1.0],
                                           color: color });

                    vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y],
                                           tilemap_index:  [tilemap_x + 1.0, tilemap_y],
                                           color: color });
                    vertices.push(Vertex { tile_position: [pos_x,   pos_y + 1.0],
                                           tilemap_index:  [tilemap_x, tilemap_y + 1.0],
                                           color: color });
                    vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y + 1.0],
                                           tilemap_index:  [tilemap_x + 1.0, tilemap_y + 1.0],
                                           color: color });

                }

                &Draw::Background(pos, background_color) => {
                    let (pos_x, pos_y) = (pos.x as f32, pos.y as f32);
                    let (tilemap_x, tilemap_y) = (0.0, 5.0);
                    let color = gl_color(background_color, alpha);

                    vertices.push(Vertex { tile_position: [pos_x,   pos_y],
                                           tilemap_index:  [tilemap_x, tilemap_y],
                                           color: color  });
                    vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y],
                                           tilemap_index:  [tilemap_x + 1.0, tilemap_y],
                                           color: color });
                    vertices.push(Vertex { tile_position: [pos_x,   pos_y + 1.0],
                                           tilemap_index:  [tilemap_x, tilemap_y + 1.0],
                                           color: color });

                    vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y],
                                           tilemap_index:  [tilemap_x + 1.0, tilemap_y],
                                           color: color });
                    vertices.push(Vertex { tile_position: [pos_x,   pos_y + 1.0],
                                           tilemap_index:  [tilemap_x, tilemap_y + 1.0],
                                           color: color });
                    vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y + 1.0],
                                           tilemap_index:  [tilemap_x + 1.0, tilemap_y + 1.0],
                                           color: color });

                }

                &Draw::Text(start_pos, ref text, color) => {
                    for (i, chr) in text.char_indices() {
                        let pos = start_pos + (i as i32, 0);
                        let (pos_x, pos_y) = (pos.x as f32, pos.y as f32);
                        let (tilemap_x, tilemap_y) = texture_coords_from_char(chr);
                        let color = gl_color(color, alpha);

                        vertices.push(Vertex { tile_position: [pos_x,   pos_y],
                                               tilemap_index:  [tilemap_x, tilemap_y],
                                               color: color  });
                        vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y],
                                               tilemap_index:  [tilemap_x + 1.0, tilemap_y],
                                               color: color });
                        vertices.push(Vertex { tile_position: [pos_x,   pos_y + 1.0],
                                               tilemap_index:  [tilemap_x, tilemap_y + 1.0],
                                               color: color });

                        vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y],
                                               tilemap_index:  [tilemap_x + 1.0, tilemap_y],
                                               color: color });
                        vertices.push(Vertex { tile_position: [pos_x,   pos_y + 1.0],
                                               tilemap_index:  [tilemap_x, tilemap_y + 1.0],
                                               color: color });
                        vertices.push(Vertex { tile_position: [pos_x + 1.0,   pos_y + 1.0],
                                               tilemap_index:  [tilemap_x + 1.0, tilemap_y + 1.0],
                                               color: color });
                    }
                }

                &Draw::Rectangle(top_left, dimensions, color) => {
                }

                &Draw::Fade(..) => {
                }
            }
        }

        let vertex_buffer = glium::VertexBuffer::new(&display, &vertices).unwrap();


        // Render
        let mut target = display.draw();
        //target.clear_color(1.0, 0.0, 1.0, 1.0);
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &program,
                    &uniforms,
                    &DrawParameters {
                        blend: glium::Blend::alpha_blending(),
                        .. Default::default()
                    }).unwrap();
        target.finish().unwrap();


        // Process events
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,   // the window has been closed by the user


                Event::KeyboardInput(PressState::Pressed, _, Some(BackendKey::LControl)) => {
                    lctrl_pressed = true;
                }
                Event::KeyboardInput(PressState::Pressed, _, Some(BackendKey::RControl)) => {
                    rctrl_pressed = true;
                }
                Event::KeyboardInput(PressState::Pressed, _, Some(BackendKey::LAlt)) => {
                    lalt_pressed = true;
                }
                Event::KeyboardInput(PressState::Pressed, _, Some(BackendKey::RAlt)) => {
                    ralt_pressed = true;
                }
                Event::KeyboardInput(PressState::Pressed, _, Some(BackendKey::LShift)) => {
                    lshift_pressed = true;
                }
                Event::KeyboardInput(PressState::Pressed, _, Some(BackendKey::RShift)) => {
                    rshift_pressed = true;
                }

                Event::KeyboardInput(PressState::Released, _, Some(BackendKey::LControl)) => {
                    lctrl_pressed = false;
                }
                Event::KeyboardInput(PressState::Released, _, Some(BackendKey::RControl)) => {
                    rctrl_pressed = false;
                }
                Event::KeyboardInput(PressState::Released, _, Some(BackendKey::LAlt)) => {
                    lalt_pressed = false;
                }
                Event::KeyboardInput(PressState::Released, _, Some(BackendKey::RAlt)) => {
                    ralt_pressed = false;
                }
                Event::KeyboardInput(PressState::Released, _, Some(BackendKey::LShift)) => {
                    lshift_pressed = false;
                }
                Event::KeyboardInput(PressState::Released, _, Some(BackendKey::RShift)) => {
                    rshift_pressed = false;
                }

                Event::KeyboardInput(PressState::Pressed, _, Some(key_code)) => {
                    //self.key_events.push((code, press_state));
                    if let Some(code) = key_code_from_backend(key_code) {
                        keys.push(Key {
                            code: code,
                            alt: lalt_pressed || ralt_pressed,
                            ctrl: lctrl_pressed || rctrl_pressed,
                            shift: lshift_pressed || rshift_pressed,
                        });
                    }
                }
                _ => ()
            }
        }

    }
}
