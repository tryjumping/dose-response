use extra::container::Deque;
use extra::ringbuf::RingBuf;
use tcod;

pub enum MainLoopState<T> {
    Running,
    NewState(T),
    Exit,
}

pub struct Color(u8, u8, u8);

pub static transparent_background: Color = Color(253, 1, 254);

impl Color {
    fn tcod(&self) -> tcod::TCOD_color_t {
        match *self { Color(r, g, b) => tcod::TCOD_color_t{r: r, g: g, b: b} }
    }
}

pub struct Display {
    priv background_console: tcod::TCOD_console_t,
    priv consoles: ~[tcod::TCOD_console_t],
}

impl Display {
    fn new(width: uint, height: uint, console_count: uint) -> Display {
        let mut result = Display {
            background_console: tcod::console_new(width, height),
            consoles: ~[],
        };
        do console_count.times {
            let con = tcod::console_new(width, height);
            tcod::console_set_key_color(con, transparent_background.tcod());
            tcod::console_set_default_background(con, transparent_background.tcod());
            result.consoles.push(con);
        }
        tcod::console_set_key_color(result.background_console, transparent_background.tcod());
        tcod::console_set_default_background(result.background_console, transparent_background.tcod());
        result
    }

    pub fn draw_char(&mut self, level: uint, x: uint, y: uint, c: char,
                     foreground: Color, background: Color) {
        assert!(level < self.consoles.len());
        self.set_background(x, y, background);
        tcod::console_put_char_ex(self.consoles[level], x, y, c,
                                  foreground.tcod(), background.tcod());
    }

    pub fn write_text(&mut self, text: &str, x: uint, y: uint,
                      foreground: Color, background: Color) {
        let level = self.consoles.len() - 1;  // write to the topmost console
        for (i, chr) in text.char_offset_iter() {
            self.draw_char(level, x+i, y, chr, foreground, background);
        }
    }

    pub fn set_background(&mut self, x: uint, y: uint, color: Color) {
        tcod::console_set_char_background(self.background_console, x, y,
                                          color.tcod(), tcod::TCOD_BKGND_NONE);
    }

    pub fn size(&self) -> (uint, uint) {
        (tcod::console_get_width(self.background_console) as uint,
         tcod::console_get_height(self.background_console) as uint)
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


pub fn main_loop<S>(width: uint, height: uint, title: &str,
                    font_path: Path,
                    initial_state: S,
                    update: &fn(&mut S, &mut Display, &mut RingBuf<Key>) -> MainLoopState<S>) {
    let fullscreen = false;
    let default_fg = Color(255, 255, 255);
    let console_count = 3;
    tcod::console_set_custom_font(font_path);
    tcod::console_init_root(width, height, title, fullscreen);
    let mut game_state = initial_state;
    let mut tcod_display = Display::new(width, height, console_count);
    let mut keys = RingBuf::new();
    while !tcod::console_is_window_closed() {
        let mut key: tcod::TCOD_key_t;
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

        tcod::console_set_default_foreground(tcod::ROOT_CONSOLE, default_fg.tcod());
        tcod::console_clear(tcod::ROOT_CONSOLE);
        tcod::console_clear(tcod_display.background_console);
        for &con in tcod_display.consoles.iter() {
            tcod::console_clear(con);
        }

        match update(&mut game_state, &mut tcod_display, &mut keys) {
            Running => (),
            NewState(new_state) => {
                game_state = new_state;
                loop;
            }
            Exit => break,
        }

        tcod::console_blit(tcod_display.background_console, 0, 0, width, height,
                           tcod::ROOT_CONSOLE, 0, 0,
                           1f, 1f);
        for &con in tcod_display.consoles.iter() {
            tcod::console_blit(con, 0, 0, width, height,
                               tcod::ROOT_CONSOLE, 0, 0,
                               1f, 1f);
        }
        tcod::console_print_ex(tcod::ROOT_CONSOLE, width-1, height-1,
                               tcod::TCOD_BKGND_NONE, tcod::TCOD_RIGHT,
                               fmt!("FPS: %?", tcod::sys_get_fps()));
        tcod::console_flush();
    }
}
