import collections
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


def distance(p1, p2):
    """
    Return distance between two points on the tile grid.
    """
    assert p1.floor == p2.floor, "Positions must be on the same floor"
    assert p1 and p2, "Must be valid positions"
    return max(abs(p1.x - p2.x), abs(p1.y - p2.y))

def equal_pos(p1, p2):
    """
    Return True when the two positions are equal.
    """
    return p1.floor == p2.floor and distance(p1, p2) == 0

def neighbor_pos(p1, p2):
    """
    Return True when the two position are touching.
    """
    return distance(p1, p2) <= 1

def has_free_aps(e):
    turn = e.get(Turn)
    return turn and turn.action_points > 0

def initialise_consoles(console_count, w, h, transparent_color):
    """
    Initialise the given number of new off-screen consoles and return their list.
    """
    consoles = [tcod.console_new(w, h) for _ in xrange(console_count)]
    for con in consoles:
        tcod.console_set_key_color(con, transparent_color)
    return consoles

def tile_system(e, pos, tile, layers, fov_map):
    if not all((e.has(c) for c in (Tile, Position))):
        return
    explored = e.has(Explorable) and e.get(Explorable).explored
    if tcod.map_is_in_fov(fov_map, pos.x, pos.y) or explored:
        if e.has(Explorable):
            e.set(Explorable(explored=True))
        con = layers[tile.level]
        tcod.console_set_char_background(con, pos.x, pos.y, tcod.black)
        tcod.console_put_char(con, pos.x, pos.y, tile.glyph, tcod.BKGND_NONE)
        tcod.console_set_char_foreground(con, pos.x, pos.y, color_from_int(tile.color))

def input_system(e, ecm, keys):
    if not keys:
        return
    pos = e.get(Position)
    if not pos:
        return
    key = keys.popleft()
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
    if dx != 0 or dy != 0:
        e.set(dest._replace(x=pos.x+dx, y=pos.y+dy))

def ai_system(e, ai, pos, ecm, player, w, h):
    if not all((e.has(c) for c in (AI, Position))):
        return
    # TODO: use an action point system. It should make things simpler: if we
    # moved, we don't have any attack actions. If we didn't move, we can attack.
    # Will help us deal with the interactions, too.
    neighbor_vectors = ((-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1),
                        (0, 1), (1, 1))
    destinations = [Position(pos.x + dx, pos.y + dy, pos.floor) for dx, dy
                    in neighbor_vectors]
    player_pos = player.get(Position)
    if player_pos in destinations:
        dest = player_pos
    else:
        e.set(ai._replace(kind='idle'))
        destinations = [dest for dest in destinations
                        if not blocked_tile(dest, ecm)
                           and within_rect(dest, 0, 0, w, h)]
        if destinations:
            dest = choice(destinations)
        else:
            dest = None
    if dest:
        e.set(MoveDestination(dest.x, dest.y, dest.floor))
        if equal_pos(player_pos, dest) or neighbor_pos(player_pos, dest):
            if ai.kind == 'idle':
                e.set(ai._replace(kind='aggressive'))
            elif ai.kind == 'aggressive':
                e.set(Attacking(player))
    else:
        e.set(MoveDestination(pos.x, pos.y, pos.floor))


def entities_on_position(pos, ecm):
    """
    Return all other entities with the same position.
    """
    return (entity for entity
            in ecm.entities_by_component_value(Position,
                                               x=pos.x, y=pos.y, floor=pos.floor))


def blocked_tile(pos, ecm):
    """
    True if the tile is non-empty or there's a bloking entity on it.
    """
    return any((entity.has(Solid) for entity
                in entities_on_position(pos, ecm)))

def within_rect(pos, x, y, w, h):
    """
    True if the tile is within the rectangle of the specified coordinates and
    dimension.
    """
    assert hasattr(pos, 'x') and hasattr(pos, 'y')
    assert x <= w and y <= h
    return x <= pos.x < x + w and y <= pos.y < y + h

def entity_spend_ap(e, spent=1):
    turns = e.get(Turn)
    e.set(turns._replace(action_points = turns.action_points - spent))

def interaction_system(e, target, ecm):
    if not all((e.has(c) for c in (Position, MoveDestination, Turn))):
        return
    interactions = [entity for entity in entities_on_position(target, ecm)
                    if entity.has(Interactive)]
    monsters = [entity for entity in entities_on_position(target, ecm)
                if entity.has(Monster)]
    for m in monsters:
        if has_free_aps(e) and not e.has(Monster):
            e.set(Attacking(m))
    for i in interactions:
        if has_free_aps(e) > 0 and i.has(Interactive) and e.has(Addicted):
            attrs = e.get(Attributes)
            if attrs:  # base this off of the actual interaction type present
                som = attrs.state_of_mind + max(50 - attrs.tolerance, 5)
                e.set(attrs._replace(state_of_mind=som,
                                     tolerance=attrs.tolerance + 1))
            ecm.remove_entity(i)
    if monsters or interactions:
        return True

def entity_strength(e):
    """
    Returns the combat strength of the given entity.
    """
    if e.has(Monster):
        return e.get(Monster).strength
    elif e.has(Attributes):
        attrs = e.get(Attributes)
        return attrs.confidence + attrs.nerve + attrs.will
    else:
        raise AssertionError('Attacker must be either the player or a monster')

def combat_system(e, ecm):
    if not all((e.has(c) for c in (Attacking, Turn, Info))):
        return
    target = e.get(Attacking).target
    e.remove(Attacking)
    if not has_free_aps(e) or not neighbor_pos(e.get(Position),
                                               target.get(Position)):
        return
    print "%s attacks %s" % (e, target)
    entity_spend_ap(e)
    attack_str = entity_strength(e)
    defense_str = entity_strength(target)
    death_reason = "Killed by %s" % e.get(Info).name
    if target.has(Monster):  # The player always kills the monster
        kill_entity(target, death_reason)
    elif attack_str > defense_str:
        if target.has(Attributes) and e.has(AttributeModifier):
            attrs = target.get(Attributes)
            modif = e.get(AttributeModifier)
            target.set(attrs._replace(
                state_of_mind = attrs.state_of_mind + modif.state_of_mind,
                tolerance = attrs.tolerance + modif.tolerance,
                confidence = attrs.confidence + modif.confidence,
                nerve = attrs.nerve + modif.nerve,
                will = attrs.will + modif.will))
            if target.get(Attributes).state_of_mind <= 0:
                kill_entity(target, death_reason)
        else:
            raise AssertionError('Target must be either a monster or a player')
        if target.has(Dead):
            stats = e.get(Statistics)
            if stats:
                e.set(stats._replace(kills=stats.kills+1))
    else:
        print '%s defends itself against the attack' % target

def movement_system(e, pos, dest, ecm, w, h):
    if not all((e.has(c) for c in (Position, MoveDestination, Turn))):
        return
    e.remove(MoveDestination)
    if not has_free_aps(e):
        print "%s tried to move but has no action points" % e
        return
    if equal_pos(pos, dest):
        # The entity waits a turn
        print "%s waits" % e
        entity_spend_ap(e)
    elif not blocked_tile(dest, ecm) and within_rect(dest, 0, 0, w, h):
        e.set(Position(dest.x, dest.y, dest.floor))
        entity_spend_ap(e)

def gui_system(ecm, player, layers, w, h, panel_height):
    attrs = player.get(Attributes)
    panel = tcod.console_new(w, panel_height)
    stats_template = "%s  SoM: %s, Tolerance: %s, Confidence: %s  Will: %s  Nerve: %s"
    tcod.console_print_ex(panel, 0, 0, tcod.BKGND_NONE, tcod.LEFT,
        stats_template % (player.get(Info).name, attrs.state_of_mind,
                          attrs.tolerance, attrs.confidence, attrs.will,
                          attrs.nerve))
    if player.has(Dead):
        tcod.console_print_ex(panel, 0, 1, tcod.BKGND_NONE, tcod.LEFT,
                                 "DEAD: %s" % player.get(Dead).reason)
    doses = len([e for e in ecm.entities(Interactive)])
    monsters = len([e for e in ecm.entities(Monster)])
    tcod.console_print_ex(panel, w-1, 1, tcod.BKGND_NONE, tcod.RIGHT,
                          "Doses: %s,  Monsters: %s" % (doses, monsters))
    tcod.console_blit(panel, 0, 0, 0, 0, layers[9], 0, h - panel_height)

def kill_entity(e, death_reason=''):
    for ctype in (UserInput, AI, Position, Tile, Turn):
        e.remove(ctype)
    e.set(Dead(death_reason))

def entity_start_a_new_turn(e):
    t = e.get(Turn)
    e.set(t._replace(active=True, action_points=t.max_aps))

def end_of_turn_system(e, ecm):
    if not all((e.has(c) for c in (Turn,))):
        return
    turn = e.get(Turn)
    e.set(turn._replace(count=turn.count+1))

def addiction_system(e, ecm):
    if not all((e.has(c) for c in (Addicted, Attributes, Turn))):
        return
    addiction = e.get(Addicted)
    attrs = e.get(Attributes)
    turn = e.get(Turn)
    dt = turn.count - addiction.turn_last_activated
    if dt > 0:
        state_of_mind = attrs.state_of_mind - (addiction.rate_per_turn * dt)
        e.set(attrs._replace(state_of_mind=state_of_mind))
        e.set(addiction._replace(turn_last_activated=turn.count))
        if state_of_mind <= 0:
            kill_entity(e, "Exhausted")
        elif state_of_mind > 100:
            kill_entity(e, "Overdosed")

def process_entities(player, ecm, w, h, keys):
    if player.has(Dead):
        return

    player_turn = player.get(Turn)
    if player_turn.active and not has_free_aps(player):
        player.set(player_turn._replace(active=False))
        for npc in ecm.entities(AI):
            entity_start_a_new_turn(npc)
    if not player_turn.active:
        npcs = list(ecm.entities(AI))
        if not any((has_free_aps(npc) for npc in npcs)):
            end_of_turn_system(player, ecm)
            for e in npcs:
                end_of_turn_system(e, ecm)
            entity_start_a_new_turn(player)
            for npc in npcs:
                npc.set(npc.get(Turn)._replace(active=False))
    assert any((e.get(Turn).active and e.get(Turn).action_points > 0
                for e in ecm.entities(Turn)))

    for e in ecm.entities(Addicted, Attributes, Turn):
        addiction_system(e, ecm)
    for e in ecm.entities(UserInput):
        if has_free_aps(e) and keys:
            input_system(e, ecm, keys)
    for e, ai, pos in ecm.entities(AI, Position, include_components=True):
        if has_free_aps(e):
            ai_system(e, ai, pos, ecm, player, w, h)
    for e, pos, dest in ecm.entities(Position, MoveDestination,
                                       include_components=True):
        interaction_system(e, dest, ecm)
        movement_system(e, pos, dest, ecm, w, h)
    for e in ecm.entities(Attacking):
        combat_system(e, ecm)

def update(game, dt_ms, consoles, w, h, panel_height, pressed_key):
    ecm = game['ecm']
    player = game['player']
    if pressed_key:
        if pressed_key.vk == tcod.KEY_ESCAPE:
            return None  # Quit the game
        elif pressed_key.vk == tcod.KEY_F5:
            return initial_state(w, h, game['empty_ratio'])
        elif pressed_key.c == ord('d'):
            import pdb; pdb.set_trace()
        else:
            game['keys'].append(pressed_key)

    process_entities(player, ecm, w, h, game['keys'])

    player_pos = player.get(Position)
    if player_pos:
        game['recompute_fov'](game['fov_map'], player_pos.x, player_pos.y)
    for e, pos, tile in ecm.entities(Position, Tile, include_components=True):
        tile_system(e, pos, tile, consoles, game['fov_map'])
    game['fade'] = max(player.get(Attributes).state_of_mind / 100.0, 0.14)
    if player.has(Dead):
        game['fade'] = 2
    gui_system(ecm, player, consoles, w, h, panel_height)
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
            if tile_kind == 'empty' and random() < 0.05:
                tile_kind = 'monster'
            floor.append([x, y, tile_kind])
    return [floor]

def make_anxiety_monster(e):
    e.add(Tile(8, int_from_color(tcod.dark_red), 'a'))
    e.add(Monster('anxiety', strength=20))
    e.add(Info('Anxiety', "Won't give you a second of rest."))
    e.add(AttributeModifier(state_of_mind=-25, tolerance=0, confidence=0, nerve=0,
                            will=-1))
    e.add(AI('idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def make_depression_monster(e):
    e.add(Tile(8, int_from_color(tcod.light_han), 'D'))
    e.add(Monster('depression', strength=10000))
    e.add(Info('Depression', "Fast and deadly. Don't let it get close."))
    e.add(AttributeModifier(state_of_mind=-10000, tolerance=0, confidence=0,
                            nerve=0, will=0))
    e.add(AI('idle'))
    e.add(Turn(action_points=0, max_aps=2, active=False, count=0))

def make_hunger_monster(e):
    e.add(Tile(8, int_from_color(tcod.light_sepia), 'h'))
    e.add(Monster('hunger', strength=10))
    e.add(Info('Hunger', ""))
    e.add(AttributeModifier(state_of_mind=-5, tolerance=0, confidence=0, nerve=0,
                            will=0))
    e.add(AI('idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def initial_state(w, h, empty_ratio=0.6):
    fov_map = tcod.map_new(SCREEN_WIDTH, SCREEN_HEIGHT - PANEL_HEIGHT)

    ecm = EntityComponentManager(autoregister_components=True)
    ecm.register_component_type(Position, (int, int, int), index=True)
    # TODO: register the component types here once things settled a bit
    player_x, player_y = w / 2, h / 2
    player = ecm.new_entity()
    player.add(Position(player_x, player_y, 0))
    player.add(Tile(9, int_from_color(tcod.white), '@'))
    player.add(UserInput())
    player.add(Info(name="The Nameless One", description=""))
    player.add(Attributes(state_of_mind=20, tolerance=0, confidence=5,
                          nerve=5, will=5))
    player.add(Turn(action_points=1, max_aps=1, active=True, count=0))
    player.add(Statistics(turns=0, kills=0, doses=0))
    player.add(Solid())
    player.add(Addicted(1, 0))
    player_pos = player.get(Position)
    initial_dose_pos = Position(
        player_x + choice([n for n in range(-3, 3) if n != 0]),
        player_y + choice([n for n in range(-3, 3) if n != 0]),
        player_pos.floor
    )
    def near_player(x, y):
        return distance(player_pos, Position(x, y, player_pos.floor)) < 6
    for floor, map in enumerate(generate_map(w, h, empty_ratio)):
        for x, y, type in map:
            transparent = True
            walkable = True
            pos = Position(x, y, floor)
            if not type == 'wall':
                background = ecm.new_entity()
                background.add(pos)
                background.add(Tile(0, int_from_color(tcod.lightest_grey), '.'))
                background.add(Explorable(explored=False))
            if equal_pos(player_pos, pos):
                pass
            elif ((type == 'dose' and not near_player(x, y))
                  or equal_pos(initial_dose_pos, pos)):
                dose = ecm.new_entity()
                dose.add(pos)
                dose.add(Tile(5, int_from_color(tcod.light_azure), 'i'))
                dose.add(Interactive())
            elif type == 'wall':
                block = ecm.new_entity()
                block.add(pos)
                color = choice((tcod.dark_green, tcod.green, tcod.light_green))
                block.add(Tile(0, int_from_color(color), '#'))
                block.add(Solid())
                block.add(Explorable(explored=False))
                walkable = False
            elif type == 'monster' and not near_player(x, y):
                monster = ecm.new_entity()
                monster.add(pos)
                monster.add(Solid())
                factories = [
                    make_anxiety_monster,
                    make_depression_monster,
                    make_hunger_monster,
                ]
                choice(factories)(monster)
            tcod.map_set_properties(fov_map, x, y, transparent, walkable)
    def recompute_fov(fov_map, x, y):
        tcod.map_compute_fov(fov_map, x, y, 3, True)
    recompute_fov(fov_map, player_x, player_y)
    return {
        'ecm': ecm,
        'player': player,
        'empty_ratio': empty_ratio,
        'keys': collections.deque(),
        'fov_map': fov_map,
        'recompute_fov': recompute_fov,
    }


if __name__ == '__main__':
    SCREEN_WIDTH = 80
    SCREEN_HEIGHT = 50
    PANEL_HEIGHT = 2
    LIMIT_FPS = 60
    TRANSPARENT_BG_COLOR = tcod.peach
    font_path = os.path.join('fonts', 'dejavu16x16_gs_tc.png')
    font_settings = tcod.FONT_TYPE_GREYSCALE | tcod.FONT_LAYOUT_TCOD
    game_title = 'Dose Response'
    tcod.console_set_custom_font(font_path, font_settings)
    tcod.console_init_root(SCREEN_WIDTH, SCREEN_HEIGHT, game_title, False)
    tcod.sys_set_fps(LIMIT_FPS)
    consoles = initialise_consoles(10, SCREEN_WIDTH, SCREEN_HEIGHT, TRANSPARENT_BG_COLOR)
    background_conlole = tcod.console_new(SCREEN_WIDTH, SCREEN_HEIGHT)
    game_state = initial_state(SCREEN_WIDTH, SCREEN_HEIGHT - PANEL_HEIGHT)
    while not tcod.console_is_window_closed():
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
        if not game_state:
            break
        fade = game_state.get('fade', 1)
        for con in consoles[:-5]:
            tcod.console_blit(con, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0, 0, 0, fade)
        for con in consoles[-5:]:
            tcod.console_blit(con, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0, 0, 0, 1)
        tcod.console_flush()
