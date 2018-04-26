use color::{Color, ColorAlpha};
use engine::{self, Drawcall, Mouse, Settings, TextMetrics, TextOptions, UpdateFn};
use game::RunningState;
use keys::KeyCode;
use point::Point;
use rect::Rectangle;
use state::State;
use util;

use std::time::{Duration, Instant};

use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::{self, Keycode as BackendKey};
use sdl2::pixels::{Color as SDLColor, PixelFormatEnum};
use sdl2::rect::Rect as SDLRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::Window;
use image;


// const DESIRED_FPS: u64 = 60;
// const EXPECTED_FRAME_LENGTH: Duration = Duration::from_millis(1000 / DESIRED_FPS);
const SDL_DRAWCALL_CAPACITY: usize = 25_000;

pub struct Metrics {
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


impl Into<SDLRect> for Rectangle {
    fn into(self) -> SDLRect {
        SDLRect::new(self.top_left().x, self.top_left().y,
                     self.size().x as u32, self.size().y as u32)
    }
}

impl Into<SDLColor> for ColorAlpha {
    fn into(self) -> SDLColor {
        SDLColor::RGBA(self.rgb.r, self.rgb.g, self.rgb.b, self.alpha)
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


fn sdl_render(canvas: &mut Canvas<Window>,
              texture: &mut Texture,
              clear_color: Color,
              drawcalls: &[Drawcall])
{
    use self::Drawcall::*;
    canvas.set_draw_color(
        sdl2::pixels::Color::RGB(clear_color.r, clear_color.g, clear_color.b));
    canvas.clear();

    for dc in drawcalls.iter() {
        // TODO: collect the results? Or at least the errors?
        match dc {
            &Rectangle(rect, color) => {
                canvas.set_draw_color(color.into());
                canvas.fill_rect(rect.map(Into::into)).unwrap();
            }
            &Image(src, dst, color) => {
                texture.set_color_mod(color.r, color.g, color.b);
                canvas.copy(&texture, Some(src.into()), Some(dst.into())).unwrap();
            }
        }
    }

    canvas.present();
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
        //.software()
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
    // TODO: calculate this from the real window size
    let display_px = Point::new(desired_window_width as i32, desired_window_height as i32);
    let mut display = engine::Display::new(
        display_size, Point::from_i32(display_size.y / 2), tilesize as i32);
    let mut sdl_drawcalls = Vec::with_capacity(SDL_DRAWCALL_CAPACITY);
    let overall_max_drawcall_count = 0;
    let mut overall_max_sdl_drawcall_count = 0;
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

                Event::TextInput { text, .. } => {
                    if text.contains('?') {
                        let key = super::Key {
                            code: KeyCode::QuestionMark,
                            alt: false,
                            ctrl: false,
                            shift: false,
                        };
                        keys.push(key);
                    }
                }

                Event::MouseMotion {x, y, ..} => {
                    let x = util::clamp(0, x, display_px.x - 1);
                    let y = util::clamp(0, y, display_px.y - 1);
                    mouse.screen_pos = Point { x, y };

                    let tile_width = display_px.x / display_size.x;
                    let mouse_tile_x = x / tile_width;

                    let tile_height = display_px.y / display_size.y;
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
            &mut display,
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

        // println!("Pre-draw duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);


        // NOTE: Turn the Engine drawcalls into SDL drawcalls.
        //
        // Instead of calling the SDL rendering functions directly,
        // record the actions we would have done (e.g. fill rect,
        // copy, set draw color) and store them in the `sdl_drawcalls`
        // Vec.
        //
        // It sounds a little strange, but it isolates calling into C
        // into its own block and hopefully lets the Rust compiler
        // optimise the drawcall processing better.
        //
        // More importantly, it'll be useful in profiling because
        // we'll be able to measure where is our "rendering" time
        // actually spent: drawcall processing or calling SDL
        // functions?

        sdl_drawcalls.clear();
        display.push_drawcalls(display_px, &mut sdl_drawcalls);

        if sdl_drawcalls.len() > overall_max_sdl_drawcall_count {
            overall_max_sdl_drawcall_count = sdl_drawcalls.len();
        }

        if sdl_drawcalls.len() > SDL_DRAWCALL_CAPACITY {
            println!(
                "Warning: SDL drawcall count exceeded initial capacity {}. Current count: {}.",
                sdl_drawcalls.len(),
                SDL_DRAWCALL_CAPACITY
            );
        }

        // println!("Pre-present duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        // NOTE: render
        sdl_render(&mut canvas, &mut texture, default_background, &sdl_drawcalls);

        // println!("Code duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

        // if let Some(sleep_duration) = EXPECTED_FRAME_LENGTH.checked_sub(frame_start_time.elapsed()) {
        //     ::std::thread::sleep(sleep_duration);
        // };

        // println!("Total frame duration: {:?}ms",
        //          frame_start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0);

    }

    println!("Engine drawcall count: {}. Capacity: {}.",
             overall_max_drawcall_count, engine::DRAWCALL_CAPACITY);
    println!("SDL drawcall count: {}. Capacity: {}.",
             overall_max_sdl_drawcall_count, SDL_DRAWCALL_CAPACITY);
}
