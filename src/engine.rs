#![allow(dead_code)]

use std::collections::RingBuf;
use std::time::Duration;

pub use tcod::{self, Color, Console, RootConsole, KeyCode};


pub struct Display {
    fade: Option<(u8, Color)>,
}

impl Display {
    fn new() -> Display {
        Display {
            fade: None,
        }
    }

    pub fn draw_char(&mut self, x: i32, y: i32, c: char,
                     foreground: Color, background: Color) {
        self.set_background(x, y, background);
        RootConsole.put_char_ex(x, y, c, foreground, background);
    }

    pub fn write_text(&mut self, text: &str, x: i32, y: i32,
                      foreground: Color, background: Color) {
        for (i, chr) in text.char_indices() {
            self.draw_char(x + i as i32, y, chr, foreground, background);
        }
    }

    pub fn set_background(&mut self, x: i32, y: i32, color: Color) {
        RootConsole.set_char_background(x, y, color, tcod::BackgroundFlag::Set);
    }

    pub fn size(&self) -> (i32, i32) {
        (RootConsole.width(), RootConsole.height())
    }

    /// `fade_percentage` is from <0f32 to 100f32>.
    /// 0% means no fade, 100% means screen is completely filled with the `color`
    pub fn fade(&mut self, fade_percentage: f32, color: Color) {
        let fade = (fade_percentage * 255.0) as u8;
        self.fade = Some((fade, color));
    }
}

pub struct Engine {
    pub display: Display,
    pub keys: RingBuf<tcod::KeyState>,
}

impl Engine {
    pub fn new(width: i32, height: i32,
               window_title: &str, font_path: Path) -> Engine {
        Console::set_custom_font(font_path, tcod::FONT_LAYOUT_TCOD | tcod::FONT_TYPE_GREYSCALE,
                                 32, 8);
        let fullscreen = false;
        Console::init_root(width, height, window_title, fullscreen);
        Engine {
            display: Display::new(),
            keys: RingBuf::new(),
        }
    }

    pub fn main_loop<T>(&mut self, mut state: T, update: fn(T, dt: Duration, &mut Engine) -> Option<T>) {
        let default_fg = Color{r: 255, g: 255, b: 255};
        while !Console::window_closed() {
            loop {
                match tcod::Console::check_for_keypress(tcod::KEY_PRESSED) {
                    None => break,
                    Some(key) => {
                        self.keys.push_back(key);
                    }
                }
            }
            RootConsole.set_default_foreground(default_fg);
            RootConsole.clear();
            self.display.fade = None;

            match update(state,
                         Duration::microseconds((tcod::system::get_last_frame_length() * 1_000_000.0) as i64),
                         self) {
                Some(new_state) => {
                    state = new_state;
                }
                None => break,
            }
            let (width, height) = self.display.size();
            RootConsole.print_ex(width-1, height-1,
                                 tcod::BackgroundFlag::None, tcod::TextAlignment::Right,
                                 format!("FPS: {}", tcod::system::get_fps()).as_slice());
            match self.display.fade {
                Some((amount, color)) => {
                    tcod::Console::set_fade(amount, color);
                }
                None => {
                    // NOTE: Colour doesn't matter here, value 255 means no fade:
                    tcod::Console::set_fade(255, Color{r: 0, g: 0, b: 0});
                }
            }
            tcod::Console::flush();
        }
    }


    /// Return true if the given key is located anywhere in the event buffer.
    pub fn key_pressed(&self, key_code: tcod::Key) -> bool {
        for &pressed_key in self.keys.iter() {
            if pressed_key.key == key_code {
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
    pub fn read_key(&mut self, key: tcod::Key) -> bool {
        let mut len = self.keys.len();
        let mut processed = 0;
        let mut found = false;
        while processed < len {
            match self.keys.pop_front() {
                Some(pressed_key) if !found && pressed_key.key == key => {
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

}
