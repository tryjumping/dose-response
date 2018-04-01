use color::Color;
use engine::{self, Draw, Mouse, Settings, TextMetrics, UpdateFn};
use game::RunningState;
use point::Point;
use state::State;
use util;

use std::time::{Duration, Instant};

use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
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
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                Event::KeyDown {keycode: Some(Keycode::F), ..} => {
                    use sdl2::video::FullscreenType::*;
                    println!("Toggling fullscreen");
                    let fullscreen_state = canvas.window().fullscreen_state();
                    println!("Current state: {:?}", fullscreen_state);
                    let result = match fullscreen_state {
                        Off => {
                            println!("Switching to (desktop-type) fullscreen");
                            canvas.window_mut().set_fullscreen(Desktop)
                        }
                        True => {
                            println!("Switching fullscreen OFF");
                            canvas.window_mut().set_fullscreen(Off)
                        }
                        Desktop => {
                            println!("Switching fullscreen OFF");
                            canvas.window_mut().set_fullscreen(Off)
                        }
                    };
                    println!("Fullscreen result: {:?}", result);
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

        keys.clear();


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
                    let (texture_index_x, texture_index_y) = super::texture_coords_from_char(chr)
                        .unwrap_or((0, 0));
                    let src = Rect::new(texture_index_x * tilesize as i32,
                                        texture_index_y * tilesize as i32,
                                        tilesize, tilesize);
                    let dst = Rect::new(pos.x * tilesize as i32 + offset_px.x,
                                        pos.y * tilesize as i32 + offset_px.y,
                                        tilesize, tilesize);

                    texture.set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);
                    if let Err(err) = canvas.copy(&texture, Some(src), Some(dst)) {
                        println!("[{}] WARNING: blitting {:?} to {:?} failed:",
                                 current_frame_id, src, dst);
                        println!("{}", err);
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
