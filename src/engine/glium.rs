use time::{Duration, PreciseTime};

use glium::{self, DisplayBuild, Surface};
use glium::draw_parameters::DrawParameters;
use glium::glutin::{Event, WindowBuilder};
use glium::glutin::ElementState as PressState;
use glium::glutin::VirtualKeyCode as BackendKey;
use image;

use color::Color;
use engine::{Draw, UpdateFn, Settings};
use keys::{Key, KeyCode};
use point::Point;

use self::vertex::Vertex;

// NOTE: This is designed specifically to deduplicated characters on
// the same position (using Vec::dedup). So the only thing considered
// equal are characters with the same pos value.
impl PartialEq for Draw {
    fn eq(&self, other: &Self) -> bool {
        use engine::Draw::*;
        match (self, other) {
            (&Char(p1, ..), &Char(p2, ..)) => p1 == p2,
            _ => false,
        }
    }
}

fn gl_color(color: Color, alpha: f32) -> [f32; 4] {
    [color.r as f32 / 255.0,
     color.g as f32 / 255.0,
     color.b as f32 / 255.0,
     alpha]
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
    let (x, y) = super::texture_coords_from_char(chr).unwrap_or((0, 0));
    (x as f32, y as f32)
}


#[allow(unsafe_code)]
mod vertex {
    #[derive(Copy, Clone, Debug)]
    pub struct Vertex {
        /// Position in the tile coordinates.
        ///
        /// Note that this doesn't have to be an integer, so you can
        /// implement smooth positioning by using a fractional value.
        pub tile_position: [f32; 2],

        /// Index into the texture map. [0, 0] is the top-left corner, the
        /// map's width and height depends on the number of textures in it.
        ///
        /// If a map has 16 textures in a 4x4 square, the top-left index
        /// is [0, 0] and the bottom-right is [3, 3].
        pub tilemap_index: [f32; 2],

        /// Colour of the glyph. The glyphs are greyscale, so this is how
        /// we set the final colour.
        pub color: [f32; 4],
    }
    implement_vertex!(Vertex, tile_position, tilemap_index, color);
}


pub fn main_loop<T>(display_size: Point,
                    default_background: Color,
                    window_title: &str,
                    mut state: T,
                    update: UpdateFn<T>) {
    let tilesize = 16; // TODO: don't hardcode this value -- calculate it from the tilemap.
    let (screen_width, screen_height) = (display_size.x as u32 * tilesize as u32,
                                         display_size.y as u32 * tilesize as u32);

    // GL setup

    let display = WindowBuilder::new()
        .with_vsync()
        .with_title(window_title)
        .with_dimensions(screen_width, screen_height)
        .build_glium()
        .expect("dose response ERROR: Could not create the window.");

    let program = program!(&display,
                           150 => {
                               outputs_srgb: true,
                               vertex: include_str!("../shader_150.glslv"),
                               fragment: include_str!("../shader_150.glslf")
                           }
        )
            .unwrap();

    let texture = {
        use std::io::Cursor;
        let data = &include_bytes!(concat!(env!("OUT_DIR"), "/font.png"))[..];
        let image = image::load(Cursor::new(data), image::PNG)
            .unwrap()
            .to_rgba();
        let (w, h) = image.dimensions();
        assert_eq!(w % tilesize, 0);
        assert_eq!(h % tilesize, 0);
        let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), (w, h));
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

    let mut settings = Settings { fullscreen: false };
    let mut drawcalls = Vec::with_capacity(4000);
    let mut lctrl_pressed = false;
    let mut rctrl_pressed = false;
    let mut lalt_pressed = false;
    let mut ralt_pressed = false;
    let mut lshift_pressed = false;
    let mut rshift_pressed = false;
    let mut vertices = Vec::with_capacity(drawcalls.len() * 6);
    let mut keys = vec![];
    let alpha = 1.0; // We're not using alpha at all for now, but it's passed everywhere.
    let mut previous_frame_time = PreciseTime::now();
    let mut fps_clock = Duration::milliseconds(0);
    let mut frame_counter = 0;
    let mut fps = 1;

    loop {
        let now = PreciseTime::now();
        let dt = previous_frame_time.to(now);
        previous_frame_time = now;

        // Calculate FPS
        fps_clock = fps_clock + dt;
        frame_counter += 1;
        if fps_clock.num_milliseconds() > 1000 {
            fps = frame_counter;
            frame_counter = 1;
            fps_clock = Duration::milliseconds(0);
        }

        drawcalls.clear();
        drawcalls.push(Draw::Rectangle(Point { x: 0, y: 0 }, display_size, default_background));
        match update(state,
                     dt,
                     display_size,
                     fps,
                     &keys,
                     settings,
                     &mut drawcalls) {
            Some((new_settings, new_state)) => {
                state = new_state;
                settings = new_settings;
            }
            None => break,
        };
        keys.clear();

        // Process drawcalls
        vertices.clear();
        // NOTE: The first item is inserted by the engine, so keep it here
        drawcalls[1..].sort_by(|a, b| {
            use std::cmp::Ordering::*;
            use engine::Draw::*;

            match (a, b) {
                (&Char(p1, ..), &Char(p2, ..)) => {
                    let x_ordering = p1.x.cmp(&p2.x);
                    if x_ordering == Equal {
                        p1.y.cmp(&p2.y)
                    } else {
                        x_ordering
                    }
                }

                (&Background(..), &Background(..)) => Equal,
                (&Text(..), &Text(..)) => Equal,
                (&Rectangle(..), &Rectangle(..)) => Equal,
                (&Fade(..), &Fade(..)) => Equal,

                (&Fade(..), _) => Greater,
                (_, &Fade(..)) => Less,

                (&Background(..), &Char(..)) => Less,
                (&Char(..), &Background(..)) => Greater,

                (&Background(..), &Text(..)) => Less,
                (&Text(..), &Background(..)) => Greater,

                (&Background(..), &Rectangle(..)) => Less,
                (&Rectangle(..), &Background(..)) => Greater,

                _ => Equal,
            }
        });

        // Remove duplicate background and foreground tiles. I.e. for
        // any given point, only the last specified drawcall of the
        // same kind will remain.
        drawcalls.reverse();
        drawcalls.dedup();
        drawcalls.reverse();

        for drawcall in &drawcalls {
            match drawcall {
                &Draw::Char(pos, chr, foreground_color) => {
                    let (pos_x, pos_y) = (pos.x as f32, pos.y as f32);
                    let (tilemap_x, tilemap_y) = texture_coords_from_char(chr);
                    let color = gl_color(foreground_color, alpha);

                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y],
                                      tilemap_index: [tilemap_x, tilemap_y],
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x + 1.0, pos_y],
                                      tilemap_index: [tilemap_x + 1.0, tilemap_y],
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y + 1.0],
                                      tilemap_index: [tilemap_x, tilemap_y + 1.0],
                                      color: color,
                                  });

                    vertices.push(Vertex {
                                      tile_position: [pos_x + 1.0, pos_y],
                                      tilemap_index: [tilemap_x + 1.0, tilemap_y],
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y + 1.0],
                                      tilemap_index: [tilemap_x, tilemap_y + 1.0],
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x + 1.0, pos_y + 1.0],
                                      tilemap_index: [tilemap_x + 1.0, tilemap_y + 1.0],
                                      color: color,
                                  });

                }

                &Draw::Background(pos, background_color) => {
                    let (pos_x, pos_y) = (pos.x as f32, pos.y as f32);
                    let tilemap_index = [0.0, 5.0];
                    let color = gl_color(background_color, alpha);

                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x + 1.0, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y + 1.0],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });

                    vertices.push(Vertex {
                                      tile_position: [pos_x + 1.0, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y + 1.0],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x + 1.0, pos_y + 1.0],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });

                }

                &Draw::Text(start_pos, ref text, color) => {
                    for (i, chr) in text.char_indices() {
                        let pos = start_pos + (i as i32, 0);
                        let (pos_x, pos_y) = (pos.x as f32, pos.y as f32);
                        let (tilemap_x, tilemap_y) = texture_coords_from_char(chr);
                        let color = gl_color(color, alpha);

                        vertices.push(Vertex {
                                          tile_position: [pos_x, pos_y],
                                          tilemap_index: [tilemap_x, tilemap_y],
                                          color: color,
                                      });
                        vertices.push(Vertex {
                                          tile_position: [pos_x + 1.0, pos_y],
                                          tilemap_index: [tilemap_x + 1.0, tilemap_y],
                                          color: color,
                                      });
                        vertices.push(Vertex {
                                          tile_position: [pos_x, pos_y + 1.0],
                                          tilemap_index: [tilemap_x, tilemap_y + 1.0],
                                          color: color,
                                      });

                        vertices.push(Vertex {
                                          tile_position: [pos_x + 1.0, pos_y],
                                          tilemap_index: [tilemap_x + 1.0, tilemap_y],
                                          color: color,
                                      });
                        vertices.push(Vertex {
                                          tile_position: [pos_x, pos_y + 1.0],
                                          tilemap_index: [tilemap_x, tilemap_y + 1.0],
                                          color: color,
                                      });
                        vertices.push(Vertex {
                                          tile_position: [pos_x + 1.0, pos_y + 1.0],
                                          tilemap_index: [tilemap_x + 1.0, tilemap_y + 1.0],
                                          color: color,
                                      });
                    }
                }

                &Draw::Rectangle(top_left, dimensions, color) => {
                    let (pos_x, pos_y) = (top_left.x as f32, top_left.y as f32);
                    let (dim_x, dim_y) = (dimensions.x as f32, dimensions.y as f32);
                    let tilemap_index = [0.0, 5.0];
                    let color = gl_color(color, alpha);

                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x + dim_x, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y + dim_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });

                    vertices.push(Vertex {
                                      tile_position: [pos_x + dim_x, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y + dim_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x + dim_x, pos_y + dim_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                }

                &Draw::Fade(fade, color) => {
                    assert!(fade >= 0.0);
                    assert!(fade <= 1.0);

                    let (pos_x, pos_y) = (0.0, 0.0);
                    let (dim_x, dim_y) = (display_size.x as f32, display_size.y as f32);
                    let tilemap_index = [0.0, 5.0];
                    let color = gl_color(color, 1.0 - fade);

                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x + dim_x, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y + dim_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });

                    vertices.push(Vertex {
                                      tile_position: [pos_x + dim_x, pos_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x, pos_y + dim_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });
                    vertices.push(Vertex {
                                      tile_position: [pos_x + dim_x, pos_y + dim_y],
                                      tilemap_index: tilemap_index,
                                      color: color,
                                  });

                }
            }
        }

        let vertex_buffer = glium::VertexBuffer::new(&display, &vertices).unwrap();


        // Render
        let mut target = display.draw();
        target.clear_color_srgb(1.0, 0.0, 1.0, 1.0);
        target
            .draw(&vertex_buffer,
                  &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                  &program,
                  &uniforms,
                  &DrawParameters {
                       blend: glium::Blend::alpha_blending(),
                       ..Default::default()
                   })
            .unwrap();
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

                Event::Focused(false) |
                Event::Suspended(false) => {
                    lctrl_pressed = false;
                    rctrl_pressed = false;
                    lalt_pressed = false;
                    ralt_pressed = false;
                    lshift_pressed = false;
                    rshift_pressed = false;
                }

                _ => (),
            }
        }

    }
}
