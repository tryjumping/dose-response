import itertools
import math

import lib.libtcodpy as tcod
from lib.enum import Enum

from components import *


# right. so. the tcod's transparent background blit colour makes the whole
# cell transparent even if it has a character set. which is not *quite* what
# we want. We want to be able to set the background and then have a
# multitude of layers where we render characters or just an empty box with
# the underlying background if there is no char specified. But setting a
# background sholud never hide the rendered character, nor sholud a
# character be hidden because we did not specify its background explicitly.

# I think we may want to take the idea with the background field further:
# write our own set-background and put-char functions that will produce the correct behaviour.
# Not sure if it makes more sense to build our own background/foreground layer arrays
# or if we can still shoehorn the desired functionnility within tocd's multiple consoles
# when we won't be using the tcod graphic calls directly.

# I think we can: have a separate background layer and then consoles for fg.
# And whenever we call set_char(x, y, glyph, fg-colour), it will read the
# bg-colour at that point and set it, too.

# Oh and we probably want to extract the graphics code into multiple systems
# (bacgkround, fog of war, dose glow, tiles, animations)


_BACKGROUND = [None for bkoffset in range(80*50)]


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



def background_system(ecm, w, h, player_pos, game, consoles, player, cheating):
    global _BACKGROUND
    for bkoffset in range(w * h):
        _BACKGROUND[bkoffset] = Color.dim_background.value
    import itertools
    for x, y in itertools.product(range(w), range(h)):
        px, py = player_pos.x, player_pos.y
        visible = in_fov(x, y, game['fov_map'], px, py, game['fov_radius'])
        if visible:
            _BACKGROUND[x + (y * w)] = Color.black.value
    for dose in ecm.entities(Dose):
        explored = dose.has(Explorable) and dose.get(Explorable).explored
        pos = dose.get(Position)
        con = consoles[0]
        px, py = player_pos.x, player_pos.y
        resistance_radius = player.get(Addicted).resistance
        visible = in_fov(pos.x, pos.y, game['fov_map'], px, py, game['fov_radius'])
        if not visible and not cheating and not explored:
            continue
        for rdx in range(-resistance_radius, resistance_radius + 1):
            for rdy in range(-resistance_radius, resistance_radius + 1):
                glow_x, glow_y = pos.x + rdx, pos.y + rdy
                if visible or cheating:
                    _BACKGROUND[glow_x + (glow_y * w)] = Color.dose_glow.value


def tile_system(e, pos, tile, layers, fov_map, player, radius, cheating):
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
        con = layers[tile.level]
        tcod.console_set_char_background(con, pos.x, pos.y, _BACKGROUND[pos.x+(pos.y*80)])
        tcod.console_put_char(con, pos.x, pos.y, tile.glyph)
        tcod.console_set_char_foreground(con, pos.x, pos.y, tile.color.value)
