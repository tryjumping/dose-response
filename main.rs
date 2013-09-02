use std::libc::{c_int, c_char, uint8_t, c_void, c_float};

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

struct TCOD_color_t {
    r: uint8_t,
    g: uint8_t,
    b: uint8_t,
}

enum TCOD_alignment_t {
        TCOD_LEFT,
        TCOD_RIGHT,
        TCOD_CENTER
}


enum TCOD_bkgnd_flag_t {
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
extern {
    fn TCOD_sys_set_fps(val: c_int) -> ();
    fn TCOD_sys_get_fps() -> c_int;
    fn TCOD_console_init_root(w: c_int, h: c_int, title: *c_char,
                              fullscreen: uint8_t, renderer: TCOD_renderer_t);
    fn TCOD_console_set_custom_font(fontFile: *c_char, flags: c_int,
                                    nb_char_horiz: c_int, nb_char_vertic: c_int) -> ();
    fn TCOD_console_is_window_closed() -> uint8_t;
    fn TCOD_console_wait_for_keypress(flush: uint8_t) -> TCOD_key_t;
    fn TCOD_console_check_for_keypress(pressed: c_int) -> TCOD_key_t;
    fn TCOD_console_set_char_background(con: *c_void, x: c_int, y: c_int,
                                        col: TCOD_color_t,
                                        flag: TCOD_bkgnd_flag_t) -> ();
    fn TCOD_console_set_char_foreground(con: *c_void, x: c_int, y: c_int, col: TCOD_color_t) -> ();
    fn TCOD_console_put_char(con: *c_void, x: c_int, y: c_int, c: c_int,
                             flag: TCOD_bkgnd_flag_t) -> ();
    fn TCOD_console_put_char_ex(con: *c_void, x: c_int, y: c_int, c: c_int,
                                fore: TCOD_color_t, back: TCOD_color_t) -> ();
    fn TCOD_console_clear(con: *c_void) -> ();
    fn TCOD_console_flush() -> ();
    fn TCOD_console_print_ex(con: *c_void, x: c_int, y: c_int,
                             flag: TCOD_bkgnd_flag_t,
                             alignment: TCOD_alignment_t,
                             fmt: *c_char) -> ();
    fn TCOD_console_new(w: c_int, h: c_int) -> *c_void;
    fn TCOD_console_set_default_background(con: *c_void, col: TCOD_color_t) -> ();
    fn TCOD_console_set_default_foreground(con: *c_void, col: TCOD_color_t) -> ();
    fn TCOD_console_set_key_color(con: *c_void, col: TCOD_color_t) -> ();
    fn TCOD_console_blit(src: *c_void, xSrc: c_int, ySrc: c_int, wSrc: c_int, hSrc: c_int,
                         dst: *c_void, xDst: c_int, yDst: c_int,
                         foreground_alpha: c_float, background_alpha: c_float) -> ();
}

fn generate_world(w: uint, h: uint) -> ~[(uint, uint, u8)] {
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result: ~[(uint, uint, u8)] = ~[];
    for std::uint::range(0, w) |x| {
        for std::uint::range(0, h) |y| {
            result.push((x, y, chars[(x * y) % chars.char_len()]));
        }
    }
    return result;
}

unsafe fn draw(layers: &[*c_void], world: &~[(uint, uint, u8)], width: uint, height: uint) {
    let con = layers[layers.len() - 1];
    for world.iter().advance |&(x, y, glyph)| {
        TCOD_console_set_char_background(con, x as c_int, y as c_int,
                                         TCOD_color_t{r: 30, g: 30, b: 30},
                                         TCOD_BKGND_SET);
        TCOD_console_put_char(con, x as c_int, y as c_int, glyph as c_int, TCOD_BKGND_DEFAULT);
    }
    (fmt!("FPS: %?", TCOD_sys_get_fps())).as_c_str(
        |text| TCOD_console_print_ex(con, (width-1) as c_int, (height-1) as c_int,
                                     TCOD_BKGND_NONE, TCOD_RIGHT,
                                     text));
}

fn main() {
    let width = 80;
    let height = 50;
    let console_count = 10;
    let transparent_bg = TCOD_color_t{r: 255, g: 0, b: 0};
    let white = TCOD_color_t{r: 255, g: 255, b: 255};

    let world = generate_world(width, height);
    let root_console = 0 as *c_void;
    let mut consoles: ~[*c_void] = ~[];
    unsafe {
        for 3.times {
            let con = TCOD_console_new(width as c_int, height as c_int);
            TCOD_console_set_key_color(con, transparent_bg);
            consoles.push(con);
        }
        "./fonts/dejavu16x16_gs_tc.png".as_c_str(
            |font_path| TCOD_console_set_custom_font(font_path, TCOD_FONT_TYPE_GREYSCALE as c_int | TCOD_FONT_LAYOUT_TCOD as c_int, 32, 8));
        "Dose Response".as_c_str(|title| TCOD_console_init_root(width as c_int, height as c_int, title, 0, TCOD_RENDERER_SDL));
        while TCOD_console_is_window_closed() == 0 {
            let key = TCOD_console_check_for_keypress(TCOD_KEY_PRESSED as c_int | TCOD_KEY_RELEASED as c_int);
            if key.c == 27 { break; }
            TCOD_console_set_default_foreground(root_console, white);
            TCOD_console_clear(root_console);
            for consoles.iter().advance |&con| {
                TCOD_console_set_default_background(con, transparent_bg);
                TCOD_console_set_default_foreground(con, white);
                TCOD_console_clear(con);
            }

            draw(consoles, &world, width, height);

            for consoles.iter().advance |&con| {
                TCOD_console_blit(con,
                                  0, 0, width as c_int, height as c_int,
                                  root_console, 0, 0,
                                  1 as c_float, 1 as c_float);
            }
            TCOD_console_flush();
        }
    }



    println(fmt!("width: %?, height: %?, consoles: %?", width, height, console_count));
}
