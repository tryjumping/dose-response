use tcod;


pub fn draw(layers: &[tcod::TCOD_console_t], world: &~[(uint, uint, char)], width: uint, height: uint) {
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
