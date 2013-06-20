import os

import libtcodpy as libtcod

SCREEN_WIDTH = 80
SCREEN_HEIGHT = 50
LIMIT_FPS = 20

font_path = os.path.join('fonts', 'arial12x12.png')
libtcod.console_set_custom_font(font_path, libtcod.FONT_TYPE_GREYSCALE | libtcod.FONT_LAYOUT_TCOD)
libtcod.console_init_root(SCREEN_WIDTH, SCREEN_HEIGHT, 'Hedonic Hypothesis', False)
libtcod.sys_set_fps(LIMIT_FPS)

while not libtcod.console_is_window_closed():
    libtcod.console_set_default_foreground(0, libtcod.white)
    libtcod.console_put_char(0, 1, 1, '@', libtcod.BKGND_NONE)
    libtcod.console_flush()
    key = libtcod.console_check_for_keypress()
    if key.vk != libtcod.KEY_NONE:
        print("key pressed:", key.c, key.vk)
