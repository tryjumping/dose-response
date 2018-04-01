use color::Color;
use engine::{self, Draw, Mouse, Settings, TextMetrics, UpdateFn};
use game::RunningState;
use keys::KeyCode;
use point::Point;
use state::State;
use util;

use std::time::{Duration, Instant};

use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::{self, Keycode as BackendKey};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use image;


//const DESIRED_FPS: u64 = 60;


pub struct Metrics {
    tile_width_px: i32,
}

impl TextMetrics for Metrics {
    fn get_text_height(&self, text_drawcall: &Draw) -> i32 {
        match text_drawcall {
            &Draw::Text(_pos, ref text, _color, options) => {
                if options.wrap && options.width > 0 {
                    // TODO: this does a needless allocation by
                    // returning Vec<String> we don't use here.
                    let lines = engine::wrap_text(&text, options.width, self.tile_width_px);
                    lines.len() as i32
                } else {
                    1
                }
            }
            _ => {
                panic!("The argument to `TextMetrics::get_text_height` must be `Draw::Text`!");
            }
        }
    }

    fn get_text_width(&self, text_drawcall: &Draw) -> i32 {
        match text_drawcall {
            &Draw::Text(_, ref text, _, options) => {
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
            _ => {
                panic!("The argument to `TextMetrics::get_text_height` must be `Draw::Text`!");
            }
        }
    }
}


fn key_code_from_backend(backend_code: BackendKey) -> Option<KeyCode> {
    match backend_code {
        BackendKey::Return => Some(KeyCode::Enter),
        BackendKey::Escape => Some(KeyCode::Esc),
        BackendKey::Space => Some(KeyCode::Space),

        BackendKey::Num0 => Some(KeyCode::D0),
        BackendKey::Num1 => Some(KeyCode::D1),
        BackendKey::Num2 => Some(KeyCode::D2),
        BackendKey::Num3 => Some(KeyCode::D3),
        BackendKey::Num4 => Some(KeyCode::D4),
        BackendKey::Num5 => Some(KeyCode::D5),
        BackendKey::Num6 => Some(KeyCode::D6),
        BackendKey::Num7 => Some(KeyCode::D7),
        BackendKey::Num8 => Some(KeyCode::D8),
        BackendKey::Num9 => Some(KeyCode::D9),

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

        BackendKey::Kp1 => Some(KeyCode::NumPad1),
        BackendKey::Kp2 => Some(KeyCode::NumPad2),
        BackendKey::Kp3 => Some(KeyCode::NumPad3),
        BackendKey::Kp4 => Some(KeyCode::NumPad4),
        BackendKey::Kp5 => Some(KeyCode::NumPad5),
        BackendKey::Kp6 => Some(KeyCode::NumPad6),
        BackendKey::Kp7 => Some(KeyCode::NumPad7),
        BackendKey::Kp8 => Some(KeyCode::NumPad8),
        BackendKey::Kp9 => Some(KeyCode::NumPad9),
        BackendKey::Kp0 => Some(KeyCode::NumPad0),

        _ => None,
    }
}


fn load_texture<T>(texture_creator: &TextureCreator<T>) -> Result<Texture, String> {
    let data = &include_bytes!(concat!(env!("OUT_DIR"), "/font.png"))[..];
    let image = image::load_from_memory(data)
        .map_err(|err| format!("Error loading image: {}", err))?.to_rgba();
    let (width, height) = image.dimensions();
    // Pitch is the length of the row in bytes. We have 4 bytes (RGBA, each is a u8):
    let pitch = width * 4;
    // NOTE: I think `SDL2` and `Image` differ in endianness and
    // that's why we have to say ABGR instead of RGBA here
    let format = PixelFormatEnum::ABGR8888;

    let raw_image = &mut image.into_raw();
    let temp_surface = Surface::from_data(raw_image, width, height, pitch, format)?;

    texture_creator.create_texture_from_surface(&temp_surface)
        .map_err(|err| format!("Could not create texture from surface: {}", err))
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

    let sdl_context = sdl2::init()
        .expect("SDL context creation failed.");
    let video_subsystem = sdl_context.video()
        .expect("SDL video subsystem creation failed.");

    // NOTE: add `.fullscreen_desktop()` to start in fullscreen.
    let window = video_subsystem.window(window_title, desired_window_width, desired_window_height)
        .opengl()
        .position_centered()
        .build()
        .expect("SDL window creation failed.");

    // NOTE: use `.software()` instead of `.accelerated()` to use software rendering
    // TODO: test this on more machines but a very simple test seems to be actually faster
    // with software???
    let mut canvas = window.into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .expect("SDL canvas creation failed.");
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);


    let mut event_pump = sdl_context.event_pump()
        .expect("SDL event pump creation failed.");

    let texture_creator = canvas.texture_creator();
    let mut texture = load_texture(&texture_creator)
        .expect("Loading texture failed.");

    let mut mouse = Mouse::new();
    let mut settings = Settings { fullscreen: false };
    let mut background_map =
        vec![Color { r: 0, g: 0, b: 0 }; (display_size.x * display_size.y) as usize];
    let mut drawcalls = Vec::with_capacity(engine::DRAWCALL_CAPACITY);
    // let expected_frame_length = Duration::from_millis(1000 / DESIRED_FPS);
    let mut keys = vec![];
    // We're not using alpha at all for now, but it's passed everywhere.
    let mut previous_frame_start_time = Instant::now();
    let mut fps_clock = Duration::from_millis(0);
    let mut frames_in_current_second = 0;
    let mut fps = 0;
    // NOTE: This will wrap after running continuously for over 64
    // years at 60 FPS. 32 bits are just fine.
    let mut current_frame_id: i32 = 0;
    let mut running = true;

    while running {
        let frame_start_time = Instant::now();
        let dt = frame_start_time.duration_since(previous_frame_start_time);
        previous_frame_start_time = frame_start_time;

        // Calculate FPS
        fps_clock = fps_clock + dt;
        frames_in_current_second += 1;
        current_frame_id += 1;
        if util::num_milliseconds(fps_clock) > 1000 {
            fps = frames_in_current_second;
            frames_in_current_second = 1;
            fps_clock = Duration::new(0, 0);
        }

        for event in event_pump.poll_iter() {
            match event {

                Event::Quit {..} => {
                    running = false;
                },

                Event::KeyDown { keycode: Some(backend_code), keymod, ..} => {
                    if let Some(code) = key_code_from_backend(backend_code) {
                        let key = super::Key {
                            code: code,
                            alt: keymod.intersects(keyboard::LALTMOD | keyboard::RALTMOD),
                            ctrl: keymod.intersects(keyboard::LCTRLMOD | keyboard::RCTRLMOD),
                            shift: keymod.intersects(keyboard::LSHIFTMOD | keyboard::RSHIFTMOD),
                        };
                        keys.push(key);
                    }
                }

                Event::MouseMotion {x, y, ..} => {
                    // TODO: calculate this from the real window size
                    let display_px = [desired_window_width, desired_window_height];

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

                Event::MouseButtonDown {..} => {
                    // NOTE: do nothing. We handle everything in the mouse up event
                }

                Event::MouseButtonUp {mouse_btn, ..} => {
                    use sdl2::mouse::MouseButton::*;
                    match mouse_btn {
                        Left => {
                            mouse.left = true;
                        }
                        Right => {
                            mouse.right = true;
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        drawcalls.clear();
        let previous_settings = settings;


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
            &mut drawcalls,
        );

        match update_result {
            RunningState::Running => {}
            RunningState::NewGame(new_state) => {
                state = new_state;
            }
            RunningState::Stopped => break,
        }

        mouse.left = false;
        mouse.right = false;
        keys.clear();


        if cfg!(feature = "fullscreen") {
            use sdl2::video::FullscreenType::*;
            if previous_settings.fullscreen != settings.fullscreen {
                if settings.fullscreen {
                    println!("[{}] Switching to (desktop-type) fullscreen", current_frame_id);
                    if let Err(err) = canvas.window_mut().set_fullscreen(Desktop) {
                        println!("[{}] WARNING: Could not switch to fullscreen:", current_frame_id);
                        println!("{:?}", err);
                    }
                } else {
                    println!("[{}] Switching fullscreen off", current_frame_id);
                    if let Err(err) = canvas.window_mut().set_fullscreen(Off) {
                        println!("[{}] WARNING: Could not leave fullscreen:", current_frame_id);
                        println!("{:?}", err);
                    }
                }
            }
        }


        if drawcalls.len() > engine::DRAWCALL_CAPACITY {
            println!(
                "Warning: drawcall count exceeded initial capacity {}. Current count: {}.",
                drawcalls.len(),
                engine::DRAWCALL_CAPACITY
            );
        }

        engine::populate_background_map(&mut background_map, display_size, &drawcalls);

        // println!("Pre-draw duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        // NOTE: render
        canvas.set_draw_color(
            sdl2::pixels::Color::RGB(default_background.r,
                                     default_background.g,
                                     default_background.b));
        canvas.clear();
        // Render the background tiles separately and before all the other drawcalls.
        for (index, background_color) in background_map.iter().enumerate() {
            let pos_x = (index as i32) % display_size.x * tilesize as i32;
            let pos_y = (index as i32) / display_size.x * tilesize as i32;

            canvas.set_draw_color(
                sdl2::pixels::Color::RGB(background_color.r,
                                         background_color.g,
                                         background_color.b));
            let rect = Rect::new(pos_x, pos_y, tilesize, tilesize);
            if let Err(err) = canvas.fill_rect(rect) {
                println!("[{}] WARNING: drawing rectangle {:?} failed:",
                         current_frame_id, rect);
                println!("{}", err);
            }
        }

        let mut screen_fade = None;

        for drawcall in &drawcalls {
            match drawcall {
                &Draw::Char(pos, chr, foreground_color, offset_px) => {
                    if pos.x >= 0 && pos.y >= 0 && pos.x < display_size.x && pos.y < display_size.y
                    {
                        let (texture_index_x, texture_index_y) = super::texture_coords_from_char(chr)
                            .unwrap_or((0, 0));
                        let src = Rect::new(texture_index_x * tilesize as i32,
                                            texture_index_y * tilesize as i32,
                                            tilesize, tilesize);
                        let dst = Rect::new(pos.x * tilesize as i32 + offset_px.x,
                                            pos.y * tilesize as i32 + offset_px.y,
                                            tilesize, tilesize);

                        let background_color = background_map[(pos.y * display_size.x + pos.x) as usize];
                        canvas.set_draw_color(
                            sdl2::pixels::Color::RGB(background_color.r,
                                                     background_color.g,
                                                     background_color.b));
                        if let Err(err) = canvas.fill_rect(dst) {
                            println!("[{}] WARNING: Draw::Char drawing rectangle {:?} failed:",
                                     current_frame_id, dst);
                            println!("{}", err);
                        }

                        // NOTE: Center the glyphs in their cells
                        let glyph_width = engine::glyph_advance_width(chr).unwrap_or(tilesize as i32);
                        let x_offset = (tilesize as i32 - glyph_width) / 2;
                        let mut dst = dst;
                        dst.offset(x_offset, 0);

                        texture.set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);
                        if let Err(err) = canvas.copy(&texture, Some(src), Some(dst)) {
                            println!("[{}] WARNING: Draw::Char blitting char {:?} at pos {:?} from source {:?} to {:?} failed:",
                                     current_frame_id, chr, pos, src, dst);
                            println!("{}", err);
                        }
                    }
                }

                &Draw::Background(..) => {
                    // NOTE: do nothing, all the BG calls have been drawn already
                }

                &Draw::Rectangle(rect, color) => {
                    let top_left_px = rect.top_left() * tilesize as i32;
                    let dimensions_px = rect.size() * tilesize as i32;

                    let rect = Rect::new(top_left_px.x, top_left_px.y,
                                         dimensions_px.x as u32, dimensions_px.y as u32);
                    canvas.set_draw_color(
                        sdl2::pixels::Color::RGB(color.r,
                                                 color.g,
                                                 color.b));
                    if let Err(err) = canvas.fill_rect(rect) {
                        println!("[{}] WARNING: `Draw::Rectangle` {:?} failed:",
                                 current_frame_id, rect);
                        println!("{}", err);
                    }
                }


                &Draw::Text(start_pos, ref text, color, options) => {
                    let mut render_line = |pos_px: Point, line: &str| {
                        let mut offset_x = 0;

                        // TODO: we need to split this by words or it
                        // won't do word breaks, split at punctuation,
                        // etc.

                        // TODO: also, we're no longer calculating the
                        // line height correctly. Needs to be set on the
                        // actual result here.
                        for chr in line.chars() {
                            let (texture_index_x, texture_index_y) = super::texture_coords_from_char(chr)
                                .unwrap_or((0, 0));

                            let src = Rect::new(texture_index_x * tilesize as i32,
                                                texture_index_y * tilesize as i32,
                                                tilesize, tilesize);
                            let dst = Rect::new(pos_px.x + offset_x,
                                                pos_px.y,
                                                tilesize, tilesize);

                            texture.set_color_mod(color.r, color.g, color.b);
                            if let Err(err) = canvas.copy(&texture, Some(src), Some(dst)) {
                                println!("[{}] WARNING: blitting {:?} to {:?} failed:",
                                         current_frame_id, src, dst);
                                println!("{}", err);
                            }

                            let advance_width =
                                engine::glyph_advance_width(chr).unwrap_or(tilesize as i32);
                            offset_x += advance_width;
                        }
                    };

                    if options.wrap && options.width > 0 {
                        // TODO: handle text alignment for wrapped text
                        let lines = engine::wrap_text(text, options.width, tilesize as i32);
                        for (index, line) in lines.iter().enumerate() {
                            let pos = (start_pos + Point::new(0, index as i32)) * tilesize as i32;
                            render_line(pos, line);
                        }
                    } else {
                        use engine::TextAlign::*;
                        let pos = match options.align {
                            Left => start_pos * tilesize as i32,
                            Right => {
                                (start_pos + (1, 0)) * tilesize as i32
                                    - Point::new(engine::text_width_px(text, tilesize as i32), 0)
                            }
                            Center => {
                                let text_width = engine::text_width_px(text, tilesize as i32);
                                let max_width = options.width * tilesize as i32;
                                if max_width < 1 || (text_width > max_width) {
                                    start_pos
                                } else {
                                    (start_pos * tilesize as i32)
                                        + Point::new((max_width - text_width) / 2, 0)
                                }
                            }
                        };
                        render_line(pos, text);
                    }
                }

                &Draw::Fade(fade, color) => {
                    screen_fade = Some((fade, color));
                }
            }
        }

        if let Some((fade, color)) = screen_fade {
            let fade = util::clampf(0.0, fade, 1.0);
            let fade = (fade * 255.0) as u8;
            let alpha = 255 - fade;
            canvas.set_draw_color(
                sdl2::pixels::Color::RGBA(color.r, color.g, color.b, alpha));
            if let Err(err) = canvas.fill_rect(None) {
                println!("[{}] WARNING: Fading screen failed:", current_frame_id);
                println!("{}", err);
            }
        }

        // println!("Pre-present duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        canvas.present();

        // println!("Code duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        // if let Some(sleep_duration) = expected_frame_length.checked_sub(frame_start_time.elapsed()) {
        //     ::std::thread::sleep(sleep_duration);
        // };

        // println!("Total frame duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);
    }
}
