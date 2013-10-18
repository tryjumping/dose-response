use std::libc::{c_int, c_char, uint8_t, c_void, c_float};
use std::cast;

pub type TCOD_console_t = *c_void;

pub type TCOD_map_t = *c_void;

pub type TCOD_path_t = *c_void;

pub type TCOD_path_callback_t = extern "C" fn(xf: c_int, _yf: c_int, _xt: c_int, _yt: c_int, ud: *c_void) -> c_float;

enum TCOD_renderer_t {
    TCOD_RENDERER_GLSL,
    TCOD_RENDERER_OPENGL,
    TCOD_RENDERER_SDL,
    TCOD_NB_RENDERERS,
}

enum TCOD_key_status_t {
    TCOD_KEY_PRESSED=1,
    TCOD_KEY_RELEASED=2,
}

enum TCOD_font_flags_t {
    TCOD_FONT_LAYOUT_ASCII_INCOL=1,
    TCOD_FONT_LAYOUT_ASCII_INROW=2,
    TCOD_FONT_TYPE_GREYSCALE=4,
    TCOD_FONT_LAYOUT_TCOD=8,
}

pub struct TCOD_key_t {
    vk: c_int,
    c: c_char,
    pressed: uint8_t,
    lalt: uint8_t,
    lctrl: uint8_t,
    ralt: uint8_t,
    rctrl: uint8_t,
    shift: uint8_t,
}

pub struct TCOD_color_t {
    r: uint8_t,
    g: uint8_t,
    b: uint8_t,
}

pub enum TCOD_alignment_t {
    TCOD_LEFT,
    TCOD_RIGHT,
    TCOD_CENTER
}

pub enum TCOD_bkgnd_flag_t {
    TCOD_BKGND_NONE,
    TCOD_BKGND_SET,
    TCOD_BKGND_MULTIPLY,
    TCOD_BKGND_LIGHTEN,
    TCOD_BKGND_DARKEN,
    TCOD_BKGND_SCREEN,
    TCOD_BKGND_COLOR_DODGE,
    TCOD_BKGND_COLOR_BURN,
    TCOD_BKGND_ADD,
    TCOD_BKGND_ADDA,
    TCOD_BKGND_BURN,
    TCOD_BKGND_OVERLAY,
    TCOD_BKGND_ALPH,
    TCOD_BKGND_DEFAULT
}


#[link_args = "-ltcod"]
extern "C" {
    fn TCOD_sys_set_fps(val: c_int) -> ();
    fn TCOD_sys_get_fps() -> c_int;
    fn TCOD_console_init_root(w: c_int, h: c_int, title: *c_char,
                              fullscreen: uint8_t, renderer: TCOD_renderer_t);
    fn TCOD_console_set_custom_font(fontFile: *c_char, flags: c_int,
                                    nb_char_horiz: c_int, nb_char_vertic: c_int) -> ();
    fn TCOD_console_is_window_closed() -> uint8_t;
    fn TCOD_console_wait_for_keypress(flush: uint8_t) -> TCOD_key_t;
    fn TCOD_console_check_for_keypress(pressed: c_int) -> TCOD_key_t;
    fn TCOD_console_set_char_background(con: TCOD_console_t, x: c_int, y: c_int,
                                        col: TCOD_color_t,
                                        flag: TCOD_bkgnd_flag_t) -> ();
    fn TCOD_console_set_char_foreground(con: TCOD_console_t, x: c_int, y: c_int, col: TCOD_color_t) -> ();
    fn TCOD_console_put_char(con: TCOD_console_t, x: c_int, y: c_int, c: c_int,
                             flag: TCOD_bkgnd_flag_t) -> ();
    fn TCOD_console_put_char_ex(con: TCOD_console_t, x: c_int, y: c_int, c: c_int,
                                fore: TCOD_color_t, back: TCOD_color_t) -> ();
    fn TCOD_console_clear(con: TCOD_console_t) -> ();
    fn TCOD_console_flush() -> ();
    fn TCOD_console_print_ex(con: TCOD_console_t, x: c_int, y: c_int,
                             flag: TCOD_bkgnd_flag_t,
                             alignment: TCOD_alignment_t,
                             fmt: *c_char) -> ();
    fn TCOD_console_new(w: c_int, h: c_int) -> TCOD_console_t;
    fn TCOD_console_get_width(con: TCOD_console_t) -> c_int;
    fn TCOD_console_get_height(con: TCOD_console_t) -> c_int;
    fn TCOD_console_set_default_background(con: TCOD_console_t, col: TCOD_color_t) -> ();
    fn TCOD_console_set_default_foreground(con: TCOD_console_t, col: TCOD_color_t) -> ();
    fn TCOD_console_set_key_color(con: TCOD_console_t, col: TCOD_color_t) -> ();
    fn TCOD_console_blit(src: TCOD_console_t, xSrc: c_int, ySrc: c_int, wSrc: c_int, hSrc: c_int,
                         dst: TCOD_console_t, xDst: c_int, yDst: c_int,
                         foreground_alpha: c_float, background_alpha: c_float) -> ();
    fn TCOD_console_delete(con: TCOD_console_t);
    fn TCOD_map_new(width: c_int, height: c_int) -> TCOD_map_t;
    fn TCOD_map_set_properties(map: TCOD_map_t, x: c_int, y: c_int,
                               is_transparent: uint8_t, is_walkable: uint8_t);
    fn TCOD_map_is_walkable(map: TCOD_map_t, x: c_int, y: c_int) -> uint8_t;
    fn TCOD_map_get_width(map: TCOD_map_t) -> c_int;
    fn TCOD_map_get_height(map: TCOD_map_t) -> c_int;
    fn TCOD_map_clear(map: TCOD_map_t, transparent: uint8_t, walkable: uint8_t);

    fn TCOD_path_new_using_map(map: TCOD_map_t, diagonalCost: c_float) -> TCOD_path_t;
    fn TCOD_path_new_using_function(map_width: c_int, map_height: c_int,
                                    func: TCOD_path_callback_t,
                                    user_data: *c_void,
                                    diagonalCost: c_float) -> TCOD_path_t;
    fn TCOD_path_compute(path: TCOD_path_t, ox: c_int, oy: c_int,
                         dx: c_int, dy: c_int) -> uint8_t;
    fn TCOD_path_walk(path: TCOD_path_t, x: *mut c_int, y: *mut c_int,
                      recalculate_when_needed: uint8_t) -> uint8_t;
    fn TCOD_path_is_empty(path: TCOD_path_t) -> uint8_t;
    fn TCOD_path_size(path: TCOD_path_t) -> c_int;
    fn TCOD_path_get_destination(path: TCOD_path_t, x: *mut c_int, y: *mut c_int);
    fn TCOD_path_delete(path: TCOD_path_t);
}

// let's make sure casting to c_int doesn't overflow
static max_uint: uint = 10000;

pub static ROOT_CONSOLE: TCOD_console_t = 0 as TCOD_console_t;

#[fixed_stack_segment]
pub fn sys_set_fps(fps: uint) {
    assert!(fps < max_uint);
    unsafe {
        TCOD_sys_set_fps(fps as c_int)
    }
}

#[fixed_stack_segment]
pub fn sys_get_fps() -> uint {
    let mut result;
    unsafe {
        result = TCOD_sys_get_fps();
    }
    assert!(result >= 0);
    return result as uint
}

#[fixed_stack_segment]
pub fn console_init_root(width: uint, height: uint, title: &str,
                         fullscreen: bool) {
    assert!(width < max_uint); assert!(height < max_uint);
    unsafe {
        title.with_c_str(
            |c_title| TCOD_console_init_root(width as c_int, height as c_int,
                                             c_title, fullscreen as uint8_t,
                                             TCOD_RENDERER_SDL));
    }
}

#[fixed_stack_segment]
pub fn console_set_custom_font(font_path: Path) {
    unsafe {
        font_path.to_str().with_c_str(
            |path| TCOD_console_set_custom_font(path, TCOD_FONT_TYPE_GREYSCALE as c_int | TCOD_FONT_LAYOUT_TCOD as c_int, 32, 8));
    }
}

#[fixed_stack_segment]
pub fn console_is_window_closed() -> bool {
    unsafe {
        TCOD_console_is_window_closed() != 0
    }
}

pub enum KeyStatus {
    KeyPressed = 1,
    KeyReleased = 2,
    KeyPressedOrReleased = 1 | 2,
}

#[fixed_stack_segment]
pub fn console_wait_for_keypress(flush: bool) -> TCOD_key_t {
    unsafe {
        TCOD_console_wait_for_keypress(flush as uint8_t)
    }
}

#[fixed_stack_segment]
pub fn console_check_for_keypress(status: KeyStatus) -> TCOD_key_t {
    unsafe {
        TCOD_console_check_for_keypress(status as c_int)
    }
}

#[fixed_stack_segment]
pub fn console_set_char_background(con: TCOD_console_t, x: uint, y: uint,
                                   color: TCOD_color_t,
                                   background_flag: TCOD_bkgnd_flag_t) {
    assert!(x < max_uint); assert!(y < max_uint);
    unsafe {
        TCOD_console_set_char_background(con, x as c_int, y as c_int,
                                         color, background_flag)
    }
}

#[fixed_stack_segment]
pub fn console_put_char(con: TCOD_console_t, x: uint, y: uint, glyph: char,
                        background_flag: TCOD_bkgnd_flag_t) {
    assert!(x < max_uint); assert!(y < max_uint);
    unsafe {
        TCOD_console_put_char(con, x as c_int, y as c_int, glyph as c_int,
                              background_flag);
    }
}

#[fixed_stack_segment]
pub fn console_put_char_ex(con: TCOD_console_t, x: uint, y: uint, glyph: char,
                           foreground: TCOD_color_t, background: TCOD_color_t) {
    assert!(x < max_uint); assert!(y < max_uint);
    unsafe {
        TCOD_console_put_char_ex(con, x as c_int, y as c_int, glyph as c_int,
                                 foreground, background);
    }
}

#[fixed_stack_segment]
pub fn console_clear(con: TCOD_console_t) {
    unsafe {
        TCOD_console_clear(con);
    }
}

#[fixed_stack_segment]
pub fn console_flush() {
    unsafe {
        TCOD_console_flush();
    }
}

#[fixed_stack_segment]
pub fn console_print_ex(con: TCOD_console_t, x: uint, y: uint,
                        background_flag: TCOD_bkgnd_flag_t,
                        alignment: TCOD_alignment_t,
                        text: &str) {
    assert!(x < max_uint); assert!(y < max_uint);
    unsafe {
        text.with_c_str(
            |c_text|
                TCOD_console_print_ex(con, x as c_int, y as c_int, background_flag, alignment, c_text));
    }
}

#[fixed_stack_segment]
pub fn console_new(width: uint, height: uint) -> TCOD_console_t {
    assert!(width < max_uint); assert!(height < max_uint);
    unsafe {
        TCOD_console_new(width as c_int, height as c_int)
    }
}

#[fixed_stack_segment]
pub fn console_get_width(con: TCOD_console_t) -> int {
    unsafe {
        TCOD_console_get_width(con) as int
    }
}

#[fixed_stack_segment]
pub fn console_get_height(con: TCOD_console_t) -> int {
    unsafe {
        TCOD_console_get_height(con) as int
    }
}

#[fixed_stack_segment]
pub fn console_set_default_background(con: TCOD_console_t, color: TCOD_color_t) {
    unsafe {
        TCOD_console_set_default_background(con, color);
    }
}

#[fixed_stack_segment]
pub fn console_set_default_foreground(con: TCOD_console_t, color: TCOD_color_t) {
    unsafe {
        TCOD_console_set_default_foreground(con, color);
    }
}

#[fixed_stack_segment]
pub fn console_set_key_color(con: TCOD_console_t, color: TCOD_color_t) {
    unsafe {
        TCOD_console_set_key_color(con, color);
    }
}

#[fixed_stack_segment]
pub fn console_blit(source_console: TCOD_console_t,
                    source_x: uint, source_y: uint,
                    source_width: uint, source_height: uint,
                    destination_console: TCOD_console_t,
                    destination_x: uint, destination_y: uint,
                    foreground_alpha: float, background_alpha: float) {
    assert!(source_x < max_uint && source_y < max_uint &&
            source_width < max_uint && source_height < max_uint &&
            destination_x < max_uint && destination_y < max_uint);
    unsafe {
        TCOD_console_blit(source_console, source_x as c_int, source_y as c_int,
                          source_width as c_int, source_height as c_int,
                          destination_console,
                          destination_x as c_int, destination_y as c_int,
                          foreground_alpha as c_float, background_alpha as c_float);
    }
}

#[fixed_stack_segment]
pub fn console_delete(con: TCOD_console_t) {
    unsafe {
        TCOD_console_delete(con);
    }
}

#[fixed_stack_segment]
pub fn map_new(width: uint, height: uint) -> TCOD_map_t {
    assert!(width < max_uint && height < max_uint);
    unsafe {
        TCOD_map_new(width as c_int, height as c_int)
    }
}

#[fixed_stack_segment]
pub fn map_set_properties(map: TCOD_map_t, x: uint, y: uint,
                          transparent: bool, walkable: bool) {
    assert!(x < max_uint && y < max_uint);
    unsafe {
        TCOD_map_set_properties(map, x as c_int, y as c_int,
                                transparent as uint8_t, walkable as uint8_t);
    }
}

#[fixed_stack_segment]
pub fn map_is_walkable(map: TCOD_map_t, x: int, y: int) -> bool {
    assert!(x >= 0 && y >= 0);
    unsafe {
        TCOD_map_is_walkable(map, x as c_int, y as c_int) != 0
    }
}

#[fixed_stack_segment]
pub fn map_size(map: TCOD_map_t) -> (uint, uint) {
    unsafe {
        let (w, h) = (TCOD_map_get_width(map), TCOD_map_get_height(map));
        assert!(w >= 0 && h >= 0);
        (w as uint, h as uint)
    }
}

#[fixed_stack_segment]
pub fn map_clear(map: TCOD_map_t, transparent: bool, walkable: bool) {
    unsafe {
        TCOD_map_clear(map, transparent as uint8_t, walkable as uint8_t);
    }
}

#[fixed_stack_segment]
pub fn path_new_using_map(map: TCOD_map_t, diagonal_cost: float) -> TCOD_path_t {
    unsafe {
        TCOD_path_new_using_map(map, diagonal_cost as c_float)
    }
}

#[fixed_stack_segment]
pub fn path_new_using_function<T>(map_width: int, map_height: int,
                               path_cb: TCOD_path_callback_t,
                               user_data: &T,
                               diagonal_cost: float) -> TCOD_path_t {
    assert!(map_width >= 0 && map_height >= 0);
    unsafe {
        TCOD_path_new_using_function(map_width as c_int, map_height as c_int,
                                     path_cb,
                                     cast::transmute(user_data),
                                     diagonal_cost as c_float)
    }
}

#[fixed_stack_segment]
pub fn path_compute(path: TCOD_path_t, ox: int, oy: int,
                    dx: int, dy: int) -> bool {
    assert!(ox >= 0 && oy >= 0 && dx >= 0 && dy >= 0);
    unsafe {
        TCOD_path_compute(path, ox as c_int, oy as c_int,
                          dx as c_int, dy as c_int) != 0
    }
}

#[fixed_stack_segment]
pub fn path_walk(path: TCOD_path_t, recalculate_when_needed: bool) -> Option<(int, int)> {
    unsafe {
        let mut x: c_int = 0;
        let mut y: c_int = 0;
        match TCOD_path_walk(path, &mut x, &mut y,
                             recalculate_when_needed as uint8_t) != 0 {
            true => Some((x as int, y as int)),
            false => None,
        }
    }
}

#[fixed_stack_segment]
pub fn path_is_empty(path: TCOD_path_t) -> bool {
    unsafe {
        TCOD_path_is_empty(path) != 0
    }
}

#[fixed_stack_segment]
pub fn path_size(path: TCOD_path_t) -> int {
    unsafe {
        TCOD_path_size(path) as int
    }
}

#[fixed_stack_segment]
pub fn path_get_destination(path: TCOD_path_t) -> (int, int) {
    unsafe {
        let mut x: c_int = 0;
        let mut y: c_int = 0;
        TCOD_path_get_destination(path, &mut x, &mut y);
        (x as int, y as int)
    }
}

#[fixed_stack_segment]
pub fn path_delete(path: TCOD_path_t) {
    unsafe {
        TCOD_path_delete(path);
    }
}
