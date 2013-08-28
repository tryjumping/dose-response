import itertools
import math

import lib.libtcodpy as tcod
from lib.enum import Enum

from components import *
from systems.state_of_mind import enumerate_state_of_mind, StateOfMind


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
    distance = precise_distance((x, y), (cx, cy))
    return math.floor(distance) <= radius

def precise_distance(p1, p2):
    """
    Return a distance between two points.

    The distance is based on a pythagorean calculation rather than a rough
    heuristic.
    """
    x1, y1 = p1
    x2, y2 = p2
    return math.sqrt((abs(x1 - x2) ** 2) + (abs(y1 - y2) ** 2))


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
    for glowee in ecm.entities(Glow):
        pos = glowee.get(Position)
        explored = glowee.has(Explorable) and glowee.get(Explorable).explored
        visible = in_fov(pos.x, pos.y, game['fov_map'], px, py, game['fov_radius'])
        if not visible and not cheating and not explored:
            continue
        radius = glowee.get(Glow).radius
        for rdx in range(-radius, radius + 1):
            for rdy in range(-radius, radius + 1):
                glow_x, glow_y = pos.x + rdx, pos.y + rdy
                if visible or cheating:
                    set_background(ctx, glow_x, glow_y, glowee.get(Glow).color.value)


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
    if player.get(Abilities).see_entities and (e.has(Monster) or e.has(Dose)):
        visible = True
    if visible or explored or cheating or player.get(Abilities).see_world:
        if e.has(Explorable) and visible:
            e.set(Explorable(explored=True))
        draw_char(ctx, tile.level, pos.x, pos.y, tile.glyph, tile.color.value)
        if not visible:
            set_background(ctx, pos.x, pos.y, Color.dim_background.value)


def describe_state_of_mind(som):
    """Return a textual repsesentation of the given state of mind value."""
    som_map = {
        StateOfMind.dead: 'Exhausted',
        StateOfMind.delirium_tremens: 'Delirium tremens',
        StateOfMind.severe_withdrawal: 'Severe withdrawal',
        StateOfMind.withdrawal: 'Withdrawal',
        StateOfMind.sober: 'Sober',
        StateOfMind.high: 'High',
        StateOfMind.very_high: 'High as a kite',
        StateOfMind.overdosed: 'Overdosed',
    }
    return som_map[enumerate_state_of_mind(som)]

def gui_system(ecm, player, layers, w, h, panel_height, cheating, dt):
    attrs = player.get(Attributes)
    panel = tcod.console_new(w, panel_height)
    if cheating:
        stats_bar = ("%s   SoM: %s, Tolerance: %s, Confidence: %s  Will: %s  Nerve: %s" %
                     (player.get(Info).name, attrs.state_of_mind,
                              attrs.tolerance, attrs.confidence, attrs.will,
                              attrs.nerve))
    else:
        stats_bar = "%s    Will: %s" % (player.get(Info).name, attrs.will)
    tcod.console_print_ex(panel, 0, 0, tcod.BKGND_NONE, tcod.LEFT,
        stats_bar)
    if player.has(Dead):
        tcod.console_print_ex(panel, 0, 1, tcod.BKGND_NONE, tcod.LEFT,
                                 "DEAD: %s" % player.get(Dead).reason)
    else:
        states = [describe_state_of_mind(attrs.state_of_mind)]
        stun_effect = player.get(StunEffect)
        if stun_effect and stun_effect.duration > 0:
            states.append('Stunned (%s)' % stun_effect.duration)
        panic_effect = player.get(PanicEffect)
        if panic_effect and panic_effect.duration > 0:
                states.append('Panic (%s)' % panic_effect.duration)
        tcod.console_print_ex(panel, 0, 1, tcod.BKGND_NONE, tcod.LEFT,
                              ' | '.join(states))
    doses = len([e for e in ecm.entities(Interactive)])
    monsters = len([e for e in ecm.entities(Monster)])
    tcod.console_print_ex(panel, w-1, 1, tcod.BKGND_NONE, tcod.RIGHT,
                          "Doses: %s,  Monsters: %s, dt: %s, FPS: %s" %
                          (doses, monsters, dt, tcod.sys_get_fps()))
    tcod.console_blit(panel, 0, 0, 0, 0, layers[9], 0, h - panel_height)
