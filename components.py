from entity_component_manager import Component

class Position(Component):
    def __init__(self, x=None, y=None, floor=None):
        self.x = x
        self.y = y
        self.floor = floor

class MoveDestination(Component):
    def __init__(self, x=None, y=None, floor=None):
        self.x = x
        self.y = y
        self.floor = floor

class Tile(Component):
    def __init__(self, level, color, glyph):
        self.level = level
        self.color = color
        self.glyph = glyph

class UserInput(Component):
    pass

class Solid(Component):
    pass

class Attributes(Component):
    def __init__(self, state_of_mind, tolerance, confidence, nerve, will):
        self.state_of_mind = state_of_mind
        self.tolerance = tolerance
        self.confidence = confidence
        self.nerve = nerve
        self.will = will

class Statistics(Component):
    def __init__(self, turns, kills, doses):
        self.turns = turns
        self.kills = kills
        self.doses = doses

class Dead(Component):
    def __init__(self, reason):
        self.reason = reason

class Interactive(Component):
    pass

class Info(Component):
    def __init__(self, name, description):
        self.name = name
        self.description = description

class Monster(Component):
    def __init__(self, kind, strength):
        self.kind = kind
        self.strength = strength

class Attacking(Component):
    def __init__(self, target):
        self.target = target
