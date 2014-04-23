#![allow(dead_code)]

use std::rc::Rc;
use std::cell::RefCell;

use collections::deque::Deque;
use collections::ringbuf::RingBuf;
pub use tcod::{Color, Console};
pub use key = tcod::key_code;
pub use tcod;

pub enum MainLoopState<T> {
    Running,
    NewState(T),
    Exit,
}

pub static transparent_background: Color = Color{r: 253, g: 1, b: 254};

pub struct Display {
    background_console: Console,
    consoles: Vec<Console>,
    fade: Option<(u8, Color)>,
}

impl Display {
    fn new(width: int, height: int, console_count: uint) -> Display {
        let mut result = Display {
            background_console: Console::new(width, height),
            consoles: Vec::new(),
            fade: None,
        };
        for _ in range(0, console_count) {
            let mut con = Console::new(width, height);
            con.set_key_color(transparent_background);
            con.set_default_background(transparent_background);
            result.consoles.push(con);
        }
        result.background_console.set_key_color(transparent_background);
        result.background_console.set_default_background(transparent_background);
        result
    }

    pub fn draw_char(&mut self, level: uint, x: int, y: int, c: char,
                     foreground: Color, background: Color) {
        assert!(level < self.consoles.len());
        self.set_background(x, y, background);
        self.consoles.get_mut(level).put_char_ex(x, y, c,
                                                 foreground, background);
    }

    pub fn write_text(&mut self, text: &str, x: int, y: int,
                      foreground: Color, background: Color) {
        let level = self.consoles.len() - 1;  // write to the topmost console
        for (i, chr) in text.char_indices() {
            self.draw_char(level, x + i as int, y, chr, foreground, background);
        }
    }

    pub fn set_background(&mut self, x: int, y: int, color: Color) {
        self.background_console.set_char_background(x, y,
                                                    color, tcod::background_flag::None);
    }

    pub fn size(&self) -> (int, int) {
        (self.background_console.width(),
         self.background_console.height())
    }

    pub fn fade(&mut self, fade_ammount: u8, color: Color) {
        self.fade = Some((fade_ammount, color));
    }
}

pub struct Engine {
    display: Rc<RefCell<Display>>,
    keys: Rc<RefCell<RingBuf<tcod::KeyState>>>,
    root_console: tcod::Console,
}

impl Engine {
    pub fn new(width: int, height: int,
               window_title: &str, font_path: Path) -> Engine {
        Console::set_custom_font(font_path);
        let fullscreen = false;
        let console_count = 3;
        Engine {
            display: Rc::new(RefCell::new(Display::new(width, height, console_count))),
            keys: Rc::new(RefCell::new(RingBuf::new())),
            root_console: Console::init_root(width, height, window_title, fullscreen),

        }
    }

    pub fn display(&self) -> Rc<RefCell<Display>> {
        self.display.clone()
    }

    pub fn keys(&self) -> Rc<RefCell<RingBuf<tcod::KeyState>>> {
        self.keys.clone()
    }

    pub fn main_loop<T>(&mut self, mut state: T, update: fn(T, dt_s: f32) -> Option<T>) {
        let default_fg = Color::new(255, 255, 255);
        while !Console::window_closed() {
            let mut key: tcod::KeyState;
            loop {
                match self.root_console.check_for_keypress(tcod::Pressed) {
                    None => break,
                    Some(key) => {
                        self.keys.borrow_mut().push_back(key);
                    }
                }
            }
            self.root_console.set_default_foreground(default_fg);
            self.root_console.clear();
            self.display.borrow_mut().background_console.clear();
            for con in self.display.borrow_mut().consoles.mut_iter() {
                con.clear();
            }
            self.display.borrow_mut().fade = None;

            match update(state, tcod::system::get_last_frame_length()) {
                Some(new_state) => {
                    state = new_state;
                    continue;
                }
                None => break,
            }
            let (width, height) = self.display.borrow().size();
            Console::blit(&self.display.borrow_mut().background_console, 0, 0, width, height,
                          &mut self.root_console, 0, 0,
                          1f32, 1f32);
            for con in self.display.borrow_mut().consoles.mut_iter() {
                Console::blit(con, 0, 0, width, height,
                              &mut self.root_console, 0, 0,
                              1f32, 1f32);
            }
            self.root_console.print_ex(width-1, height-1,
                                  tcod::background_flag::None, tcod::Right,
                                  format!("FPS: {}", tcod::system::get_fps()));
            match self.display.borrow().fade {
                Some((amount, color)) => self.root_console.set_fade(amount, color),
                // colour doesn't matter, value 255 means no fade:
                None => self.root_console.set_fade(255, Color{r: 0, g: 0, b: 0}),
            }
            self.root_console.flush();
        }
    }
}
