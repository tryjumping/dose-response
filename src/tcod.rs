#[allow(dead_code)];

use std::cast;
use std::path;

pub use self::ffi::Color;
pub use self::ffi::console_t;
pub use self::ffi::BKGND_NONE;
pub use self::ffi::key_code;
pub use self::ffi::Key;
pub use self::ffi::*;
pub use self::ffi::Right;

use self::ffi::{c_int, c_float, c_bool};


pub struct Map {
    priv tcod_map: ffi::map_t,
}

pub struct Path {
    priv tcod_path: ffi::path_t,
}

impl Map {
    pub fn new(width: int, height: int) -> Map {
        assert!(width > 0 && height > 0);
        unsafe {
            Map{tcod_map: ffi::TCOD_map_new(width as c_int, height as c_int)}
        }
    }

    pub fn size(&self) -> (int, int) {
        unsafe {
            (ffi::TCOD_map_get_width(self.tcod_map) as int,
             ffi::TCOD_map_get_height(self.tcod_map) as int)
        }
    }

    pub fn set(&mut self, x: int, y: int, transparent: bool, walkable: bool) {
        assert!(x >= 0 && y >= 0);
        unsafe {
            ffi::TCOD_map_set_properties(self.tcod_map, x as c_int, y as c_int,
                                         transparent as c_bool,
                                         walkable as c_bool);
        }
    }

    pub fn is_walkable(&self, x: int, y: int) -> bool {
        assert!(x >= 0 && y >= 0);
        unsafe {
            ffi::TCOD_map_is_walkable(self.tcod_map, x as c_int, y as c_int) != 0
        }
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        unsafe {
            ffi::TCOD_map_delete(self.tcod_map)
        }
    }
}


impl Path {
    pub fn new_using_map(map: Map, diagonal_cost: f32) -> Path {
        unsafe {
            Path {
                tcod_path: ffi::TCOD_path_new_using_map(map.tcod_map,
                                                        diagonal_cost as c_float)
            }
        }
    }

    pub fn new_using_function<T>(map_width: int, map_height: int,
                                 path_cb: ffi::path_callback_t,
                                 user_data: &T,
                                 diagonal_cost: f32) -> Path {
        assert!(map_width > 0 && map_height > 0);
        unsafe {
            Path {
                tcod_path: ffi::TCOD_path_new_using_function(map_width as c_int,
                                                             map_height as c_int,
                                                             path_cb,
                                                             cast::transmute(user_data),
                                                             diagonal_cost as c_float)
            }
        }
    }

    pub fn find(&mut self,
                from_x: int, from_y: int,
                to_x: int, to_y: int)
                -> bool {
        assert!(from_x >= 0 && from_y >= 0 && to_x >= 0 && to_y >= 0);
        unsafe {
            ffi::TCOD_path_compute(self.tcod_path,
                                   from_x as c_int, from_y as c_int,
                                   to_x as c_int, to_y as c_int) != 0
        }
    }

    pub fn walk(&mut self, recalculate_when_needed: bool)
                -> Option<(int, int)> {
        unsafe {
            let mut x: c_int = 0;
            let mut y: c_int = 0;
            match ffi::TCOD_path_walk(self.tcod_path, &mut x, &mut y,
                                      recalculate_when_needed as c_bool) != 0 {
                true => Some((x as int, y as int)),
                false => None,
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        unsafe {
            ffi::TCOD_path_is_empty(self.tcod_path) != 0
        }
    }

    pub fn len(&self) -> int {
        unsafe {
            ffi::TCOD_path_size(self.tcod_path) as int
        }
    }

    pub fn destination(&self) -> (int, int) {
        unsafe {
            let mut x: c_int = 0;
            let mut y: c_int = 0;
            ffi::TCOD_path_get_destination(self.tcod_path, &mut x, &mut y);
            (x as int, y as int)
        }
    }

}

impl Drop for Path {
    fn drop(&mut self) {
        unsafe {
            ffi::TCOD_path_delete(self.tcod_path);
        }
    }
}


mod ffi {
    pub use std::libc::{c_int, c_char, uint8_t, c_void, c_float};

    pub type c_bool = uint8_t;
    pub type console_t = *c_void;
    pub type map_t = *c_void;
    pub type path_t = *c_void;
    pub type path_callback_t = extern "C"
        fn(xf: c_int, _yf: c_int, _xt: c_int, _yt: c_int, ud: *c_void)
           -> c_float;

    #[repr(C)]
    pub enum renderer_t {
        RENDERER_GLSL,
        RENDERER_OPENGL,
        RENDERER_SDL,
        NB_RENDERERS,
    }

    #[repr(C)]
    pub enum font_flags_t {
        FONT_LAYOUT_ASCII_INCOL=1,
        FONT_LAYOUT_ASCII_INROW=2,
        FONT_TYPE_GREYSCALE=4,
        FONT_LAYOUT_TCOD=8,
    }

    pub mod key_code {
        #[deriving(Eq, ToStr)]
        #[repr(C)]
        pub enum KeyCode {
            NoKey,
            Escape,
            Backspace,
            Tab,
            Enter,
            Shift,
            Control,
            Alt,
            Pause,
            Capslock,
            Pageup,
            Pagedown,
            End,
            Home,
            Up,
            Left,
            Right,
            Down,
            PrintScreen,
            Insert,
            Delete,
            LeftWin,
            RightWin,
            Apps,
            // The numbers on the alphanum section of the keyboard
            Number0,
            Number1,
            Number2,
            Number3,
            Number4,
            Number5,
            Number6,
            Number7,
            Number8,
            Number9,
            // The numbers on the numeric keypad
            NumPad0,
            NumPad1,
            NumPad2,
            NumPad3,
            NumPad4,
            NumPad5,
            NumPad6,
            NumPad7,
            NumPad8,
            NumPad9,
            NumPadAdd,
            NumPadSubtract,
            NumPadDivide,
            NumPadMultiply,
            NumPadDecimal,
            NumPadEnter,
            F1,
            F2,
            F3,
            F4,
            F5,
            F6,
            F7,
            F8,
            F9,
            F10,
            F11,
            F12,
            NUMLOCK,
            SCROLLLOCK,
            Spacebar,
            Char,
        }
    }

    #[repr(C)]
    pub struct Key {
        vk: key_code::KeyCode,
        c: c_char,
        pressed: c_bool,
        lalt: c_bool,
        lctrl: c_bool,
        ralt: c_bool,
        rctrl: c_bool,
        shift: c_bool,
    }

    #[deriving(Clone, Eq)]
    #[repr(C)]
    pub struct Color {
        r: uint8_t,
        g: uint8_t,
        b: uint8_t,
    }

    impl Color {
        pub fn new(red: u8, green: u8, blue: u8) -> Color {
            Color{r: red as uint8_t, g: green as uint8_t, b: blue as uint8_t}
        }
    }

    #[repr(C)]
    pub enum TextAlignment {
        Left,
        Right,
        Center,
    }

    #[repr(C)]
    pub enum BackgroundFlag {
        BKGND_NONE,
        BKGND_SET,
        BKGND_MULTIPLY,
        BKGND_LIGHTEN,
        BKGND_DARKEN,
        BKGND_SCREEN,
        BKGND_COLOR_DODGE,
        BKGND_COLOR_BURN,
        BKGND_ADD,
        BKGND_ADDA,
        BKGND_BURN,
        BKGND_OVERLAY,
        BKGND_ALPH,
        BKGND_DEFAULT
    }

    #[link(name = "tcod")]
    extern "C" {
        pub fn TCOD_sys_set_fps(val: c_int) -> ();
        pub fn TCOD_sys_get_fps() -> c_int;
        pub fn TCOD_sys_get_last_frame_length() -> c_float;
        pub fn TCOD_console_init_root(w: c_int, h: c_int, title: *c_char,
                                  fullscreen: c_bool, renderer: renderer_t);
        pub fn TCOD_console_set_custom_font(fontFile: *c_char, flags: c_int,
                                        nb_char_horiz: c_int, nb_char_vertic: c_int);
        pub fn TCOD_console_set_fade(fade: u8, fading_color: Color);
        pub fn TCOD_console_is_window_closed() -> c_bool;
        pub fn TCOD_console_wait_for_keypress(flush: c_bool) -> Key;
        pub fn TCOD_console_check_for_keypress(pressed: c_int) -> Key;
        pub fn TCOD_console_set_char_background(con: console_t, x: c_int, y: c_int,
                                            col: Color,
                                            flag: BackgroundFlag);
        pub fn TCOD_console_set_char_foreground(con: console_t, x: c_int, y: c_int,
                                            col: Color);
        pub fn TCOD_console_put_char(con: console_t, x: c_int, y: c_int, c: c_int,
                                 flag: BackgroundFlag);
        pub fn TCOD_console_put_char_ex(con: console_t, x: c_int, y: c_int, c: c_int,
                                    fore: Color, back: Color) -> ();
        pub fn TCOD_console_clear(con: console_t);
        pub fn TCOD_console_flush() -> ();
        pub fn TCOD_console_print_ex(con: console_t, x: c_int, y: c_int,
                                 flag: BackgroundFlag,
                                 alignment: TextAlignment,
                                 fmt: *c_char) -> ();
        pub fn TCOD_console_new(w: c_int, h: c_int) -> console_t;
        pub fn TCOD_console_get_width(con: console_t) -> c_int;
        pub fn TCOD_console_get_height(con: console_t) -> c_int;
        pub fn TCOD_console_set_default_background(con: console_t, col: Color);
        pub fn TCOD_console_set_default_foreground(con: console_t, col: Color);
        pub fn TCOD_console_set_key_color(con: console_t, col: Color);
        pub fn TCOD_console_blit(src: console_t, xSrc: c_int, ySrc: c_int,
                             wSrc: c_int, hSrc: c_int,
                             dst: console_t, xDst: c_int, yDst: c_int,
                             foreground_alpha: c_float, background_alpha: c_float);
        pub fn TCOD_console_delete(con: console_t);
        pub fn TCOD_map_new(width: c_int, height: c_int) -> map_t;
        pub fn TCOD_map_set_properties(map: map_t, x: c_int, y: c_int,
                                   is_transparent: c_bool, is_walkable: c_bool);
        pub fn TCOD_map_is_walkable(map: map_t, x: c_int, y: c_int) -> c_bool;
        pub fn TCOD_map_get_width(map: map_t) -> c_int;
        pub fn TCOD_map_get_height(map: map_t) -> c_int;
        pub fn TCOD_map_clear(map: map_t, transparent: c_bool, walkable: c_bool);
        pub fn TCOD_map_delete(map: map_t);
        pub fn TCOD_path_new_using_map(map: map_t, diagonalCost: c_float)
                                   -> path_t;
        pub fn TCOD_path_new_using_function(map_width: c_int, map_height: c_int,
                                        func: path_callback_t,
                                        user_data: *c_void,
                                        diagonalCost: c_float) -> path_t;
        pub fn TCOD_path_compute(path: path_t, ox: c_int, oy: c_int,
                             dx: c_int, dy: c_int) -> c_bool;
        pub fn TCOD_path_walk(path: path_t, x: *mut c_int, y: *mut c_int,
                          recalculate_when_needed: c_bool) -> c_bool;
        pub fn TCOD_path_is_empty(path: path_t) -> c_bool;
        pub fn TCOD_path_size(path: path_t) -> c_int;
        pub fn TCOD_path_get_destination(path: path_t,
                                     x: *mut c_int, y: *mut c_int);
        pub fn TCOD_path_delete(path: path_t);
    }
}


pub static ROOT_CONSOLE: console_t = 0 as console_t;


pub fn sys_set_fps(fps: int) {
    assert!(fps > 0);
    unsafe {
        ffi::TCOD_sys_set_fps(fps as c_int)
    }
}

pub fn sys_get_fps() -> int {
    let mut result;
    unsafe {
        result = ffi::TCOD_sys_get_fps();
    }
    assert!(result >= 0);
    return result as int
}

pub fn sys_get_last_frame_length() -> f32 {
    unsafe {
        ffi::TCOD_sys_get_last_frame_length() as f32
    }
}

pub fn console_init_root(width: int, height: int, title: &str, fullscreen: bool) {
    assert!(width > 0 && height > 0);
    unsafe {
        title.with_c_str(
            |c_title| ffi::TCOD_console_init_root(width as c_int, height as c_int,
                                                  c_title, fullscreen as c_bool,
                                                  ffi::RENDERER_SDL));
    }
}

pub fn console_set_fade(fade: u8, fading_color: Color) {
    unsafe {
        ffi::TCOD_console_set_fade(fade, fading_color);
    }
}

pub fn console_set_custom_font(font_path: path::Path) {
    unsafe {
        let flags = (ffi::FONT_TYPE_GREYSCALE as c_int |
                     ffi::FONT_LAYOUT_TCOD as c_int);
        font_path.with_c_str( |path| {
            ffi::TCOD_console_set_custom_font(path, flags, 32, 8);
        });
    }
}

pub fn console_is_window_closed() -> bool {
    unsafe {
        ffi::TCOD_console_is_window_closed() != 0
    }
}

pub enum KeyStatus {
    KeyPressed = 1,
    KeyReleased = 2,
    KeyPressedOrReleased = 1 | 2,
}

pub fn console_wait_for_keypress(flush: bool) -> Key {
    unsafe {
        ffi::TCOD_console_wait_for_keypress(flush as c_bool)
    }
}

pub fn console_check_for_keypress(status: KeyStatus) -> Key {
    unsafe {
        ffi::TCOD_console_check_for_keypress(status as c_int)
    }
}

pub fn console_set_char_background(con: console_t, x: int, y: int,
                                   color: Color,
                                   background_flag: ffi::BackgroundFlag) {
    assert!(x >= 0 && y >= 0);
    unsafe {
        ffi::TCOD_console_set_char_background(con, x as c_int, y as c_int,
                                              color, background_flag)
    }
}

pub fn console_put_char(con: console_t, x: int, y: int, glyph: char,
                        background_flag: ffi::BackgroundFlag) {
    assert!(x >= 0 && y >= 0);
    unsafe {
        ffi::TCOD_console_put_char(con, x as c_int, y as c_int, glyph as c_int,
                                   background_flag);
    }
}

pub fn console_put_char_ex(con: console_t, x: int, y: int, glyph: char,
                           foreground: Color, background: Color) {
    assert!(x >= 0 && y >= 0);
    unsafe {
        ffi::TCOD_console_put_char_ex(con, x as c_int, y as c_int, glyph as c_int,
                                      foreground, background);
    }
}

pub fn console_clear(con: console_t) {
    unsafe {
        ffi::TCOD_console_clear(con);
    }
}

pub fn console_flush() {
    unsafe {
        ffi::TCOD_console_flush();
    }
}

pub fn console_print_ex(con: console_t, x: int, y: int,
                        background_flag: ffi::BackgroundFlag,
                        alignment: ffi::TextAlignment,
                        text: &str) {
    assert!(x >= 0 && y >= 0);
    unsafe {
        text.with_c_str(
            |c_text|
                ffi::TCOD_console_print_ex(con, x as c_int, y as c_int,
                                           background_flag, alignment, c_text));
    }
}

pub fn console_new(width: int, height: int) -> console_t {
    assert!(width > 0 && height > 0);
    unsafe {
        ffi::TCOD_console_new(width as c_int, height as c_int)
    }
}

pub fn console_get_width(con: console_t) -> int {
    unsafe {
        ffi::TCOD_console_get_width(con) as int
    }
}

pub fn console_get_height(con: console_t) -> int {
    unsafe {
        ffi::TCOD_console_get_height(con) as int
    }
}

pub fn console_set_default_background(con: console_t, color: Color) {
    unsafe {
        ffi::TCOD_console_set_default_background(con, color);
    }
}

pub fn console_set_default_foreground(con: console_t, color: Color) {
    unsafe {
        ffi::TCOD_console_set_default_foreground(con, color);
    }
}

pub fn console_set_key_color(con: console_t, color: Color) {
    unsafe {
        ffi::TCOD_console_set_key_color(con, color);
    }
}

pub fn console_blit(source_console: console_t,
                    source_x: int, source_y: int,
                    source_width: int, source_height: int,
                    destination_console: console_t,
                    destination_x: int, destination_y: int,
                    foreground_alpha: f32, background_alpha: f32) {
    assert!(source_x >= 0 && source_y >= 0 &&
            source_width > 0 && source_height > 0 &&
            destination_x >= 0 && destination_y >= 0);
    unsafe {
        ffi::TCOD_console_blit(source_console, source_x as c_int, source_y as c_int,
                               source_width as c_int, source_height as c_int,
                               destination_console,
                               destination_x as c_int, destination_y as c_int,
                               foreground_alpha as c_float,
                               background_alpha as c_float);
    }
}

pub fn console_delete(con: console_t) {
    unsafe {
        ffi::TCOD_console_delete(con);
    }
}
