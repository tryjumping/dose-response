from random import choice

from components import *
from systems.graphics import Color


def player(e):
    e.add(Tile(9, Color.player, '@'))
    e.add(UserInput())
    e.add(Info(name="The Nameless One", description=""))
    e.add(Attributes(state_of_mind=20,
                     tolerance=0,
                     confidence=5,
                     nerve=5,
                     will=2))
    e.add(Turn(action_points=1, max_aps=1, active=True, count=0))
    e.add(Statistics(turns=0, kills=0, doses=0))
    e.add(Solid())
    e.add(Addicted(resistance=0, rate_per_turn=1, turn_last_activated=0))
    e.add(KillCounter(anxieties=0, anxiety_threshold=10))
    e.add(Abilities(see_entities=False, see_world=False))
    e.add(LeaveLevel(leaving=False))

def dose(dose, pos, kind='weak'):
    dose.add(Position._make(pos))
    if kind == 'weak':
        glyph = 'i'
        som = 40
        kill_radius = 4
        irresistibility = 2
    elif kind == 'strong':
        glyph = 'I'
        som = 90
        kill_radius = 6
        irresistibility = 3
    else:
        raise ValueError('Unknown dose type: %s' % type)
    dose.add(Dose(irresistibility))
    dose.add(Tile(6, Color.dose, glyph))
    dose.add(AttributeModifier(
        state_of_mind = som + choice(range(-10, 11)),
        tolerance = 1,
        confidence = 0,
        nerve = 0,
        will = 0,
    ))
    dose.add(Explorable(False))
    dose.add(Interactive())
    dose.add(Glow(radius=0, color=Color.dose_glow))
    dose.add(KillSurroundingMonsters(radius=kill_radius))

def wall(e, kind='wall'):
    if kind == 'wall':
        color = choice((Color.wall_1, Color.wall_2, Color.wall_3))
        e.add(Tile(0, color, '#'))
        e.add(Solid())
    else:
        raise ValueError('Unknown wall type: "%s"' % kind)

def anxiety_monster(e):
    e.add(Tile(8, Color.anxiety, 'a'))
    e.add(Monster('anxiety', hit_effect='modify_attributes'))
    e.add(Info('Anxiety', "Won't give you a second of rest."))
    e.add(AttributeModifier(state_of_mind=0, tolerance=0, confidence=0, nerve=0,
                            will=-1))
    e.add(AI('individual', 'idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def depression_monster(e):
    e.add(Tile(8, Color.depression, 'D'))
    e.add(Monster('depression', hit_effect='modify_attributes'))
    e.add(Info('Depression', "Fast and deadly. Don't let it get close."))
    e.add(AttributeModifier(state_of_mind=-10000, tolerance=0, confidence=0,
                            nerve=0, will=0))
    e.add(AI('individual', 'idle'))
    e.add(Turn(action_points=0, max_aps=2, active=False, count=0))

def hunger_monster(e):
    e.add(Tile(8, Color.hunger, 'h'))
    e.add(Monster('hunger', hit_effect='modify_attributes'))
    e.add(Info('Hunger', ""))
    e.add(AttributeModifier(state_of_mind=-10, tolerance=0, confidence=0, nerve=0,
                            will=0))
    e.add(AI('pack', 'idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def voices_monster(e):
    e.add(Tile(8, Color.voices, 'v'))
    e.add(Monster('voices', hit_effect='stun'))
    e.add(Info('Voices in your head', "I'm not crazy. Can't be, can I?"))
    e.add(AI('individual', 'idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))

def shadows_monster(e):
    e.add(Tile(8, Color.shadows, 'S'))
    e.add(Monster('shadows', hit_effect='panic'))
    e.add(Info('Shadows', "Hey! What was that?"))
    e.add(AI('individual', 'idle'))
    e.add(Turn(action_points=0, max_aps=1, active=False, count=0))
