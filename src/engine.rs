use extra::container::Deque;
use extra::ringbuf::RingBuf;
pub use tcod::Color;
pub use tcod;

pub enum MainLoopState<T> {
    Running,
    NewState(T),
    Exit,
}

pub static transparent_background: Color = Color{r: 253, g: 1, b: 254};

pub struct Display {
    priv background_console: tcod::console_t,
    priv consoles: ~[tcod::console_t],
    priv fade: Option<(u8, Color)>,
}

impl Display {
    fn new(width: int, height: int, console_count: uint) -> Display {
        let mut result = Display {
            background_console: tcod::console_new(width, height),
            consoles: ~[],
            fade: None,
        };
        do console_count.times {
            let con = tcod::console_new(width, height);
            tcod::console_set_key_color(con, transparent_background);
            tcod::console_set_default_background(con, transparent_background);
            result.consoles.push(con);
        }
        tcod::console_set_key_color(result.background_console, transparent_background);
        tcod::console_set_default_background(result.background_console, transparent_background);
        result
    }

    pub fn draw_char(&mut self, level: uint, x: int, y: int, c: char,
                     foreground: Color, background: Color) {
        assert!(level < self.consoles.len());
        self.set_background(x, y, background);
        tcod::console_put_char_ex(self.consoles[level], x, y, c,
                                  foreground, background);
    }

    pub fn write_text(&mut self, text: &str, x: int, y: int,
                      foreground: Color, background: Color) {
        let level = self.consoles.len() - 1;  // write to the topmost console
        for (i, chr) in text.char_offset_iter() {
            self.draw_char(level, x + i as int, y, chr, foreground, background);
        }
    }

    pub fn set_background(&mut self, x: int, y: int, color: Color) {
        tcod::console_set_char_background(self.background_console, x, y,
                                          color, tcod::BKGND_NONE);
    }

    pub fn size(&self) -> (int, int) {
        (tcod::console_get_width(self.background_console),
         tcod::console_get_height(self.background_console))
    }

    pub fn fade(&mut self, fade_ammount: u8, color: Color) {
        self.fade = Some((fade_ammount, color));
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        tcod::console_delete(self.background_console);
        for &con in self.consoles.iter() {
            tcod::console_delete(con);
        }
    }
}

pub struct Key {
    code: int,
    char: char,
    left_alt: bool,
    right_alt: bool,
    left_ctrl: bool,
    right_ctrl: bool,
    shift: bool,
}

impl Key {
    pub fn alt(&self) -> bool { self.left_alt || self.right_alt }
    pub fn ctrl(&self) -> bool { self.left_ctrl || self.right_ctrl }
    pub fn shift(&self) -> bool { self.shift }
}


pub fn main_loop<S>(width: int, height: int, title: &str,
                    font_path: Path,
                    initial_state: S,
                    update: fn(&mut S, &mut Display, &mut RingBuf<Key>, dt_s: float) -> MainLoopState<S>) {
    let fullscreen = false;
    let default_fg = Color::new(255, 255, 255);
    let console_count = 3;
    tcod::console_set_custom_font(font_path);
    tcod::console_init_root(width, height, title, fullscreen);
    let mut game_state = initial_state;
    let mut tcod_display = Display::new(width, height, console_count);
    let mut keys = RingBuf::new();
    while !tcod::console_is_window_closed() {
        let mut key: tcod::Key;
        loop {
            key = tcod::console_check_for_keypress(tcod::KeyPressed);
            match key.vk {
                0 => break,
                _ => {
                    keys.push_back(Key{
                        code: key.vk as int,
                        char: key.c as u8 as char,
                        left_alt: key.lalt != 0,
                        right_alt: key.ralt != 0,
                        left_ctrl: key.lctrl != 0,
                        right_ctrl: key.rctrl != 0,
                        shift: key.shift != 0,
                    });
                }
            }
        }

        tcod::console_set_default_foreground(tcod::ROOT_CONSOLE, default_fg);
        tcod::console_clear(tcod::ROOT_CONSOLE);
        tcod::console_clear(tcod_display.background_console);
        for &con in tcod_display.consoles.iter() {
            tcod::console_clear(con);
        }
        tcod_display.fade = None;

        match update(&mut game_state,
                     &mut tcod_display,
                     &mut keys,
                     tcod::sys_get_last_frame_length()) {
            Running => (),
            NewState(new_state) => {
                game_state = new_state;
                continue;
            }
            Exit => break,
        }

        tcod::console_blit(tcod_display.background_console, 0, 0, width, height,
                           tcod::ROOT_CONSOLE, 0, 0,
                           1f32, 1f32);
        for &con in tcod_display.consoles.iter() {
            tcod::console_blit(con, 0, 0, width, height,
                               tcod::ROOT_CONSOLE, 0, 0,
                               1f32, 1f32);
        }
        tcod::console_print_ex(tcod::ROOT_CONSOLE, width-1, height-1,
                               tcod::BKGND_NONE, tcod::Right,
                               fmt!("FPS: %?", tcod::sys_get_fps()));
        match tcod_display.fade {
            Some((amount, color)) => tcod::console_set_fade(amount, color),
            // colour doesn't matter, value 255 means no fade:
            None => tcod::console_set_fade(255, Color{r: 0, g: 0, b: 0}),
        }
        tcod::console_flush();
    }
}
