use std::path::Path;

use time::Duration;
use tcod::{self, Console, FontLayout, FontType, RootConsole};
use tcod::input::Key as TcodKey;
use tcod::input::KeyCode as TcodCode;

use color::Color;
use engine::{Draw, UpdateFn, Settings};
use keys::{Key, KeyCode};
use point::Point;

fn key_from_tcod(tcod_key: TcodKey) -> Option<Key> {
    let key_code = match tcod_key.code {
        TcodCode::Escape => Some(KeyCode::Esc),
        TcodCode::Enter => Some(KeyCode::Enter),
        TcodCode::Spacebar => Some(KeyCode::Space),

        TcodCode::Left => Some(KeyCode::Left),
        TcodCode::Right => Some(KeyCode::Right),
        TcodCode::Down => Some(KeyCode::Down),
        TcodCode::Up => Some(KeyCode::Up),

        TcodCode::Number0 => Some(KeyCode::D0),
        TcodCode::Number1 => Some(KeyCode::D1),
        TcodCode::Number2 => Some(KeyCode::D2),
        TcodCode::Number3 => Some(KeyCode::D3),
        TcodCode::Number4 => Some(KeyCode::D4),
        TcodCode::Number5 => Some(KeyCode::D5),
        TcodCode::Number6 => Some(KeyCode::D6),
        TcodCode::Number7 => Some(KeyCode::D7),
        TcodCode::Number8 => Some(KeyCode::D8),
        TcodCode::Number9 => Some(KeyCode::D9),

        TcodCode::NumPad0 => Some(KeyCode::NumPad0),
        TcodCode::NumPad1 => Some(KeyCode::NumPad1),
        TcodCode::NumPad2 => Some(KeyCode::NumPad2),
        TcodCode::NumPad3 => Some(KeyCode::NumPad3),
        TcodCode::NumPad4 => Some(KeyCode::NumPad4),
        TcodCode::NumPad5 => Some(KeyCode::NumPad5),
        TcodCode::NumPad6 => Some(KeyCode::NumPad6),
        TcodCode::NumPad7 => Some(KeyCode::NumPad7),
        TcodCode::NumPad8 => Some(KeyCode::NumPad8),
        TcodCode::NumPad9 => Some(KeyCode::NumPad9),

        TcodCode::F1 => Some(KeyCode::F1),
        TcodCode::F2 => Some(KeyCode::F2),
        TcodCode::F3 => Some(KeyCode::F3),
        TcodCode::F4 => Some(KeyCode::F4),
        TcodCode::F5 => Some(KeyCode::F5),
        TcodCode::F6 => Some(KeyCode::F6),
        TcodCode::F7 => Some(KeyCode::F7),
        TcodCode::F8 => Some(KeyCode::F8),
        TcodCode::F9 => Some(KeyCode::F9),
        TcodCode::F10 => Some(KeyCode::F10),
        TcodCode::F11 => Some(KeyCode::F11),
        TcodCode::F12 => Some(KeyCode::F12),

        TcodCode::Char => match tcod_key.printable {
            'a' => Some(KeyCode::A),
            'b' => Some(KeyCode::B),
            'c' => Some(KeyCode::C),
            'd' => Some(KeyCode::D),
            'e' => Some(KeyCode::E),
            'f' => Some(KeyCode::F),
            'g' => Some(KeyCode::G),
            'h' => Some(KeyCode::H),
            'i' => Some(KeyCode::I),
            'j' => Some(KeyCode::J),
            'k' => Some(KeyCode::K),
            'l' => Some(KeyCode::L),
            'm' => Some(KeyCode::M),
            'n' => Some(KeyCode::N),
            'o' => Some(KeyCode::O),
            'p' => Some(KeyCode::P),
            'q' => Some(KeyCode::Q),
            'r' => Some(KeyCode::R),
            's' => Some(KeyCode::S),
            't' => Some(KeyCode::T),
            'u' => Some(KeyCode::U),
            'v' => Some(KeyCode::V),
            'w' => Some(KeyCode::W),
            'x' => Some(KeyCode::X),
            'y' => Some(KeyCode::Y),
            'z' => Some(KeyCode::Z),

            _ => None,
        },
        _ => None,
    };

    key_code.map(|code| Key {
        code: code,
        alt: tcod_key.alt,
        ctrl: tcod_key.ctrl,
        shift: tcod_key.shift,
    })
}


impl Into<tcod::Color> for Color {
    fn into(self) -> tcod::Color {
        tcod::Color {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}


#[cfg(not(debug_assertions))]
fn limit_fps_in_release(fps: i32) {
    tcod::system::set_fps(fps);
}

#[cfg(debug_assertions)]
fn limit_fps_in_release(_fps: i32) { }


pub struct Engine {
    root: RootConsole,
    settings: Settings,
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
        root.set_default_background(default_background.into());

        // Limit FPS in the release mode
        limit_fps_in_release(60);


        Engine {
            root: root,
            settings: Settings {
                fullscreen: false,
            },
        }
    }

    pub fn main_loop<T>(&mut self, mut state: T, update: UpdateFn<T>) {
        let default_fg = Color{r: 255, g: 255, b: 255};
        let mut drawcalls = Vec::with_capacity(8192);
        let display_size = Point {x: self.root.width(), y: self.root.height()};
        let mut keys = vec![];
        while !self.root.window_closed() {
            keys.clear();
            loop {
                match self.root.check_for_keypress(tcod::input::KEY_PRESSED) {
                    None => break,
                    Some(key) => {
                        // NOTE: Ignore keys that we can't convert yet
                        if let Some(key) = key_from_tcod(key) {
                            keys.push(key);
                        }
                    }
                }
            }
            self.root.set_default_foreground(default_fg.into());
            self.root.clear();
            drawcalls.clear();

            match update(state,
                         Duration::microseconds((tcod::system::get_last_frame_length() * 1_000_000.0) as i64),
                         display_size,
                         tcod::system::get_fps(),
                         &keys,
                         self.settings,
                         &mut drawcalls) {
                Some((new_settings, new_state)) => {
                    state = new_state;
                    if self.settings.fullscreen != new_settings.fullscreen {
                        self.root.set_fullscreen(new_settings.fullscreen);
                    }
                    self.settings = new_settings;
                }
                None => break,
            }

            // NOTE: reset the fade value. Fade in tcod is stateful,
            // so if we don't do this, it will stay there even if it's
            // no longer set.
            self.root.set_fade(255, tcod::Color{r: 0, g: 0, b: 0});

            for drawcall in &drawcalls {
                match drawcall {
                    &Draw::Char(pos, chr, foreground_color) => {
                        if self.within_bounds(pos) {
                            self.root.set_char(pos.x, pos.y, chr);
                            self.root.set_char_foreground(pos.x, pos.y, foreground_color.into());
                        }
                    }

                    &Draw::Text(start_pos, ref text, color) => {
                        for (i, chr) in text.char_indices() {
                            let pos = start_pos + (i as i32, 0);
                            if self.within_bounds(pos) {
                                self.root.set_char(pos.x, pos.y, chr);
                                self.root.set_char_foreground(pos.x, pos.y, color.into());
                            }
                        }
                    }

                    &Draw::Background(pos, background_color) => {
                        if self.within_bounds(pos) {
                            self.root.set_char_background(pos.x, pos.y,
                                                                  background_color.into(),
                                                                  tcod::BackgroundFlag::Set);
                        }
                    }

                    &Draw::Rectangle(top_left, dimensions, background) => {
                        let original_background = self.root.get_default_background();
                        self.root.set_default_background(background.into());
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
                        self.root.set_fade(fade, color.into());
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

}
