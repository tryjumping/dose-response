use self::vertex::Vertex;

use color::Color;
use engine::{self, Drawcall, Mouse, Settings, TextMetrics, TextOptions, UpdateFn};
use game::RunningState;
use state::State;

use glium::{self, Surface};
use glium::draw_parameters::DrawParameters;
use glium::glutin::{Event, EventsLoop, MonitorId, WindowBuilder, WindowEvent};
use glium::glutin::VirtualKeyCode as BackendKey;
use image;
use keys::{Key, KeyCode};
use point::Point;
use rect::Rectangle;
use std::time::{Duration, Instant};
use util;

const DRAWCALL_CAPACITY: usize = 10_000;
const VERTICES_CAPACITY: usize = 50_000;

fn gl_color(color: Color, alpha: f32) -> [f32; 4] {
    [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        alpha,
    ]
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

fn get_current_monitor(monitors: &[MonitorId], window_pos: Point) -> Option<MonitorId> {
    for monitor in monitors {
        let monitor_pos = {
            let pos = monitor.get_position();
            Point::new(pos.0 as i32, pos.1 as i32)
        };
        let monitor_dimensions = {
            let dim = monitor.get_dimensions();
            Point::new(dim.0 as i32, dim.1 as i32)
        };

        let monitor_bottom_left = monitor_pos + monitor_dimensions;
        if window_pos >= monitor_pos && window_pos < monitor_bottom_left {
            return Some(monitor.clone());
        }
    }

    monitors.iter().cloned().next()
}

struct Metrics {
    tile_width_px: i32,
}

impl TextMetrics for Metrics {

    fn get_text_height(&self, text: &str, options: TextOptions) -> i32 {
        if options.wrap && options.width > 0 {
            // TODO: this does a needless allocation by
            // returning Vec<String> we don't use here.
            let lines = engine::wrap_text(&text, options.width, self.tile_width_px);
            lines.len() as i32
        } else {
            1
        }
    }

    fn get_text_width(&self, text: &str, options: TextOptions) -> i32 {
        let pixel_width = if options.wrap && options.width > 0 {
            // // TODO: handle text alignment for wrapped text
            let lines = engine::wrap_text(text, options.width, self.tile_width_px);
            lines
                .iter()
                .map(|line| engine::text_width_px(line, self.tile_width_px))
                .max()
                .unwrap_or(0)
        } else {
            engine::text_width_px(text, self.tile_width_px)
        };
        let tile_width = (pixel_width as f32 / self.tile_width_px as f32).ceil();
        tile_width as i32
    }

}

#[allow(unsafe_code)]
mod vertex {
    #[derive(Copy, Clone, Debug)]
    pub struct Vertex {
        /// Position in the tile coordinates.
        ///
        /// Note that this doesn't have to be an integer, so you can
        /// implement smooth positioning by using a fractional value.
        pub pos_px: [f32; 2],

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
    implement_vertex!(Vertex, pos_px, tilemap_index, color);
}


fn build_vertices(_drawcalls: &[Drawcall], _vertices: &mut Vec<Vertex>) {
    unimplemented!()
}


pub fn main_loop(
    display_size: Point,
    default_background: Color,
    window_title: &str,
    mut state: State,
    update: UpdateFn,
) {
    let tilesize = super::TILESIZE;
    let (desired_window_width, desired_window_height) = (
        display_size.x as u32 * tilesize as u32,
        display_size.y as u32 * tilesize as u32,
    );

    println!("Requested display in tiles: {} x {}", display_size.x, display_size.y);
    println!("Desired window size: {} x {}", desired_window_width, desired_window_height);

    let mut window_width = desired_window_width;
    let mut window_height = desired_window_height;

    // GL setup

    let mut events_loop = EventsLoop::new();

    // We'll just assume the monitors won't change throughout the game.
    let monitors: Vec<_> = events_loop.get_available_monitors().collect();

    let window = WindowBuilder::new()
        .with_title(window_title)
        .with_dimensions(desired_window_width, desired_window_height);

    let context = glium::glutin::ContextBuilder::new().with_vsync(true);

    let display = glium::Display::new(window, context, &events_loop)
        .expect("dose response ERROR: Could not create the display.");

    let program = program!(&display,
                           150 => {
                               outputs_srgb: true,
                               vertex: include_str!("../shader_150.glslv"),
                               fragment: include_str!("../shader_150.glslf")
                           }
        ).unwrap();

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

    // Main loop
    let mut window_pos = {
        match display.gl_window().get_position() {
            Some((x, y)) => Point::new(x as i32, y as i32),
            None => Default::default(),
        }
    };
    println!("Window pos: {:?}", window_pos);
    let mut pre_fullscreen_window_pos = window_pos;

    let mut current_monitor = get_current_monitor(&monitors, window_pos);
    println!("All monitors:");
    for monitor in &monitors {
        println!("* {:?}, pos: {:?}, size: {:?}",
                 monitor.get_name(), monitor.get_position(), monitor.get_dimensions());
    }
    println!("Current monitor: {:?}, pos: {:?}, size: {:?}",
             current_monitor.as_ref().map(|m| m.get_name()),
             current_monitor.as_ref().map(|m| m.get_position()),
             current_monitor.as_ref().map(|m| m.get_dimensions()));

    let mut mouse = Mouse::new();
    let mut settings = Settings { fullscreen: false };
    let mut engine_display = engine::Display::new(
        display_size, Point::from_i32(display_size.y / 2), tilesize as i32);
    let mut lctrl_pressed = false;
    let mut rctrl_pressed = false;
    let mut lalt_pressed = false;
    let mut ralt_pressed = false;
    let mut lshift_pressed = false;
    let mut rshift_pressed = false;
    let mut drawcalls = Vec::with_capacity(DRAWCALL_CAPACITY);
    let mut vertices = Vec::with_capacity(VERTICES_CAPACITY);
    let mut keys = vec![];
    let mut previous_frame_time = Instant::now();
    let mut fps_clock = Duration::from_millis(0);
    let mut frame_counter = 0;
    let mut fps = 1;
    let mut running = true;
    // NOTE: This will wrap after running continuously for over 64
    // years at 60 FPS. 32 bits are just fine.
    let mut current_frame: i32 = 0;

    while running {
        let now = Instant::now();
        let dt = now.duration_since(previous_frame_time);
        previous_frame_time = now;

        // Calculate FPS
        fps_clock = fps_clock + dt;
        frame_counter += 1;
        current_frame += 1;
        if util::num_milliseconds(fps_clock) > 1000 {
            fps = frame_counter;
            frame_counter = 1;
            fps_clock = Duration::from_millis(0);
        }

        let previous_settings = settings;

        // NOTE: Skip the first frame -- the window isn't set up
        // properly there.
        if current_frame > 1 {
            engine_display.draw_rectangle(
                Rectangle::from_point_and_size(Point::new(0, 0), display_size),
                default_background);
            let update_result = update(
                &mut state,
                dt,
                display_size,
                fps,
                &keys,
                mouse,
                &mut settings,
                &Metrics {
                    tile_width_px: tilesize as i32,
                },
                &mut engine_display,
            );

            match update_result {
                RunningState::Running => {}
                RunningState::NewGame(new_state) => {
                    state = new_state;
                }
                RunningState::Stopped => break,
            }
        }

        mouse.left = false;
        mouse.right = false;

        keys.clear();

        let mut switched_from_fullscreen = false;

        if cfg!(feature = "fullscreen") {
            if previous_settings.fullscreen != settings.fullscreen {
                if settings.fullscreen {
                    println!("Switching to fullscreen.");
                    if let Some(ref monitor) = current_monitor {
                        pre_fullscreen_window_pos = window_pos;
                        println!(
                            "Monitor: {:?}, pos: {:?}, dimensions: {:?}",
                            monitor.get_name(),
                            monitor.get_position(),
                            monitor.get_dimensions()
                        );
                        display.gl_window().set_fullscreen(Some(monitor.clone()));
                    } else {
                        println!("`current_monitor` is not set!??");
                    }
                } else {
                    println!("Switched from fullscreen.");
                    display.gl_window().set_fullscreen(None);
                    let pos = display.gl_window().get_position();
                    println!("New window position: {:?}", pos);
                    switched_from_fullscreen = true;
                }
            }
        }

        // Return a pixel position from the given tile position
        let pixel_from_tile = |tile_pos: Point| -> Point {
            tile_pos * (tilesize as i32)
        };

        // Process drawcalls
        drawcalls.clear();
        engine_display.push_drawcalls(
            Point::new(desired_window_width as i32, desired_window_height as i32),
            &mut drawcalls);

        vertices.clear();
        build_vertices(&drawcalls, &mut vertices);

        // NOTE: So the rendering is a little more involved than I
        // initially planned.
        //
        // Here's the problem, we want to be able to set the
        // background colour independently and possibly *after*
        // setting the glyph.
        //
        // It might also be interspersed by the `Rectangle` drawcalls
        // which will clear the given area.
        //
        // On top of that, we only want to render the topmost glyph on
        // any given tile.
        //
        // I could not figure out how to do this by merely sorting the
        // drawcalls with a custom ordering closure. It may yet be
        // possible but I don't know.
        //
        // So what we do instead is a three-phase render:
        //
        // First, we'll build a map of the background tiles. We do
        // this by going through all drawcalls and recording the
        // colour for each `Background` or `Rectangle` call.
        //
        // Second, we render all the backgrounds by iterating over the
        // entires in the background map. This makes sure that we've
        // rendered all background tiles even if there was no
        // corresponding glyph there.
        //
        // Third, we render everything *except* for the `Background`
        // tiles. They've been rendered already so let's not do it
        // again. This handles the situation where a later background
        // would overwrite a glyph.
        //
        // Also, when we render `Glyph`s, we will first clear the tile
        // to the background in the map. This will make sure that any
        // previously rendered glyphs will be overwritten.
        //
        // Furthermore, we still DO render the `Rectangle`s, because
        // they should still clear all the areas they cover.
        // Basically, we pre-render the background and then have
        // OpenGL handle the rest.
        //
        // Finally, we do NOT render the `Fade` during the rendering
        // pass, but record the fade colour and value and render it
        // after all the other drawcalls have been processed. That
        // way, it will not be overwritten by any subsequent
        // drawcalls.

        // NOTE: last time we tried it it was 3632
        //println!("Drawcall count: {}", drawcalls.len());

        // NOTE: this has been deleted
        //engine::populate_background_map(&mut background_map, &drawcalls);

        // Render the background tiles separately and before all the other drawcalls.
        // NOTE: use `cells` here
        // for (pos, background_color) in background_map.points() {
        //     let pos_x = pos.x as f32;
        //     let pos_y = pos.y as f32;
        //     let tile_width = tilesize as f32;
        //     let tile_height = tilesize as f32;
        //     let tilemap_index = [0.0, 5.0];
        //     let color = gl_color(*background_color, alpha);

        //     vertices.push(Vertex {
        //         pos_px: [pos_x, pos_y],
        //         tilemap_index: tilemap_index,
        //         color: color,
        //     });
        //     vertices.push(Vertex {
        //         pos_px: [pos_x + tile_width, pos_y],
        //         tilemap_index: tilemap_index,
        //         color: color,
        //     });
        //     vertices.push(Vertex {
        //         pos_px: [pos_x, pos_y + tile_height],
        //         tilemap_index: tilemap_index,
        //         color: color,
        //     });

        //     vertices.push(Vertex {
        //         pos_px: [pos_x + tile_width, pos_y],
        //         tilemap_index: tilemap_index,
        //         color: color,
        //     });
        //     vertices.push(Vertex {
        //         pos_px: [pos_x, pos_y + tile_height],
        //         tilemap_index: tilemap_index,
        //         color: color,
        //     });
        //     vertices.push(Vertex {
        //         pos_px: [pos_x + tile_width, pos_y + tile_height],
        //         tilemap_index: tilemap_index,
        //         color: color,
        //     });
        // }

        let screen_fade = None;

        // for drawcall in &drawcalls {
        //     match drawcall {
                // &Draw::Char(pos, chr, foreground_color, offset_px) => {
                //     if pos.x >= 0 && pos.y >= 0 && pos.x < display_size.x && pos.y < display_size.y
                //     {
                //         let pixel_pos = pixel_from_tile(pos) + offset_px;
                //         let (pos_x, pos_y) = (pixel_pos.x as f32, pixel_pos.y as f32);
                //         let tile_width = tilesize as f32;
                //         let tile_height = tilesize as f32;
                //         let fill_tile_tilemap_index = [0.0, 5.0];
                //         let (tilemap_x, tilemap_y) = texture_coords_from_char(chr);
                //         let color = gl_color(foreground_color, alpha);
                //         let background_color = gl_color(
                //             background_map.get(pos),
                //             alpha,
                //         );

                //         // NOTE: fill the tile with the background colour
                //         vertices.push(Vertex {
                //             pos_px: [pos_x, pos_y],
                //             tilemap_index: fill_tile_tilemap_index,
                //             color: background_color,
                //         });
                //         vertices.push(Vertex {
                //             pos_px: [pos_x + tile_width, pos_y],
                //             tilemap_index: fill_tile_tilemap_index,
                //             color: background_color,
                //         });
                //         vertices.push(Vertex {
                //             pos_px: [pos_x, pos_y + tile_height],
                //             tilemap_index: fill_tile_tilemap_index,
                //             color: background_color,
                //         });

                //         vertices.push(Vertex {
                //             pos_px: [pos_x + tile_width, pos_y],
                //             tilemap_index: fill_tile_tilemap_index,
                //             color: background_color,
                //         });
                //         vertices.push(Vertex {
                //             pos_px: [pos_x, pos_y + tile_height],
                //             tilemap_index: fill_tile_tilemap_index,
                //             color: background_color,
                //         });
                //         vertices.push(Vertex {
                //             pos_px: [pos_x + tile_width, pos_y + tile_height],
                //             tilemap_index: fill_tile_tilemap_index,
                //             color: background_color,
                //         });

                //         // NOTE: Center the glyphs in their cells
                //         let glyph_width = engine::glyph_advance_width(chr).unwrap_or(tilesize as i32);
                //         let x_offset = (tilesize as i32 - glyph_width) / 2;
                //         let pos_x = pos_x + x_offset as f32;

                //         // NOTE: draw the glyph
                //         vertices.push(Vertex {
                //             pos_px: [pos_x, pos_y],
                //             tilemap_index: [tilemap_x, tilemap_y],
                //             color: color,
                //         });
                //         vertices.push(Vertex {
                //             pos_px: [pos_x + tile_width, pos_y],
                //             tilemap_index: [tilemap_x + 1.0, tilemap_y],
                //             color: color,
                //         });
                //         vertices.push(Vertex {
                //             pos_px: [pos_x, pos_y + tile_height],
                //             tilemap_index: [tilemap_x, tilemap_y + 1.0],
                //             color: color,
                //         });

                //         vertices.push(Vertex {
                //             pos_px: [pos_x + tile_width, pos_y],
                //             tilemap_index: [tilemap_x + 1.0, tilemap_y],
                //             color: color,
                //         });
                //         vertices.push(Vertex {
                //             pos_px: [pos_x, pos_y + tile_height],
                //             tilemap_index: [tilemap_x, tilemap_y + 1.0],
                //             color: color,
                //         });
                //         vertices.push(Vertex {
                //             pos_px: [pos_x + tile_width, pos_y + tile_height],
                //             tilemap_index: [tilemap_x + 1.0, tilemap_y + 1.0],
                //             color: color,
                //         });
                //     }
                // }

                // &Draw::Text(start_pos, ref text, color, options) => {
                //     let color = gl_color(color, alpha);
                //     let tile_width = tilesize as f32;
                //     let tile_height = tilesize as f32;

                //     let mut render_line = |pos_px: Point, line: &str| {
                //         let pos_x = pos_px.x as f32;
                //         let pos_y = pos_px.y as f32;

                //         let mut offset_x = 0.0;
                //         //let mut offset_y = 0.0;

                //         // TODO: we need to split this by words or it
                //         // won't do word breaks, split at punctuation,
                //         // etc.

                //         // TODO: also, we're no longer calculating the
                //         // line height correctly. Needs to be set on the
                //         // actual result here.
                //         for chr in line.chars() {
                //             let (tilemap_x, tilemap_y) = texture_coords_from_char(chr);
                //             // if options.wrap && options.width > 0 {
                //             //     if offset_x >= (options.width as f32 * tile_width) {
                //             //         offset_y += tile_height;
                //             //         offset_x = 0.0;
                //             //     }
                //             // }
                //             let pos_x = pos_x + offset_x;
                //             //let pos_y = pos_y + offset_y;

                //             vertices.push(Vertex {
                //                 pos_px: [pos_x, pos_y],
                //                 tilemap_index: [tilemap_x, tilemap_y],
                //                 color: color,
                //             });
                //             vertices.push(Vertex {
                //                 pos_px: [pos_x + tile_width, pos_y],
                //                 tilemap_index: [tilemap_x + 1.0, tilemap_y],
                //                 color: color,
                //             });
                //             vertices.push(Vertex {
                //                 pos_px: [pos_x, pos_y + tile_height],
                //                 tilemap_index: [tilemap_x, tilemap_y + 1.0],
                //                 color: color,
                //             });

                //             vertices.push(Vertex {
                //                 pos_px: [pos_x + tile_width, pos_y],
                //                 tilemap_index: [tilemap_x + 1.0, tilemap_y],
                //                 color: color,
                //             });
                //             vertices.push(Vertex {
                //                 pos_px: [pos_x, pos_y + tile_height],
                //                 tilemap_index: [tilemap_x, tilemap_y + 1.0],
                //                 color: color,
                //             });
                //             vertices.push(Vertex {
                //                 pos_px: [pos_x + tile_width, pos_y + tile_height],
                //                 tilemap_index: [tilemap_x + 1.0, tilemap_y + 1.0],
                //                 color: color,
                //             });

                //             let advance_width =
                //                 engine::glyph_advance_width(chr).unwrap_or(tilesize as i32);
                //             offset_x += advance_width as f32;
                //         }
                //     };

                //     if options.wrap && options.width > 0 {
                //         // TODO: handle text alignment for wrapped text
                //         let lines = engine::wrap_text(text, options.width, tile_width as i32);
                //         for (index, line) in lines.iter().enumerate() {
                //             let pos = pixel_from_tile(start_pos + Point::new(0, index as i32));
                //             render_line(pos, line);
                //         }
                //     } else {
                //         use engine::TextAlign::*;
                //         let pos = match options.align {
                //             Left => pixel_from_tile(start_pos),
                //             Right => {
                //                 pixel_from_tile(start_pos + (1, 0))
                //                     - Point::new(engine::text_width_px(text, tile_width as i32), 0)
                //             }
                //             Center => {
                //                 let tile_width = tile_width as i32;
                //                 let text_width = engine::text_width_px(text, tile_width);
                //                 let max_width = options.width * (tile_width);
                //                 if max_width < 1 || (text_width > max_width) {
                //                     start_pos
                //                 } else {
                //                     pixel_from_tile(start_pos)
                //                         + Point::new((max_width - text_width) / 2, 0)
                //                 }
                //             }
                //         };
                //         render_line(pos, text);
                //     }
                // }

                // &Draw::Rectangle(rect, color) => {
                //     let top_left = rect.top_left();
                //     let dimensions = rect.size();
                //     let top_left_px = pixel_from_tile(top_left);
                //     let (pos_x, pos_y) = (top_left_px.x as f32, top_left_px.y as f32);
                //     let dimensions_px = pixel_from_tile(dimensions);
                //     let (dim_x, dim_y) = (dimensions_px.x as f32, dimensions_px.y as f32);
                //     let tilemap_index = [0.0, 5.0];
                //     let color = gl_color(color, alpha);

                //     vertices.push(Vertex {
                //         pos_px: [pos_x, pos_y],
                //         tilemap_index: tilemap_index,
                //         color: color,
                //     });
                //     vertices.push(Vertex {
                //         pos_px: [pos_x + dim_x, pos_y],
                //         tilemap_index: tilemap_index,
                //         color: color,
                //     });
                //     vertices.push(Vertex {
                //         pos_px: [pos_x, pos_y + dim_y],
                //         tilemap_index: tilemap_index,
                //         color: color,
                //     });

                //     vertices.push(Vertex {
                //         pos_px: [pos_x + dim_x, pos_y],
                //         tilemap_index: tilemap_index,
                //         color: color,
                //     });
                //     vertices.push(Vertex {
                //         pos_px: [pos_x, pos_y + dim_y],
                //         tilemap_index: tilemap_index,
                //         color: color,
                //     });
                //     vertices.push(Vertex {
                //         pos_px: [pos_x + dim_x, pos_y + dim_y],
                //         tilemap_index: tilemap_index,
                //         color: color,
                //     });
                // }

                // &Draw::Fade(fade, color) => {
                //     screen_fade = Some((fade, color));
                // }

        //     }
        // }

        // NOTE: render the fade overlay
        if let Some((mut fade, color)) = screen_fade {
            if fade < 0.0 {
                fade = 0.0;
            }
            if fade > 1.0 {
                fade = 1.0;
            }
            let (pos_x, pos_y) = (0.0, 0.0);
            let display_size_px = pixel_from_tile(display_size);
            let (dim_x, dim_y) = (display_size_px.x as f32, display_size_px.y as f32);
            let tilemap_index = [0.0, 5.0];
            let color = gl_color(color, 1.0 - fade);

            vertices.push(Vertex {
                pos_px: [pos_x, pos_y],
                tilemap_index: tilemap_index,
                color: color,
            });
            vertices.push(Vertex {
                pos_px: [pos_x + dim_x, pos_y],
                tilemap_index: tilemap_index,
                color: color,
            });
            vertices.push(Vertex {
                pos_px: [pos_x, pos_y + dim_y],
                tilemap_index: tilemap_index,
                color: color,
            });

            vertices.push(Vertex {
                pos_px: [pos_x + dim_x, pos_y],
                tilemap_index: tilemap_index,
                color: color,
            });
            vertices.push(Vertex {
                pos_px: [pos_x, pos_y + dim_y],
                tilemap_index: tilemap_index,
                color: color,
            });
            vertices.push(Vertex {
                pos_px: [pos_x + dim_x, pos_y + dim_y],
                tilemap_index: tilemap_index,
                color: color,
            });
        }

        if vertices.len() > VERTICES_CAPACITY {
            println!(
                "Warning: vertex count exceeded initial capacity {}. Current count: {} ",
                vertices.len(),
                VERTICES_CAPACITY
            );
        }

        let vertex_buffer = glium::VertexBuffer::new(&display, &vertices).unwrap();

        // Calculate the dimensions to provide the largest display
        // area while maintaining the aspect ratio (and letterbox the
        // display).
        let (native_display_px, display_px, extra_px) = {
            let window_width = window_width as f32;
            let window_height = window_height as f32;
            let tilecount_x = display_size.x as f32;
            let tilecount_y = display_size.y as f32;

            let unscaled_game_width = tilecount_x * tilesize as f32;
            let unscaled_game_height = tilecount_y * tilesize as f32;

            // println!("window w x h: {:?}", (window_width, window_height));
            // println!("unscaled game {:?}", (unscaled_game_width, unscaled_game_height));

            // TODO: we're assuming that the unscaled dimensions
            // already fit into the display. So the game is only going
            // to be scaled up, not down.



            // NOTE: try if the hight should fill the display area
            let scaled_tilesize = (window_height / tilecount_y).floor();
            let scaled_width = scaled_tilesize * tilecount_x;
            let scaled_height = scaled_tilesize * tilecount_y;
            let (final_scaled_width, final_scaled_height) = if scaled_width <= window_width {
                (scaled_width, scaled_height)
            } else {
                // NOTE: try if the width should fill the display area
                let scaled_tilesize = (window_width / tilecount_x).floor();
            let scaled_width = scaled_tilesize * tilecount_x;
            let scaled_height = scaled_tilesize * tilecount_y;

                if scaled_height <= window_height {
                    // NOTE: we're good
                } else {
                    println!("Can't scale neither to width nor height wtf.");
                }
                (scaled_width, scaled_height)
            };
            //println!("Final scaled: {} x {}", final_scaled_width, final_scaled_height);

            let native_display_px = [unscaled_game_width, unscaled_game_height];
            let display_px = [final_scaled_width, final_scaled_height];
            let extra_px = [window_width - final_scaled_width, window_height - final_scaled_height];
            //println!("{:?}", (native_display_px, display_px, extra_px));
            (native_display_px, display_px, extra_px)
        };

        // TODO: Once we support multiple font sizes, we can adjust it
        // here. We could also potentially only allow resizes in steps
        // that would result in crisp text (i.e. no font resizing on
        // the GL side).

        let uniforms = uniform! {
            tex: &texture,
            tile_count: [display_size.x as f32, display_size.y as f32],
            // TODO: pass this from the block above
            native_display_px: native_display_px,
            display_px: display_px,
            extra_px: extra_px,
            texture_gl_dimensions: [1.0 / texture_tile_count_x,
                                    1.0 / texture_tile_count_y],
        };

        // Render
        let mut target = display.draw();
        target.clear_color_srgb(0.1, 0.0, 0.1, 1.0);
        target
            .draw(
                &vertex_buffer,
                &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &program,
                &uniforms,
                &DrawParameters {
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
        target.finish().unwrap();

        // Process events
        events_loop.poll_events(|ev| {
            //println!("{:?}", ev);
            match ev {
                Event::WindowEvent {
                    window_id: _,
                    event,
                } => {
                    match event {
                        WindowEvent::Closed => running = false,
                        WindowEvent::Resized(width, height) => {
                            println!("[FRAME {}] Window resized to: {} x {}",
                                     current_frame, width, height);
                            window_width = width;
                            window_height = height;
                        }
                        WindowEvent::Moved(x, y) => {
                            if settings.fullscreen || switched_from_fullscreen {
                                // Don't update the window position
                                //
                                // Even after we switch from
                                // fullscreen, the `Moved` event has a
                                // wrong value that messes things up.
                                // So we restore the previous position
                                // manually instead.
                            } else {
                                println!("[FRAME {}] Window moved to: {}, {}",
                                         current_frame, x, y);
                                window_pos.x = x;
                                window_pos.y = y;
                                current_monitor = get_current_monitor(&monitors, window_pos);
                                println!("Current monitor: {:?}, pos: {:?}, size: {:?}",
                                         current_monitor.as_ref().map(|m| m.get_name()),
                                         current_monitor.as_ref().map(|m| m.get_position()),
                                         current_monitor.as_ref().map(|m| m.get_dimensions()));
                            }
                        }
                        WindowEvent::ReceivedCharacter(chr) => {
                            let code = match chr {
                                '?' => Some(KeyCode::QuestionMark),
                                _ => None,
                            };
                            if let Some(code) = code {
                                keys.push(Key {
                                    code: code,
                                    alt: false,
                                    ctrl: false,
                                    shift: false,
                                });
                            }
                        }
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            input,
                        } => {
                            use glium::glutin::ElementState::*;
                            let pressed = match input.state {
                                Pressed => true,
                                Released => false,
                            };

                            // println!("KeyboardInput event!");
                            // println!("{:?}", input);

                            // TODO: this is a temp fix for a
                            // glutin/winit bug where the keypress
                            // release event for the Shift keys has
                            // its `virtual_keycode` set to `None`
                            // instead of `Some(LShift)`. But the
                            // scancodes still work so we'll use them
                            // instead for now.
                            // It's a winit issue:
                            // https://github.com/tomaka/winit/issues/361
                            if input.scancode == 42 && !pressed {
                                lshift_pressed = false;
                            }
                            if input.scancode == 54 && !pressed {
                                rshift_pressed = false;
                            }

                            match input.virtual_keycode {
                                Some(BackendKey::LControl) => {
                                    lctrl_pressed = pressed;
                                }
                                Some(BackendKey::RControl) => {
                                    rctrl_pressed = pressed;
                                }
                                Some(BackendKey::LAlt) => {
                                    lalt_pressed = pressed;
                                }
                                Some(BackendKey::RAlt) => {
                                    ralt_pressed = pressed;
                                }
                                Some(BackendKey::LShift) => {
                                    lshift_pressed = pressed;
                                }
                                Some(BackendKey::RShift) => {
                                    rshift_pressed = pressed;
                                }
                                Some(key_code) => {
                                    if pressed {
                                        if let Some(code) = key_code_from_backend(key_code) {
                                            let key = Key {
                                                code: code,
                                                alt: lalt_pressed || ralt_pressed || input.modifiers.alt,
                                                ctrl: lctrl_pressed || rctrl_pressed || input.modifiers.ctrl,
                                                shift: lshift_pressed || rshift_pressed || input.modifiers.shift,
                                            };
                                            // println!("Pushing {:?}", key);
                                            keys.push(key);
                                        }
                                    }
                                }
                                None => {
                                    let code = match input.scancode {
                                        79 => Some(KeyCode::NumPad7),
                                        80 => Some(KeyCode::NumPad8),
                                        81 => Some(KeyCode::NumPad9),
                                        83 => Some(KeyCode::NumPad4),
                                        84 => Some(KeyCode::NumPad5),
                                        85 => Some(KeyCode::NumPad6),
                                        87 => Some(KeyCode::NumPad1),
                                        88 => Some(KeyCode::NumPad2),
                                        89 => Some(KeyCode::NumPad3),
                                        _ => None,
                                    };
                                    if pressed {
                                        if let Some(code) = code {
                                            let key = Key {
                                                code: code,
                                                alt: lalt_pressed || ralt_pressed || input.modifiers.alt,
                                                ctrl: lctrl_pressed || rctrl_pressed || input.modifiers.ctrl,
                                                shift: lshift_pressed || rshift_pressed || input.modifiers.shift,
                                            };
                                            // println!("Pushing {:?}", key);
                                            keys.push(key);
                                        }
                                    }
                                }
                            }
                        }
                        WindowEvent::CursorMoved {
                            position: (x, y), ..
                        } => {
                            // println!("Extra px: {:?}", extra_px);
                            // println!("Display px: {:?}", display_px);
                            // println!("Native display px: {:?}", native_display_px);
                            // println!("screen width/height: {:?}", (screen_width, screen_height));
                            let (x, y) = (x as i32, y as i32);

                            let (x, y) = (x - (extra_px[0] / 2.0) as i32, y - (extra_px[1] / 2.0) as i32);
                            let x = util::clamp(0, x, display_px[0] as i32 - 1);
                            let y = util::clamp(0, y, display_px[1] as i32 - 1);

                            mouse.screen_pos = Point { x, y };

                            let tile_width = display_px[0] as i32 / display_size.x;
                            let mouse_tile_x = x / tile_width;

                            let tile_height = display_px[1] as i32 / display_size.y;
                            let mouse_tile_y = y / tile_height;

                            mouse.tile_pos = Point {
                                x: mouse_tile_x,
                                y: mouse_tile_y,
                            };
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            use glium::glutin::MouseButton::*;
                            use glium::glutin::ElementState::*;

                            match (state, button) {
                                (Released, Left) => {
                                    mouse.left = true;
                                }
                                (Released, Right) => {
                                    mouse.right = true;
                                }
                                _ => {}
                            }
                        }
                        WindowEvent::Focused(false) => {
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
                _ => (),
            }
        });

        if current_frame == 1 {
            // NOTE: We should have the proper window position and
            // monitor info at this point but not sooner.

            // NOTE: If the primary monitor is different from the
            // monitor the window actually spawns at (this happens on
            // my dev machine where the primary monitor is in the
            // portrait orientation and therefore more narrow, but the
            // game window normally spawns on my landscape monitor),
            // it gets resized. We can detect it because this event
            // fires on the first frame. So we ask it to resize to the
            // expected size again and leave it at that.
            println!("Current monitor: {:?}", current_monitor.as_ref().map(|m| m.get_dimensions()));

            if desired_window_width != window_width || desired_window_height != window_height {
                if let Some(ref monitor) = current_monitor {
                    let (monitor_width, monitor_height) = monitor.get_dimensions();
                    if desired_window_width <= monitor_width &&
                        desired_window_height <= monitor_height
                    {
                        println!("Resetting the window to its expected size: {} x {}.",
                                 desired_window_width, desired_window_height);
                        display.gl_window().set_inner_size(
                            desired_window_width, desired_window_height);
                    } else {
                        println!("TODO: try to resize but maintain aspect ratio.");
                    }
                }
            }

        }


        // If we just switched from fullscreen back to a windowed
        // mode, restore the window position we had before. We do this
        // because the `Moved` event fires with an incorrect value
        // when coming back from full screen.
        //
        // This ensures that we can switch full screen back and fort
        // on a multi monitor setup.
        if switched_from_fullscreen {
            window_pos = pre_fullscreen_window_pos;
        }
    }
}
