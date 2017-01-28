use std::borrow::Cow;
use std::path::Path;

use time::Duration;
pub use tcod::{self, Color, Console, FontLayout, FontType, RootConsole};
pub use tcod::input::{Key, KeyCode};
// use rustbox::{self, RustBox};

use point::Point;


#[derive(Debug, Clone)]
pub enum Draw {
    Char(Point, char, Color),
    Text(Point, Cow<'static, str>, Color),
    Background(Point, Color),
    Rectangle(Point, Point, Color),
    Fade(f32, Color),
}


#[cfg(not(debug_assertions))]
fn limit_fps_in_release(fps: i32) {
    tcod::system::set_fps(fps);
}

#[cfg(debug_assertions)]
fn limit_fps_in_release(_fps: i32) { }


pub struct Engine {
    root: RootConsole,
}

impl Engine {
    pub fn new(display_size: Point, default_background: Color,
               window_title: &str, font_path: &Path) -> Engine {
        let mut root = RootConsole::initializer()
            .title(window_title)
            .size(display_size.x, display_size.y)
            .font(font_path, FontLayout::Tcod)
            .font_type(FontType::Greyscale)
            .init();
        root.set_default_background(default_background);

        // Limit FPS in the release mode
        limit_fps_in_release(60);

        // let rustbox = RustBox::init(Default::default()).expect(
        //     "Failed to initialise rustbox!");
        // let terminal_size = (rustbox.width() as i32, rustbox.height() as i32);
        // if (terminal_size.0 < width) || (terminal_size.1 < height) {
        //     drop(rustbox);
        //     panic!("The terminal size is too small. Current size: {:?}, required size: {:?}",
        //              terminal_size, (width, height));
        // }

        Engine {
            // display: Display::new(root, rustbox),
            root: root,
        }
    }

    pub fn main_loop<T>(&mut self, mut state: T, update: fn(T, dt: Duration, size: Point, fps: i32, keys: &[Key], drawcalls: &mut Vec<Draw>) -> Option<T>) {
        let default_fg = Color{r: 255, g: 255, b: 255};
        let mut drawcalls = Vec::with_capacity(8192);
        let display_size = Point {x: self.root.width(), y: self.root.height()};
        let keys = vec![];
        while !self.root.window_closed() {
            // self.display.rustbox.present();
            keys.clear();
            loop {
                match self.root.check_for_keypress(tcod::input::KEY_PRESSED) {
                    None => break,
                    Some(key) => {
                        keys.push(key);
                    }
                }
            }
            self.root.set_default_foreground(default_fg);
            self.root.clear();
            drawcalls.clear();

            match update(state,
                         Duration::microseconds((tcod::system::get_last_frame_length() * 1_000_000.0) as i64),
                         display_size,
                         tcod::system::get_fps(),
                         &keys,
                         &mut drawcalls) {
                Some(new_state) => {
                    state = new_state;
                }
                None => break,
            }

            // NOTE: reset the fade value. Fade in tcod is stateful,
            // so if we don't do this, it will stay there even if it's
            // no longer set.
            self.root.set_fade(255, Color{r: 0, g: 0, b: 0});

            for drawcall in &drawcalls {
                match drawcall {
                    &Draw::Char(pos, chr, foreground_color) => {
                        if self.within_bounds(pos) {
                            self.root.set_char(pos.x, pos.y, chr);
                            self.root.set_char_foreground(pos.x, pos.y, foreground_color);
                        }
                    }

                    &Draw::Text(start_pos, ref text, color) => {
                        for (i, chr) in text.char_indices() {
                            let pos = start_pos + (i as i32, 0);
                            if self.within_bounds(pos) {
                                self.root.set_char(pos.x, pos.y, chr);
                                self.root.set_char_foreground(pos.x, pos.y, color);
                            }
                        }
                    }

                    &Draw::Background(pos, background_color) => {
                        if self.within_bounds(pos) {
                            self.root.set_char_background(pos.x, pos.y,
                                                                  background_color,
                                                                  tcod::BackgroundFlag::Set);
                        }
                    }

                    &Draw::Rectangle(top_left, dimensions, background) => {
                        let original_background = self.root.get_default_background();
                        self.root.set_default_background(background);
                        // TODO: this seems to be an invalid assert in tcod. We should
                        // be able to specify the full width & height here, but it
                        // crashes.
                        self.root.rect(top_left.x, top_left.y, dimensions.x, dimensions.y, true,
                                       tcod::BackgroundFlag::Set);
                        self.root.set_default_background(original_background);
                    }

                    &Draw::Fade(fade_percentage, color) => {
                        let fade_percentage = if fade_percentage < 0.0 {
                            0.0
                        } else if fade_percentage > 1.0 {
                            1.0
                        } else {
                            fade_percentage
                        };
                        let fade = (fade_percentage * 255.0) as u8;
                        self.root.set_fade(fade, color);
                    },
                }
            }

            self.root.flush();
        }
    }

    pub fn within_bounds(&self, pos: Point) -> bool {
        let size = Point {x: self.root.width(), y: self.root.height()};
        pos >= (0, 0) && pos < size
    }

    pub fn toggle_fullscreen(&mut self) {
        let current_fullscreen_value = self.root.is_fullscreen();
        self.root.set_fullscreen(!current_fullscreen_value);
    }
}
