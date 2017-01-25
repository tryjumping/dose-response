#![allow(dead_code)]

use std::borrow::Cow;
use std::collections::VecDeque;
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
    Rectangle(Point, Color),
    Fade(f32, Color),
}


pub struct Display {
    root: RootConsole,
    // rustbox: RustBox,
    fade: Option<(u8, Color)>,
}

impl Display {
    // fn new(root: RootConsole, rustbox: RustBox) -> Display {
    fn new(root: RootConsole) -> Display {
        Display {
            root: root,
            // rustbox: rustbox,
            fade: None,
        }
    }

    pub fn within_bounds(&self, pos: Point) -> bool {
        pos >= (0, 0) && pos < self.size()
    }

    pub fn get_background(&self, pos: Point) -> Color {
        self.root.get_char_background(pos.x, pos.y)
    }

    pub fn clear_rect<P: Into<Point>, Q: Into<Point>>(&mut self, start: P, dimensions: Q, background: Color) {
        let original_background = self.root.get_default_background();
        self.root.set_default_background(background);
        let start = start.into();
        // TODO: this seems to be an invalid assert in tcod. We should
        // be able to specify the full width & height here, but it
        // crashes.
        let dimensions = dimensions.into();
        self.root.rect(start.x, start.y, dimensions.x, dimensions.y, true,
                       tcod::BackgroundFlag::Set);
        self.root.set_default_background(original_background);

    }

    pub fn size(&self) -> Point {
        (self.root.width(), self.root.height()).into()
    }

    /// `fade_percentage` is from <0f32 to 100f32>.
    /// 0% means no fade, 100% means screen is completely filled with the `color`
    pub fn fade(&mut self, fade_percentage: f32, color: Color) {
        let fade_percentage = if fade_percentage < 0.0 {
            0.0
        } else if fade_percentage > 1.0 {
            1.0
        } else {
            fade_percentage
        };
        let fade = (fade_percentage * 255.0) as u8;
        self.fade = Some((fade, color));
    }
}

#[cfg(not(debug_assertions))]
fn limit_fps_in_release(fps: i32) {
    tcod::system::set_fps(fps);
}

#[cfg(debug_assertions)]
fn limit_fps_in_release(_fps: i32) { }


pub struct Engine {
    pub display: Display,
    pub keys: VecDeque<Key>,
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
            display: Display::new(root),
            keys: VecDeque::new(),
        }
    }

    pub fn main_loop<T>(&mut self, mut state: T, update: fn(T, dt: Duration, &mut Engine, drawcalls: &mut Vec<Draw>) -> Option<T>) {
        let default_fg = Color{r: 255, g: 255, b: 255};
        let mut drawcalls = Vec::with_capacity(8192);
        while !self.display.root.window_closed() {
            // self.display.rustbox.present();
            loop {
                match self.display.root.check_for_keypress(tcod::input::KEY_PRESSED) {
                    None => break,
                    Some(key) => {
                        self.keys.push_back(key);
                    }
                }
            }
            self.display.root.set_default_foreground(default_fg);
            self.display.root.clear();
            self.display.fade = None;
            drawcalls.clear();

            match update(state,
                         Duration::microseconds((tcod::system::get_last_frame_length() * 1_000_000.0) as i64),
                         self,
                         &mut drawcalls) {
                Some(new_state) => {
                    state = new_state;
                }
                None => break,
            }

            for drawcall in &drawcalls {
                match drawcall {
                    &Draw::Char(pos, chr, foreground_color) => {
                        if self.display.within_bounds(pos) {
                            self.display.root.set_char(pos.x, pos.y, chr);
                            self.display.root.set_char_foreground(pos.x, pos.y, foreground_color);
                        }
                    }

                    &Draw::Text(start_pos, ref text, color) => {
                        for (i, chr) in text.char_indices() {
                            let pos = start_pos + (i as i32, 0);
                            if self.display.within_bounds(pos) {
                                self.display.root.set_char(pos.x, pos.y, chr);
                                self.display.root.set_char_foreground(pos.x, pos.y, color);
                            }
                        }
                    }

                    &Draw::Background(pos, background_color) => {
                        if self.display.within_bounds(pos) {
                            self.display.root.set_char_background(pos.x, pos.y,
                                                                  background_color,
                                                                  tcod::BackgroundFlag::Set);
                        }
                    }
                    _ => {},
                }
            }

            // TODO: remove this
            match self.display.fade {
                Some((amount, color)) => {
                    self.display.root.set_fade(amount, color);
                }
                None => {
                    // NOTE: Colour doesn't matter here, value 255 means no fade:
                    self.display.root.set_fade(255, Color{r: 0, g: 0, b: 0});
                }
            }
            self.display.root.flush();
        }
    }


    pub fn fps(&self) -> i32 {
        tcod::system::get_fps()
    }

    /// Return true if the given key is located anywhere in the event buffer.
    pub fn key_pressed(&self, key: Key) -> bool {
        for &pressed_key in self.keys.iter() {
            if pressed_key == key {
                return true;
            }
        }
        false
    }

    /// Consumes the first occurence of the given key in the buffer.
    ///
    /// This is useful when we have a multiple keys in the queue but we want to
    /// check for a presence of a key which should be processed immediately.
    ///
    /// Returns `true` if the key has been in the buffer.
    ///
    /// TODO: investigate using a priority queue instead.
    pub fn read_key(&mut self, key: KeyCode) -> bool {
        let mut len = self.keys.len();
        let mut processed = 0;
        let mut found = false;
        while processed < len {
            match self.keys.pop_front() {
                Some(pressed_key) if !found && pressed_key.code == key => {
                    len -= 1;
                    found = true;
                }
                Some(pressed_key) => {
                    self.keys.push_back(pressed_key);
                }
                None => return false
            }
            processed += 1;
        }
        return found;
    }

    pub fn toggle_fullscreen(&mut self) {
        let current_fullscreen_value = self.display.root.is_fullscreen();
        self.display.root.set_fullscreen(!current_fullscreen_value);
    }
}
