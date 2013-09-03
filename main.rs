extern mod rtcod;

fn generate_world(w: uint, h: uint) -> ~[(uint, uint, char)] {
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result: ~[(uint, uint, char)] = ~[];
    for std::uint::range(0, w) |x| {
        for std::uint::range(0, h) |y| {
            result.push((x, y, chars[(x * y) % chars.char_len()] as char));
        }
    }
    return result;
}

fn draw(layers: &[rtcod::TCOD_console_t], world: &~[(uint, uint, char)], width: uint, height: uint) {
    let con = layers[layers.len() - 1];
    for world.iter().advance |&(x, y, glyph)| {
        rtcod::console_set_char_background(con, x, y,
                                          rtcod::TCOD_color_t{r: 30, g: 30, b: 30},
                                          rtcod::TCOD_BKGND_SET);
        rtcod::console_put_char(con, x, y, glyph, rtcod::TCOD_BKGND_DEFAULT);
    }
    rtcod::console_print_ex(con, width - 1, height-1,
                           rtcod::TCOD_BKGND_NONE, rtcod::TCOD_RIGHT,
                           fmt!("FPS: %?", rtcod::sys_get_fps()));
}


fn main() {
    let width = 80;
    let height = 50;
    let console_count = 10;
    let transparent_bg = rtcod::TCOD_color_t{r: 255, g: 0, b: 0};
    let white = rtcod::TCOD_color_t{r: 255, g: 255, b: 255};

    let world = generate_world(width, height);
    let mut consoles: ~[rtcod::TCOD_console_t] = ~[];
    for 3.times {
        let con = rtcod::console_new(width, height);
        rtcod::console_set_key_color(con, transparent_bg);
        consoles.push(con);
    }
    rtcod::console_set_custom_font("./fonts/dejavu16x16_gs_tc.png");

    rtcod::console_init_root(width, height, "Dose Response", false);
    while !rtcod::console_is_window_closed() {
        let key = rtcod::console_check_for_keypress(rtcod::KeyPressedOrReleased);
        if key.c == 27 { break; }
        rtcod::console_set_default_foreground(rtcod::ROOT_CONSOLE, white);
        rtcod::console_clear(rtcod::ROOT_CONSOLE);
        for consoles.iter().advance |&con| {
            rtcod::console_set_default_background(con, transparent_bg);
            rtcod::console_set_default_foreground(con, white);
            rtcod::console_clear(con);
        }

        draw(consoles, &world, width, height);

        for consoles.iter().advance |&con| {
            rtcod::console_blit(con, 0, 0, width, height,
                               rtcod::ROOT_CONSOLE, 0, 0,
                               1f, 1f);
        }
        rtcod::console_flush();
    }


    println(fmt!("width: %?, height: %?, consoles: %?", width, height, console_count));
}
