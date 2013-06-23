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
