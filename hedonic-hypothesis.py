from collections import namedtuple
import os
from random import random

import libtcodpy as libtcod

from entity_component_manager import EntityComponentManager
from components import *

draw_commands = None

def clear_leveled_graphics():
    global draw_commands
    draw_commands = [[] for _ in range(10)]

def put_char_with_level(level, x, y, glyph, color):
    global draw_commands
    draw_commands[level].append((libtcod.console_put_char, 0, x, y, glyph, color))


def tile_system(e, dt_ms):
    pos = e.get(Position)
    if not pos:
        return
    tile = e.get(Tile)
    put_char_with_level(tile.level, pos.x, pos.y, tile.glyph, libtcod.BKGND_NONE)

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

def movement_system(e, dt_ms, w, h):
    dest = e.get(MoveDestination)
    pos = e.get(Position)
    if dest.x < 0 or dest.x >= w:
        dest.x = pos.x
    if dest.y < 0 or dest.y >= h:
        dest.y = pos.y
    e.set(Position(dest.x, dest.y, dest.floor))
    e.remove(MoveDestination)

def update(game, dt_ms, w, h, key):
    ecm = game['ecm']
    if key and key.vk == libtcod.KEY_ESCAPE:
        return None  # Quit the game
    for controllable in [e for e in ecm.entities(UserInput)]:
        input_system(controllable, dt_ms, key)
    for moving in [e for e in ecm.entities(MoveDestination)]:
        movement_system(e, dt_ms, w, h)
    for renderable in [e for e in ecm.entities(Tile)]:
        tile_system(renderable, dt_ms)
    return game

def generate_map(w, h):
    floor = []
    for x in xrange(w):
        for y in xrange(h):
            tile_kind = 'empty'
            if random() > 0.7:
                tile_kind = 'wall'
            floor.append([x, y, tile_kind])
    return [floor]

def initial_state(w, h):
    ecm = EntityComponentManager()
    ecm.register_component_type(Position)
    ecm.register_component_type(MoveDestination)
    ecm.register_component_type(Tile)
    ecm.register_component_type(UserInput)
    ecm.register_component_type(Solid)
    player = ecm.new_entity()
    player.set(Position(w / 2, h / 2, 1))
    player.set(Tile(9, None, '@'))
    player.set(UserInput())
    for floor, map in enumerate(generate_map(w, h)):
        for x, y, type in map:
            block = ecm.new_entity()
            block.set(Position(x, y, floor+1))
            if type == 'empty':
                block.set(Tile(0, None, ' '))
            else:
                block.set(Tile(0, None, '#'))
                block.set(Solid())
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
        clear_leveled_graphics()
        game_state = update(game_state, dt_ms, SCREEN_WIDTH, SCREEN_HEIGHT, key)
        for level, commands in enumerate(draw_commands):
            for command in commands:
                fun, args = command[0], command[1:]
                fun(*args)
        libtcod.console_flush()
