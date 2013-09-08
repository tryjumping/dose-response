mod tcod;


pub enum MainLoopState {
    Running,
    Exit,
}

pub fn main_loop<T>(width: uint, height: uint, title: &str,
                    font_path: &str,
                    initial_state: &fn(uint, uint) -> ~T,
                    update: &fn(&mut T) -> MainLoopState) {
    let fullscreen = false;
    let transparent_bg = tcod::TCOD_color_t{r: 255, g: 0, b: 0};
    let default_fg = tcod::TCOD_color_t{r: 255, g: 255, b: 255};
    let console_count = 3;
    let mut consoles: ~[tcod::TCOD_console_t] = ~[];
    for console_count.times {
        let con = tcod::console_new(width, height);
        tcod::console_set_key_color(con, transparent_bg);
        consoles.push(con);
    }
    tcod::console_set_custom_font(font_path);
    tcod::console_init_root(width, height, title, fullscreen);

    let mut game_state = initial_state(width, height);
    while !tcod::console_is_window_closed() {
        let key = tcod::console_check_for_keypress(tcod::KeyPressedOrReleased);

        tcod::console_set_default_foreground(tcod::ROOT_CONSOLE, default_fg);
        tcod::console_clear(tcod::ROOT_CONSOLE);
        for consoles.iter().advance |&con| {
            tcod::console_set_default_background(con, transparent_bg);
            tcod::console_set_default_foreground(con, default_fg);
            tcod::console_clear(con);
        }

        match update(game_state) {
            Running => (),
            Exit => break,
        }

        for consoles.iter().advance |&con| {
            tcod::console_blit(con, 0, 0, width, height,
                               tcod::ROOT_CONSOLE, 0, 0,
                               1f, 1f);
        }
        tcod::console_flush();
    }
}
