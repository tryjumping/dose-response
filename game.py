import collections
import math
import os
from random import random, choice

import lib.libtcodpy as tcod
from lib.enum import Enum

from ecm_artemis import EntityComponentManager
from components import *


CHEATING = False


def inc(n):
    return n + 1

def dec(n):
    return n - 1

def add(n):
    return lambda increment: n + increment

def bounded_add(lower_bound, n, upper_bound=None):
    if upper_bound is None:
        return lambda increment: max(n + increment, lower_bound)
    else:
        return lambda increment: min(max(n + increment, lower_bound),
                                     upper_bound)


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
    wall_1 = tcod.dark_green
    wall_2 = tcod.green
    wall_3 = tcod.light_green


StateOfMind = Enum('StateOfMind', [
    'dead',
    'delirium_tremens',
    'severe_withdrawal',
    'withdrawal',
    'sober',
    'high',
    'very_high',
    'overdosed'
])


def distance(p1, p2):
    """
    Return distance between two points on the tile grid.
    """
    assert p1.floor == p2.floor, "Positions must be on the same floor"
    assert p1 and p2, "Must be valid positions"
    return max(abs(p1.x - p2.x), abs(p1.y - p2.y))

def precise_distance(p1, p2):
    """
    Return a distance between two points.

    The distance is based on a pythagorean calculation rather than a rough
    heuristic.
    """
    return math.sqrt((abs(p1.x - p2.x) ** 2) + (abs(p1.y - p2.y) ** 2))

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

def has_free_aps(e, required=1):
    turn = e.get(Turn)
    return turn and turn.action_points >= required

def modify_entity_attributes(e, modif):
    """
    Updates entity's attributes based on the passed modifier.
    """
    assert e.has(Attributes) and modif
    e.update(Attributes,
             state_of_mind=bounded_add(0, modif.state_of_mind),
             tolerance=bounded_add(0, modif.tolerance),
             confidence=bounded_add(0, modif.confidence),
             nerve=bounded_add(0, modif.nerve),
             will=bounded_add(0, modif.will))


def initialise_consoles(console_count, w, h, transparent_color):
    """
    Initialise the given number of new off-screen consoles and return their list.
    """
    consoles = [tcod.console_new(w, h) for _ in xrange(console_count)]
    for con in consoles:
        tcod.console_set_key_color(con, transparent_color)
    return consoles

def in_fov(x, y, fov_map, cx, cy, radius):
    """
    Return true if the position is within the field of view given by the map and
    a radius.
    """
    if not tcod.map_is_in_fov(fov_map, x, y) or radius < 1:
        return False
    distance = precise_distance(Position(x, y, 0), Position(cx, cy, 0))
    return math.floor(distance) <= radius

def tile_system(e, pos, tile, layers, fov_map, player_pos, radius):
    if not all((e.has(c) for c in (Tile, Position))):
        return
    explored = e.has(Explorable) and e.get(Explorable).explored
    if player_pos:
        px, py = player_pos.x, player_pos.y
    else:
        px, py, radius = 0, 0, 0
    if in_fov(pos.x, pos.y, fov_map, px, py, radius) or explored:
        if e.has(Explorable):
            e.set(Explorable(explored=True))
        con = layers[tile.level]
        tcod.console_set_char_background(con, pos.x, pos.y, Color.black.value)
        # Make the explored but not directly visible areas distinct
        if not in_fov(pos.x, pos.y, fov_map, px, py, radius):
            tcod.console_set_char_background(con, pos.x, pos.y, Color.dim_background.value)
        tcod.console_put_char(con, pos.x, pos.y, tile.glyph, tcod.BKGND_NONE)
        tcod.console_set_char_foreground(con, pos.x, pos.y, tile.color.value)

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

def available_destinations(pos, ecm, w, h):
    """
    Return blocks neigbouring the given position that can be walked into.
    """
    neighbor_vectors = ((-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1),
                        (0, 1), (1, 1))
    destinations = [Position(pos.x + dx, pos.y + dy, pos.floor)
                    for dx, dy in neighbor_vectors]
    return [dest for dest in destinations
            if not blocked_tile(dest, ecm) and within_rect(dest, 0, 0, w, h)]

def ai_system(e, ai, pos, ecm, player, w, h):
    if not all((e.has(c) for c in (AI, Position))):
        return
    player_pos = player.get(Position)
    player_distance = distance(pos, player_pos)
    if player_distance < 3:
        e.set(ai._replace(kind='aggressive'))
    else:
        e.set(ai._replace(kind='idle'))
    ai = e.get(AI)
    destinations = available_destinations(pos, ecm, w, h)
    if not destinations:
        dest = None
    elif ai.kind == 'aggressive':
        if neighbor_pos(player_pos, pos):
            dest = player_pos
        else:
            destinations.sort(lambda x, y: distance(x, player_pos) - distance(y, player_pos))
            dest = destinations[0]
        e.set(Attacking(player))
    elif ai.kind == 'idle':
        dest = choice(destinations)
    else:
        raise AssertionError('Unknown AI kind: "%s"' % ai.kind)

    if dest:
        e.set(MoveDestination(dest.x, dest.y, dest.floor))
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
    # TODO: add a fov_system that updates the FOV blocked status and use that
    # for a faster lookup
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

def interaction_system(e, ecm):
    if not all((e.has(c) for c in (Position, Turn))):
        return
    pos = e.get(Position)
    interactions = [entity for entity in entities_on_position(pos, ecm)
                    if entity.has(Interactive)]
    for i in interactions:
        if (i.has(Interactive) and e.has(Addicted)):
            modify_entity_attributes(e, i.get(AttributeModifier))
            ecm.remove_entity(i)

def combat_system(e, ecm):
    if not all((e.has(c) for c in (Attacking, Turn, Info))):
        return
    target = e.get(Attacking).target
    assert e != target, "%s tried to attack itself" % e
    e.remove(Attacking)
    if not has_free_aps(e) or not neighbor_pos(e.get(Position),
                                               target.get(Position)):
        return
    print "%s attacks %s" % (e, target)

    entity_spend_ap(e)
    death_reason = "Killed by %s" % e.get(Info).name
    if e.has(Monster):
        hit_effect = e.get(Monster).hit_effect
        if hit_effect == 'modify_attributes':
            assert target.has(Attributes) and e.has(AttributeModifier)
            modify_entity_attributes(target, e.get(AttributeModifier))
            if target.get(Attributes).state_of_mind <= 0:
                kill_entity(target, death_reason)
        elif hit_effect == 'stun':
            duration = 3
            if target.has(StunEffect):
                target.update(StunEffect, duration=add(duration))
            else:
                target.set(StunEffect(duration))
            kill_entity(e, "Disappeared after the attack.")
        elif hit_effect == 'panic':
            duration = 3
            if target.has(PanicEffect):
                target.update(PanicEffect, duration=add(duration))
            else:
                target.set(PanicEffect(duration))
            kill_entity(e, "Disappeared after the attack.")
        else:
            raise AssertionError('Unknown hit_effect')
    else:
        kill_entity(target, death_reason)
    if target.has(Dead) and e.has(Statistics):
        e.update(Statistics, kills=inc)

def panic_system(e, ecm, w, h):
    if not all(e.has(c) for c in (PanicEffect, Position, MoveDestination)):
        return
    panic = e.get(PanicEffect)
    if panic.duration <= 0:
        e.remove(PanicEffect)
    else:
        print "%s panics" % e
        pos = e.get(Position)
        destinations = available_destinations(pos, ecm, w, h)
        if destinations:
            dest = choice(destinations)
        else:
            dest = pos
        e.set(MoveDestination(dest.x, dest.y, dest.floor))
        e.update(PanicEffect, duration=dec)

def stun_system(e, ecm):
    if not all(e.has(c) for c in (StunEffect, Position, MoveDestination)):
        return
    stun = e.get(StunEffect)
    if stun.duration <= 0:
        e.remove(StunEffect)
    else:
        print "%s is stunned" % e
        pos = e.get(Position)
        e.set(MoveDestination(pos.x, pos.y, pos.floor))
        e.update(StunEffect, duration=dec)

def movement_system(e, ecm, w, h):
    if not all((e.has(c) for c in (Position, MoveDestination, Turn))):
        return
    pos = e.get(Position)
    dest = e.get(MoveDestination)
    e.remove(MoveDestination)
    if not has_free_aps(e):
        print "%s tried to move but has no action points" % e
        return
    if equal_pos(pos, dest):
        # The entity waits a turn
        entity_spend_ap(e)
    elif blocked_tile(dest, ecm):
        bumped_entities = [entity for entity in entities_on_position(dest, ecm)
                           if entity.has(Solid)]
        assert len(bumped_entities) < 2, "There should be at most 1 solid entity on a given position"
        if bumped_entities:
            e.set(Bump(bumped_entities[0]))
    elif not within_rect(dest, 0, 0, w, h):
        pass  # TODO: move to the next screen
    else:
        e.set(Position(dest.x, dest.y, dest.floor))
        entity_spend_ap(e)

def bump_system(e, ecm):
    if not all((e.has(c) for c in (Bump,))):
        return
    target = e.get(Bump).target
    e.remove(Bump)
    assert e != target, "%s tried to bump itself" % e
    valid_target = ((not e.has(Monster) and target.has(Monster)) or
                    (e.has(Monster) and not target.has(Monster)))
    if valid_target:
        e.set(Attacking(target))
    else:
        pass  # bumped into a wall or something else that's not interactive

def enumerate_state_of_mind(som):
    """Return an enum representing the given state of mind.
    """
    if som <= 0:
        return StateOfMind.dead
    elif som <= 5:
        return StateOfMind.delirium_tremens
    elif som <= 25:
        return StateOfMind.severe_withdrawal
    elif som <= 50:
        return StateOfMind.withdrawal
    elif som <= 55:
        return StateOfMind.sober
    elif som <= 94:
        return StateOfMind.high
    elif som <= 99:
        return StateOfMind.very_high
    else:
        return StateOfMind.overdosed

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

def gui_system(ecm, player, layers, w, h, panel_height, dt):
    attrs = player.get(Attributes)
    panel = tcod.console_new(w, panel_height)
    if CHEATING:
        stats_bar = ("%s       SoM: %s, Tolerance: %s, Confidence: %s  Will: %s  Nerve: %s" %
                     (player.get(Info).name, attrs.state_of_mind,
                              attrs.tolerance, attrs.confidence, attrs.will,
                              attrs.nerve))
    else:
        stats_bar = "%s       Will: %s" % (player.get(Info).name, attrs.will)
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

def kill_entity(e, death_reason=''):
    for ctype in (UserInput, AI, Solid, Tile, Turn):
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
            kill_entity(e, "Withdrawal shock")
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
    for e in ecm.entities(Position, MoveDestination):
        panic_system(e, ecm, w, h)
        stun_system(e, ecm)
        movement_system(e, ecm, w, h)
        bump_system(e, ecm)
        interaction_system(e, ecm)
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
        assert player.has(Attributes)
        som = player.get(Attributes).state_of_mind
        game['fov_radius'] = (4 * som + 293) / 99  # range(3, 8)
        game['recompute_fov'](game['fov_map'], player_pos.x, player_pos.y, game['fov_radius'])
    for e, pos, tile in ecm.entities(Position, Tile, include_components=True):
        tile_system(e, pos, tile, consoles, game['fov_map'], player_pos,
                    game['fov_radius'])
    game['fade'] = max(player.get(Attributes).state_of_mind / 100.0, 0.14)
    if player.has(Dead):
        game['fade'] = 2
    gui_system(ecm, player, consoles, w, h, panel_height, dt_ms)
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
    e.add(Tile(8, Color.anxiety, 'a'))
    e.add(Monster('anxiety', hit_effect='modify_attributes'))
    e.add(Info('Anxiety', "Won't give you a second of rest."))
    e.add(AttributeModifier(state_of_mind=0, tolerance=0, confidence=0, nerve=0,
                            will=-1))
    e.add(AI('idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def make_depression_monster(e):
    e.add(Tile(8, Color.depression, 'D'))
    e.add(Monster('depression', hit_effect='modify_attributes'))
    e.add(Info('Depression', "Fast and deadly. Don't let it get close."))
    e.add(AttributeModifier(state_of_mind=-10000, tolerance=0, confidence=0,
                            nerve=0, will=0))
    e.add(AI('idle'))
    e.add(Turn(action_points=0, max_aps=2, active=False, count=0))

def make_hunger_monster(e):
    e.add(Tile(8, Color.hunger, 'h'))
    e.add(Monster('hunger', hit_effect='modify_attributes'))
    e.add(Info('Hunger', ""))
    e.add(AttributeModifier(state_of_mind=-10, tolerance=0, confidence=0, nerve=0,
                            will=0))
    e.add(AI('idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def make_voices_monster(e):
    e.add(Tile(8, Color.voices, 'v'))
    e.add(Monster('voices', hit_effect='stun'))
    e.add(Info('Voices in your head', "I'm not crazy. Can't be, can I?"))
    e.add(AI('idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def make_shadows_monster(e):
    e.add(Tile(8, Color.shadows, 'S'))
    e.add(Monster('shadows', hit_effect='panic'))
    e.add(Info('Shadows', "Hey! What was that?"))
    e.add(AI('idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def initial_state(w, h, empty_ratio=0.6):
    fov_map = tcod.map_new(w, h)

    ecm = EntityComponentManager(autoregister_components=True)
    ecm.register_component_type(Position, (int, int, int), index=True)
    # TODO: register the component types here once things settled a bit
    player_x, player_y = w / 2, h / 2
    player = ecm.new_entity()
    player.add(Position(player_x, player_y, 0))
    player.add(Tile(9, Color.player, '@'))
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
            background = ecm.new_entity()
            background.add(pos)
            background.add(Tile(0, Color.empty_tile, '.'))
            explored = precise_distance(pos, player_pos) < 6
            background.add(Explorable(explored=explored))
            if equal_pos(player_pos, pos):
                pass
            elif ((type == 'dose' and not near_player(x, y))
                  or equal_pos(initial_dose_pos, pos)):
                dose = ecm.new_entity()
                dose.add(pos)
                dose.add(Tile(5, Color.dose, 'i'))
                dose.add(AttributeModifier(
                    state_of_mind = 70 + choice(range(-10, 11)),
                    tolerance = 1,
                    confidence = choice(range(0, 2)),
                    nerve = choice(range(0, 2)),
                    will = choice(range(0, 2)),
                ))
                dose.add(Explorable(explored))
                dose.add(Interactive())
            elif type == 'wall':
                color = choice((Color.wall_1, Color.wall_2, Color.wall_3))
                background.add(Tile(0, color, '#'))
                background.add(Solid())
                walkable = False
            elif type == 'monster' and not near_player(x, y):
                monster = ecm.new_entity()
                monster.add(pos)
                monster.add(Solid())
                factories = [
                    make_anxiety_monster,
                    make_depression_monster,
                    make_hunger_monster,
                    make_voices_monster,
                    make_shadows_monster,
                ]
                choice(factories)(monster)
            tcod.map_set_properties(fov_map, x, y, transparent, walkable)

    assert len(set(ecm.entities_by_component_value(Position, x=player_x, y=player_y))) > 1
    fov_radius = 3
    def recompute_fov(fov_map, x, y, radius):
        tcod.map_compute_fov(fov_map, x, y, radius, True)
    recompute_fov(fov_map, player_x, player_y, fov_radius)
    return {
        'ecm': ecm,
        'player': player,
        'empty_ratio': empty_ratio,
        'keys': collections.deque(),
        'fov_map': fov_map,
        'fov_radius': fov_radius,
        'recompute_fov': recompute_fov,
    }


def run():
    """Start the game.

    This is a blocking function that runs the main game loop.
    """
    SCREEN_WIDTH = 80
    SCREEN_HEIGHT = 50
    PANEL_HEIGHT = 2
    LIMIT_FPS = 60
    font_path = os.path.join('fonts', 'dejavu16x16_gs_tc.png')
    font_settings = tcod.FONT_TYPE_GREYSCALE | tcod.FONT_LAYOUT_TCOD
    game_title = 'Dose Response'
    tcod.console_set_custom_font(font_path, font_settings)
    tcod.console_init_root(SCREEN_WIDTH, SCREEN_HEIGHT, game_title, False)
    tcod.sys_set_fps(LIMIT_FPS)
    consoles = initialise_consoles(10, SCREEN_WIDTH, SCREEN_HEIGHT, Color.transparent.value)
    background_conlole = tcod.console_new(SCREEN_WIDTH, SCREEN_HEIGHT)
    game_state = initial_state(SCREEN_WIDTH, SCREEN_HEIGHT - PANEL_HEIGHT)
    while not tcod.console_is_window_closed():
        tcod.console_set_default_foreground(0, Color.foreground.value)
        key = tcod.console_check_for_keypress(tcod.KEY_PRESSED)
        if key.vk == tcod.KEY_NONE:
            key = None
        dt_ms = math.trunc(tcod.sys_get_last_frame_length() * 1000)
        tcod.console_clear(None)
        for con in consoles:
            tcod.console_set_default_background(con, Color.transparent.value)
            tcod.console_set_default_foreground(con, Color.foreground.value)
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
