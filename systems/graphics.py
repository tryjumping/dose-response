import itertools
import math

import lib.libtcodpy as tcod
from lib.enum import Enum

from components import *


class Color(Enum):
    transparent = tcod.peach
    black = tcod.black
    dim_background = tcod.Color(15, 15, 15)
    foreground = tcod.white
    anxiety = tcod.dark_red
    depression = tcod.light_han
    hunger = tcod.light_sepia
    voices = tcod.dark_grey
    shadows = tcod.dark_grey
    player = tcod.white
    empty_tile = tcod.lightest_grey
    dose = tcod.light_azure
    dose_glow = tcod.darkest_turquoise
    wall_1 = tcod.dark_green
    wall_2 = tcod.green
    wall_3 = tcod.light_green

def in_fov(x, y, fov_map, cx, cy, radius):
    """
    Return true if the position is within the field of view given by the map and
    a radius.
    """
    if not tcod.map_is_in_fov(fov_map, x, y) or radius < 1:
        return False
    distance = precise_distance(Position(x, y, 0), Position(cx, cy, 0))
    return math.floor(distance) <= radius

def precise_distance(p1, p2):
    """
    Return a distance between two points.

    The distance is based on a pythagorean calculation rather than a rough
    heuristic.
    """
    return math.sqrt((abs(p1.x - p2.x) ** 2) + (abs(p1.y - p2.y) ** 2))


def set_background(ctx, x, y, color):
    """
    Sets the background colour of a given screen cell in the given drawing
    context.

    This will never interfere with the rendering of any charactern on any level
    in the context (unless they have the same foreground colour in which case
    they won't be visible).
    """
    background_console = ctx[0]
    tcod.console_set_char_background(background_console, x, y, color)

def draw_char(ctx, level, x, y, char, color=None):
    """
    Puts the character of the given position and optionally colour on screen.

    `ctx` is the drawing context and `layer` is a layer within that context.

    Character of a lower layer is overwritten by the character of a higher one.
    """
    background_console = ctx[0]
    char_background = tcod.console_get_char_background(background_console, x, y)
    con = ctx[level]
    tcod.console_set_char_background(con, x, y, char_background)
    tcod.console_put_char(con, x, y, char)
    if color:
        tcod.console_set_char_foreground(con, x, y, color)


def background_system(ecm, w, h, player_pos, game, ctx, player, cheating):
    px, py = player_pos.x, player_pos.y
    for x, y in itertools.product(xrange(w), xrange(h)):
        set_background(ctx, x, y, Color.black.value)
    for x, y in itertools.product(xrange(w), xrange(h)):
        visible = in_fov(x, y, game['fov_map'], px, py, game['fov_radius'])
        if visible:
            set_background(ctx, x, y, Color.black.value)
    for dose in ecm.entities(Dose):
        explored = dose.has(Explorable) and dose.get(Explorable).explored
        pos = dose.get(Position)
        resistance_radius = player.get(Addicted).resistance
        visible = in_fov(pos.x, pos.y, game['fov_map'], px, py, game['fov_radius'])
        if not visible and not cheating and not explored:
            continue
        for rdx in range(-resistance_radius, resistance_radius + 1):
            for rdy in range(-resistance_radius, resistance_radius + 1):
                glow_x, glow_y = pos.x + rdx, pos.y + rdy
                if visible or cheating:
                    set_background(ctx, glow_x, glow_y, Color.dose_glow.value)


def tile_system(e, pos, tile, ctx, fov_map, player, radius, cheating):
    if not all((e.has(c) for c in (Tile, Position))):
        return
    explored = e.has(Explorable) and e.get(Explorable).explored
    player_pos = player.get(Position)
    if player_pos:
        px, py = player_pos.x, player_pos.y
    else:
        px, py, radius = 0, 0, 0
    visible = in_fov(pos.x, pos.y, fov_map, px, py, radius)
    if visible or explored or cheating:
        if e.has(Explorable) and visible:
            e.set(Explorable(explored=True))
        draw_char(ctx, tile.level, pos.x, pos.y, tile.glyph, tile.color.value)
        if not visible:
            set_background(ctx, pos.x, pos.y, Color.dim_background.value)
