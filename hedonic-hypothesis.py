from collections import namedtuple
import os

import libtcodpy as libtcod

from entity_component_manager import EntityComponentManager
from components import *


def tile_system(e, dt_ms):
    pos = e.get(Position)
    if not pos:
        return
    tile = e.get(Tile)
    libtcod.console_put_char(0, pos.x, pos.y, tile.glyph, libtcod.BKGND_NONE)

def input_system(e, dt_ms, key):
    if not key:
        return
    pos = e.get(Position)
    if not pos:
        return
    dest = MoveDestination(pos.x, pos.y, pos.floor)
    if key.vk == libtcod.KEY_UP:
        dest.y -= 1
    elif key.vk == libtcod.KEY_DOWN:
        dest.y += 1
    elif key.vk == libtcod.KEY_LEFT:
        dest.x -= 1
    elif key.vk == libtcod.KEY_RIGHT:
        dest.x += 1
    e.set(dest)

def movement_system(e, dt_ms):
    dest = e.get(MoveDestination)
    # TODO: test collision
    e.set(Position(dest.x, dest.y, dest.floor))
    e.remove(MoveDestination)

def update(game, dt_ms, w, h, key):
    ecm = game['ecm']
    if key and key.vk == libtcod.KEY_ESCAPE:
        return None  # Quit the game
    for controllable in [e for e in ecm.entities(UserInput)]:
        input_system(controllable, dt_ms, key)
    for moving in [e for e in ecm.entities(MoveDestination)]:
        movement_system(e, dt_ms)
    for renderable in [e for e in ecm.entities(Tile)]:
        tile_system(renderable, dt_ms)
    return game

def initial_state(w, h):
    ecm = EntityComponentManager()
    ecm.register_component_type(Position)
    ecm.register_component_type(MoveDestination)
    ecm.register_component_type(Tile)
    ecm.register_component_type(UserInput)
    player = ecm.new_entity()
    player.set(Position(w / 2, h / 2, 1))
    player.set(Tile('player', None, '@'))
    player.set(UserInput())
    return {'ecm': ecm}


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
        game_state = update(game_state, dt_ms, SCREEN_WIDTH, SCREEN_HEIGHT, key)
        libtcod.console_flush()
