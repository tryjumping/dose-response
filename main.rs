mod tcod;

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

fn draw(layers: &[tcod::TCOD_console_t], world: &~[(uint, uint, char)], width: uint, height: uint) {
    let con = layers[layers.len() - 1];
    for world.iter().advance |&(x, y, glyph)| {
        tcod::console_set_char_background(con, x, y,
                                          tcod::TCOD_color_t{r: 30, g: 30, b: 30},
                                          tcod::TCOD_BKGND_SET);
        tcod::console_put_char(con, x, y, glyph, tcod::TCOD_BKGND_DEFAULT);
    }
    tcod::console_print_ex(con, width - 1, height-1,
                           tcod::TCOD_BKGND_NONE, tcod::TCOD_RIGHT,
                           fmt!("FPS: %?", tcod::sys_get_fps()));
}


fn main() {
    let width = 80;
    let height = 50;
    let console_count = 10;
    let transparent_bg = tcod::TCOD_color_t{r: 255, g: 0, b: 0};
    let white = tcod::TCOD_color_t{r: 255, g: 255, b: 255};

    let world = generate_world(width, height);
    let mut consoles: ~[tcod::TCOD_console_t] = ~[];
    for 3.times {
        let con = tcod::console_new(width, height);
        tcod::console_set_key_color(con, transparent_bg);
        consoles.push(con);
    }
    tcod::console_set_custom_font("./fonts/dejavu16x16_gs_tc.png");

    tcod::console_init_root(width, height, "Dose Response", false);
    while !tcod::console_is_window_closed() {
        let key = tcod::console_check_for_keypress(tcod::KeyPressedOrReleased);
        if key.c == 27 { break; }
        tcod::console_set_default_foreground(tcod::ROOT_CONSOLE, white);
        tcod::console_clear(tcod::ROOT_CONSOLE);
        for consoles.iter().advance |&con| {
            tcod::console_set_default_background(con, transparent_bg);
            tcod::console_set_default_foreground(con, white);
            tcod::console_clear(con);
        }

        draw(consoles, &world, width, height);

        for consoles.iter().advance |&con| {
            tcod::console_blit(con, 0, 0, width, height,
                               tcod::ROOT_CONSOLE, 0, 0,
                               1f, 1f);
        }
        tcod::console_flush();
    }


    println(fmt!("width: %?, height: %?, consoles: %?", width, height, console_count));
}
