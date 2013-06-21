from collections import namedtuple
import os

import libtcodpy as libtcod


Position = namedtuple('Position', ['x', 'y', 'floor'])

def update(game, dt_ms, key):
    pos = game['player']['position']
    w, h = game['screen']['width'], game['screen']['height']
    newpos = pos
    if key:
        if key.vk == libtcod.KEY_UP:
            newpos = pos._replace(y=max(pos.y - 1, 0))
        elif key.vk == libtcod.KEY_DOWN:
            newpos = pos._replace(y=min(pos.y + 1, h - 1))
        elif key.vk == libtcod.KEY_LEFT:
            newpos = pos._replace(x=max(pos.x - 1, 0))
        elif key.vk == libtcod.KEY_RIGHT:
            newpos = pos._replace(x=min(pos.x + 1, w - 1))
        elif key.vk == libtcod.KEY_ESCAPE:
            return None
    game['player']['position'] = newpos
    libtcod.console_put_char(0, newpos.x, newpos.y, '@', libtcod.BKGND_NONE)
    return game

def initial_state(w, h):
    return {
        'player': {
            'position': Position(w / 2, h / 2, 1)
        },
        'screen': {'width': w, 'height': h}
    }


if __name__ == '__main__':
    SCREEN_WIDTH = 80
    SCREEN_HEIGHT = 50
    LIMIT_FPS = 60
    font_path = os.path.join('fonts', 'arial12x12.png')
    font_settings = libtcod.FONT_TYPE_GREYSCALE | libtcod.FONT_LAYOUT_TCOD
    game_title = 'Hedonic Hypothesis'
    libtcod.console_set_custom_font(font_path, font_settings)
    libtcod.console_init_root(SCREEN_WIDTH, SCREEN_HEIGHT, game_title, False)
    libtcod.sys_set_fps(LIMIT_FPS)
    game_state = initial_state(SCREEN_WIDTH, SCREEN_HEIGHT)
    while game_state and not libtcod.console_is_window_closed():
        libtcod.console_set_default_foreground(0, libtcod.white)
        key = libtcod.console_check_for_keypress(libtcod.KEY_PRESSED)
        if key.vk == libtcod.KEY_NONE:
            key = None
        dt_ms = 10
        libtcod.console_clear(None)
        game_state = update(game_state, dt_ms, key)
        libtcod.console_flush()
