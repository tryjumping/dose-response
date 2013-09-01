import os
import string

import lib.libtcodpy as tcod


def generate_world(w, h):
    world = []
    for y in range(h):
        for x in range(w):
            glyph = string.ascii_letters[(x * y) % len(string.ascii_letters)]
            world.append((x, y, glyph))
    return world

def draw(layers, world, w, h):
    con = layers[-1]
    for x, y, glyph in world:
        tcod.console_set_char_background(con, x, y, tcod.darkest_gray)
        tcod.console_put_char(con, x, y, glyph)
    tcod.console_print_ex(con, w-1, h-1, tcod.BKGND_NONE, tcod.RIGHT,
                          "FPS: %s" %tcod.sys_get_fps())

if __name__ == '__main__':
    WIDTH = 80
    HEIGHT = 50
    world = generate_world(WIDTH, HEIGHT)

    TRANSPARENT_BG = tcod.red
    CONSOLE_COUNT = 10
    font_path = os.path.join('fonts', 'dejavu16x16_gs_tc.png')
    font_settings = tcod.FONT_TYPE_GREYSCALE | tcod.FONT_LAYOUT_TCOD
    tcod.console_set_custom_font(font_path, font_settings)
    tcod.console_init_root(WIDTH, HEIGHT, 'tcod bench', False)
    consoles = []
    for _ in range(CONSOLE_COUNT):
        con = tcod.console_new(WIDTH, HEIGHT)
        tcod.console_set_key_color(con, TRANSPARENT_BG)
        consoles.append(con)
    while not tcod.console_is_window_closed():
        tcod.console_set_default_foreground(0, tcod.white)
        key = tcod.console_check_for_keypress(tcod.KEY_PRESSED)
        if key.vk == tcod.KEY_NONE:
            key = None
        elif key.vk == tcod.KEY_ESCAPE:
            exit()
        tcod.console_clear(None)
        for con in consoles:
            tcod.console_set_default_background(con, TRANSPARENT_BG)
            tcod.console_set_default_foreground(con, tcod.white)
            tcod.console_clear(con)

        draw(consoles, world, WIDTH, HEIGHT)

        for con in consoles:
            tcod.console_blit(con, 0, 0, WIDTH, HEIGHT, 0, 0, 0, 1)
        tcod.console_flush()
