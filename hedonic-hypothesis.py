import os
from random import random, choice

import libtcodpy as tcod

from entity_component_manager import EntityComponentManager
from components import *

def int_from_color(c):
    return c.r * 256 * 256 + c.g * 256 + c.b

def color_from_int(n):
    b = n % 256
    n = n / 256
    g = n % 256
    n = n / 256
    r = n
    return tcod.Color(r,g,b)


def equal_pos(p1, p2):
    return p1.x == p2.x and p1.y == p2.y and p1.floor == p2.floor

def neighbor_pos(p1, p2):
    return abs(p1.x - p2.x) * abs(p1.y - p2.y) <= 1

def initialise_consoles(console_count, w, h, transparent_color):
    """
    Initialise the given number of new off-screen consoles and return their list.
    """
    consoles = [tcod.console_new(w, h) for _ in xrange(console_count)]
    for con in consoles:
        tcod.console_set_key_color(con, transparent_color)
    return consoles

def tile_system(e, dt_ms, layers):
    pos = e.get(Position)
    if not pos:
        return
    tile = e.get(Tile)
    con = layers[tile.level]
    tcod.console_set_char_background(con, pos.x, pos.y, tcod.black)
    tcod.console_put_char(con, pos.x, pos.y, tile.glyph, tcod.BKGND_NONE)
    tcod.console_set_char_foreground(con, pos.x, pos.y, color_from_int(tile.color))

def input_system(e, dt_ms, key):
    if not key:
        return
    pos = e.get(Position)
    if not pos:
        return
    dest = MoveDestination(pos.x, pos.y, pos.floor)
    dx, dy = 0, 0
    if key.vk == tcod.KEY_UP:
        dy = -1
    elif key.vk == tcod.KEY_DOWN:
        dy = 1
    elif key.vk == tcod.KEY_LEFT:
        dx = -1
        if key.shift:
            dy = -1
        elif key.lctrl or key.rctrl or key.lalt or key.ralt:
            dy = 1
    elif key.vk == tcod.KEY_RIGHT:
        dx = 1
        if key.shift:
            dy = -1
        elif key.lctrl or key.rctrl or key.lalt or key.ralt:
            dy = 1
    e.set(dest._replace(x=pos.x+dx, y=pos.y+dy))

def ai_system(e, ecm, dt_ms):
    return
    pos = e.get(Position)
    neighbor_vectors = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1),
                        (0, 1), (1, 1)]
    available_destinations = [Position(pos.x + dx, pos.y + dy, pos.floor)
                              for dx, dy in neighbor_vectors
                              if not blocked_tile(Position(pos.x + dx,
                                      pos.y + dy, pos.floor), ecm)]
    if available_destinations:
        dest = choice(available_destinations)
        e.set(MoveDestination(dest.x, dest.y, dest.floor))

def entities_on_position(pos, ecm):
    """
    Return all other entities with the same position.
    """
    return [entity for entity in ecm.entities(Position)
            if equal_pos(entity.get(Position), pos)]

def blocked_tile(pos, ecm):
    """
    True if the tile is non-empty or there's a bloking entity on it.
    """
    colliding = entities_on_position(pos, ecm)
    is_void = len(colliding) == 0  # you can't step into the void (i.e. no tile)
    return is_void or any((entity.has(Solid) for entity in colliding))

def collision_system(e, ecm, dt_ms):
    dest = e.get(MoveDestination)
    interactions = [entity for entity in entities_on_position(dest, ecm)
                    if entity.has(Interactive) or entity.has(Monster)]
    if blocked_tile(dest, ecm):
        e.remove(MoveDestination)
    if interactions:
        assert len(interactions) == 1, ('More than 1 interaction on a block %s'
                                        % interactions)
        i = interactions[0]
        if i.has(Interactive):
            attrs = e.get(Attributes)
            if attrs:  # base this off of the actual interaction type present
                som = attrs.state_of_mind + max(20 - attrs.tolerance, 5)
                e.set(attrs._replace(state_of_mind=som,
                                     tolerance=attrs.tolerance + 1))
            ecm.remove_entity(i)
        elif i.has(Monster):
            e.set(Attacking(i))

def combat_system(e, ecm, dt_ms):
    target = e.get(Attacking).target
    if not neighbor_pos(e.get(Position), target.get(Position)):
        return
    ecm.remove_entity(target)
    e.remove(Attacking)
    stats = e.get(Statistics)
    if stats:
        e.set(stats._replace(kills=stats.kills+1))

def movement_system(e, dt_ms, w, h):
    dest = e.get(MoveDestination)
    pos = e.get(Position)
    e.set(Position(dest.x, dest.y, dest.floor))
    if not equal_pos(pos, dest) and e.has(Statistics):
        stats = e.get(Statistics)
        e.set(stats._replace(turns=stats.turns+1))
    e.remove(MoveDestination)

def gui_system(ecm, dt_ms, player, layers, w, h, panel_height):
    attrs = player.get(Attributes)
    panel = tcod.console_new(w, panel_height)
    stats_template = "%s  Confidence: %s  Will: %s  Nerve: %s"
    tcod.console_print_ex(panel, 0, 0, tcod.BKGND_NONE, tcod.LEFT,
        stats_template % (player.get(Info).name, attrs.confidence, attrs.will,
                          attrs.nerve))
    if player.has(Dead):
        tcod.console_print_ex(panel, 0, 1, tcod.BKGND_NONE, tcod.LEFT,
                                 "DEAD: %s" % player.get(Dead).reason)
    else:
        max_bar_length = 20
        max_sate_of_mind = 100
        bar_length = attrs.state_of_mind * (max_bar_length - 1) / max_sate_of_mind
        full_bar = ' ' * (max_bar_length)
        bar = ' ' * (bar_length + 1)
        tcod.console_set_default_background(panel, tcod.dark_gray)
        tcod.console_print_ex(panel, 0, 1, tcod.BKGND_SET, tcod.LEFT, full_bar)
        if attrs.state_of_mind <  25:
            bar_color = tcod.dark_red
        elif attrs.state_of_mind < 60:
            bar_color = tcod.orange
        elif attrs.state_of_mind < 80:
            bar_color = tcod.chartreuse
        else:
            bar_color = tcod.turquoise
        tcod.console_set_default_background(panel, bar_color)
        tcod.console_print_ex(panel, 0, 1, tcod.BKGND_SET, tcod.LEFT, bar)
    tcod.console_blit(panel, 0, 0, 0, 0, layers[9], 0, h - panel_height)

# TODO: change to a generic component that indicates attribute change over time
def state_of_mind_system(ecm, dt_ms, e):
    attrs = e.get(Attributes)
    e.set(attrs._replace(state_of_mind=attrs.state_of_mind - 1))

def death_system(ecm, dt_ms, e):
    attrs = e.get(Attributes)
    if attrs:
        if attrs.state_of_mind <= 0:
            e.remove(UserInput)
            e.set(Dead("Exhausted"))
        elif attrs.state_of_mind > 100:
            e.remove(UserInput)
            e.set(Dead("Overdosed"))

def update(game, dt_ms, consoles, w, h, panel_height, pressed_key):
    ecm = game['ecm']
    player = game['player']
    last_turn_count = player.get(Statistics).turns
    if pressed_key:
        if pressed_key.vk == tcod.KEY_ESCAPE:
            return None  # Quit the game
        elif pressed_key.vk == tcod.KEY_F5:
            return initial_state(w, h, game['empty_ratio'])
        elif pressed_key.c == ord('['):
            return initial_state(w, h, game['empty_ratio'] - 0.05)
        elif pressed_key.c == ord(']'):
            return initial_state(w, h, game['empty_ratio'] + 0.05)
    for controllable in [e for e in ecm.entities(UserInput)]:
        input_system(controllable, dt_ms, key)
    for collidable in [e for e in ecm.entities(MoveDestination)]:
        collision_system(collidable, ecm, dt_ms)
    for moving in [e for e in ecm.entities(MoveDestination)]:
        movement_system(moving, dt_ms, w, h)
    for attacker in [e for e in ecm.entities(Attacking)]:
        combat_system(attacker, ecm, dt_ms)

    new_turn = last_turn_count < player.get(Statistics).turns
    if new_turn:
        for ai in [e for e in ecm.entities(AI)]:
            ai_system(ai, ecm, dt_ms)
        for entity_with_attributes in [e for e in ecm.entities(Attributes)]:
            state_of_mind_system(ecm, dt_ms, entity_with_attributes)
    for vulnerable in [e for e in ecm.entities(Attributes)]:
        death_system(ecm, dt_ms, vulnerable)
    for renderable in [e for e in ecm.entities(Tile)]:
        tile_system(renderable, dt_ms, consoles)
    gui_system(ecm, dt_ms, player, consoles, w, h, panel_height)
    tcod.console_print_ex(consoles[9], w-1, h-1, tcod.BKGND_NONE, tcod.RIGHT,
                          str(game['empty_ratio']))
    return game

def generate_map(w, h, empty_ratio):
    floor = []
    for x in xrange(w):
        for y in xrange(h):
            rand = random()
            if rand < empty_ratio:
                tile_kind = 'empty'
            elif rand < 0.99:
                tile_kind = 'wall'
            else:
                tile_kind = 'dose'
            if tile_kind == 'empty' and random() < 0.1:
                tile_kind = 'monster'
            floor.append([x, y, tile_kind])
    return [floor]

def initial_state(w, h, empty_ratio=0.6):
    ecm = EntityComponentManager(autoregister_components=True)
    # TODO: register the component types here once things settled a bit
    player_x, player_y = w / 2, h / 2
    player = ecm.new_entity()
    player.add(Position(player_x, player_y, 0))
    player.add(Tile(9, int_from_color(tcod.white), '@'))
    player.add(UserInput())
    player.add(Info(name="The Nameless One", description=""))
    player.add(Attributes(state_of_mind=20, tolerance=0, confidence=5,
                          nerve=5, will=5))
    player.add(Statistics(turns=0, kills=0, doses=0))
    player_pos = player.get(Position)
    for floor, map in enumerate(generate_map(w, h, empty_ratio)):
        for x, y, type in map:
            block = ecm.new_entity()
            block.add(Position(x, y, floor))
            empty_tile = Tile(0, int_from_color(tcod.lightest_gray), '.')
            if type == 'empty' or (x, y) == (player_x, player_y):
                block.add(empty_tile)
            elif type == 'wall':
                assert color_from_int(int_from_color(tcod.light_green)) == tcod.light_green
                color = choice((tcod.dark_green, tcod.green, tcod.light_green))
                block.add(Tile(0, int_from_color(color), '#'))
                block.add(Solid())
            elif type == 'dose':
                block.add(empty_tile)
                dose = ecm.new_entity()
                dose.add(Position(x, y, floor))
                assert color_from_int(int_from_color(tcod.light_azure)) == tcod.light_azure
                dose.add(Tile(1, int_from_color(tcod.light_azure), 'i'))
                dose.add(Interactive())
            elif type == 'monster':
                block.add(empty_tile)
                monster = ecm.new_entity()
                monster.add(Position(x, y, floor))
                assert color_from_int(int_from_color(tcod.dark_red)) == tcod.dark_red
                monster.add(Tile(1, int_from_color(tcod.dark_red), 'a'))
                monster.add(Solid())
                monster.add(Monster('a', strength=10))
                monster.add(AI('aggressive'))
            else:
                raise Exception('Unexpected tile type: "%s"' % type)
    return {
        'ecm': ecm,
        'player': player,
        'empty_ratio': empty_ratio,
    }


if __name__ == '__main__':
    SCREEN_WIDTH = 80
    SCREEN_HEIGHT = 50
    PANEL_HEIGHT = 2
    LIMIT_FPS = 60
    TRANSPARENT_BG_COLOR = tcod.red
    font_path = os.path.join('fonts', 'dejavu16x16_gs_tc.png')
    font_settings = tcod.FONT_TYPE_GREYSCALE | tcod.FONT_LAYOUT_TCOD
    game_title = 'Dose Response'
    tcod.console_set_custom_font(font_path, font_settings)
    tcod.console_init_root(SCREEN_WIDTH, SCREEN_HEIGHT, game_title, False)
    tcod.sys_set_fps(LIMIT_FPS)
    consoles = initialise_consoles(10, SCREEN_WIDTH, SCREEN_HEIGHT, TRANSPARENT_BG_COLOR)
    game_state = initial_state(SCREEN_WIDTH, SCREEN_HEIGHT - PANEL_HEIGHT)
    while game_state and not tcod.console_is_window_closed():
        tcod.console_set_default_foreground(0, tcod.white)
        key = tcod.console_check_for_keypress(tcod.KEY_PRESSED)
        if key.vk == tcod.KEY_NONE:
            key = None
        dt_ms = 10
        tcod.console_clear(None)
        for con in consoles:
            tcod.console_set_default_background(con, TRANSPARENT_BG_COLOR)
            tcod.console_set_default_foreground(con, tcod.white)
            tcod.console_clear(con)
        game_state = update(game_state, dt_ms, consoles,
                            SCREEN_WIDTH, SCREEN_HEIGHT, PANEL_HEIGHT, key)
        for con in consoles:
            tcod.console_blit(con, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0, 0, 0)
        tcod.console_flush()
