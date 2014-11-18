#![allow(dead_code)]

use std::collections::RingBuf;
pub use tcod::{mod, Color, Console, RootConsole, KeyCode};


pub struct Display {
    fade: Option<(u8, Color)>,
}

impl Display {
    fn new() -> Display {
        Display {
            fade: None,
        }
    }

    pub fn draw_char(&mut self, x: int, y: int, c: char,
                     foreground: Color, background: Color) {
        self.set_background(x, y, background);
        RootConsole.put_char_ex(x, y, c, foreground, background);
    }

    pub fn write_text(&mut self, text: &str, x: int, y: int,
                      foreground: Color, background: Color) {
        for (i, chr) in text.char_indices() {
            self.draw_char(x + i as int, y, chr, foreground, background);
        }
    }

    pub fn set_background(&mut self, x: int, y: int, color: Color) {
        RootConsole.set_char_background(x, y, color, tcod::BackgroundFlag::None);
    }

    pub fn size(&self) -> (int, int) {
        (RootConsole.width(), RootConsole.height())
    }

    pub fn fade(&mut self, fade_ammount: u8, color: Color) {
        self.fade = Some((fade_ammount, color));
    }
}

pub struct Engine {
    pub display: Display,
    pub keys: RingBuf<tcod::KeyState>,
}

impl Engine {
    pub fn new(width: int, height: int,
               window_title: &str, font_path: Path) -> Engine {
        use tcod::FontFlags::{LayoutTcod, TypeGreyscale};
        Console::set_custom_font(font_path, [LayoutTcod, TypeGreyscale],
                                 32, 8);
        let fullscreen = false;
        Console::init_root(width, height, window_title, fullscreen);
        Engine {
            display: Display::new(),
            keys: RingBuf::new(),
        }
    }

    pub fn main_loop<T>(&mut self, mut state: T, update: fn(T, dt_s: f32, &mut Engine) -> Option<T>) {
        let default_fg = Color::new(255, 255, 255);
        while !Console::window_closed() {
            loop {
                match tcod::Console::check_for_keypress(tcod::Pressed) {
                    None => break,
                    Some(key) => {
                        self.keys.push_back(key);
                    }
                }
            }
            RootConsole.set_default_foreground(default_fg);
            RootConsole.clear();
            self.display.fade = None;

            match update(state, tcod::system::get_last_frame_length(), self) {
                Some(new_state) => {
                    state = new_state;
                }
                None => break,
            }
            let (width, height) = self.display.size();
            RootConsole.print_ex(width-1, height-1,
                                 tcod::BackgroundFlag::None, tcod::Right,
                                 format!("FPS: {}", tcod::system::get_fps()).as_slice());
            match self.display.fade {
                Some((amount, color)) => tcod::Console::set_fade(amount, color),
                // colour doesn't matter, value 255 means no fade:
                None => tcod::Console::set_fade(255, Color{r: 0, g: 0, b: 0}),
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
