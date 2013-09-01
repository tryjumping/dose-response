use std::libc::{c_int, c_char, uint8_t};

struct TCOD_key_t {
    vk: c_int,
    c: c_char,
    pressed: uint8_t,
    lalt: uint8_t,
    lctrl: uint8_t,
    ralt: uint8_t,
    rctrl: uint8_t,
    shift: uint8_t,
}

#[link_args = "-ltcod"]
extern {
    fn TCOD_sys_set_fps(val: c_int) -> ();
    fn TCOD_sys_get_fps() -> c_int;
    fn TCOD_console_init_root(w: c_int, h: c_int, title: *c_char,
                              fullscreen: uint8_t, renderer: c_int);
    fn TCOD_console_set_custom_font(fontFile: *c_char, flags: c_int,
                                    nb_char_horiz: c_int, nb_char_vertic: c_int) -> ();
    fn TCOD_console_is_window_closed() -> uint8_t;
    fn TCOD_console_wait_for_keypress(flush: uint8_t) -> TCOD_key_t;
    fn TCOD_console_check_for_keypress(pressed: c_int) -> TCOD_key_t;
    fn TCOD_console_flush() -> ();
}

fn generate_world(w: uint, h: uint) -> ~[(uint, uint, u8)] {
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result: ~[(uint, uint, u8)] = ~[];
    for std::uint::range(0, w) |x| {
        for std::uint::range(0, h) |y| {
            result.push((x, y, chars[(x + y) % chars.char_len()]));
        }
    }
    return result;
}

fn main() {
    let width = 80;
    let height = 50;
    let console_count = 10;

    generate_world(width, height);
    let TCOD_FONT_TYPE_GREYSCALE = 4;
    let TCOD_FONT_LAYOUT_TCOD = 8;
    let TCOD_KEY_PRESSED = 1;
    let TCOD_KEY_RELEASED = 2;
    unsafe {
        "./fonts/dejavu16x16_gs_tc.png".as_c_str(
            |font_path| TCOD_console_set_custom_font(font_path, TCOD_FONT_TYPE_GREYSCALE | TCOD_FONT_LAYOUT_TCOD, 32, 8));
        "tcod bench".as_c_str(|title| TCOD_console_init_root(width as c_int, height as c_int, title, 0, 2));
        while TCOD_console_is_window_closed() == 0 {
            let key = TCOD_console_check_for_keypress(TCOD_KEY_PRESSED | TCOD_KEY_RELEASED);
            if key.c == 27 { break; }
            TCOD_console_flush();
        }
    }



    println(fmt!("width: %?, height: %?, consoles: %?", width, height, console_count));
}