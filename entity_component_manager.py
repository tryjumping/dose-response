"""
Implementation of an Entity/Component system.
"""


class Component(object):
    def __init__(self):
        pass



class EntityComponentManager(object):

    class Entity(object):
        def __init__(self, ecm, id):
            self._ecm = ecm
            self._id = id

        def __eq__(self, other):
            return (isinstance(other, self.__class__) and
                    (self._ecm == other._ecm) and (self._id == other._id))

    def __init__(self):
        self._entities = set()
        self._last_entity_id = 0
        self._components = {}

    def new_entity(self):
        id = self._last_entity_id + 1
        self._entities.add(id)
        self._last_entity_id = id
        return EntityComponentManager.Entity(self, id)

    def register_component_type(self, ctype):
        if not isinstance(ctype, Component):
            raise TypeError('The type must be a Component instance')
        if ctype in self._components:
            return
        self._components[ctype] = [None] * len(self._last_entity_id)

    def add_component(self, entity, component):
        ctype = component.__class__
        if not isinstance(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            raise ValueError('Unknown component type. Register it before use.')
        components = self._components[ctype]
        id = entity._id
        if len(components) <= id:
            components.extend(repeat(None, id - len(components) + 1))
        components[id] = component

    def get_component(self, entity, ctype):
        if not isinstance(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            raise ValueError('Unknown component type. Register it before use.')
        return self._components[ctype][entity._id]

    def remove_component(self, entity, ctype):
        if not isinstance(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            raise ValueError('Unknown component type. Register it before use.')
        del self._components[ctype]

    def entities(self, ctype=None):
        if not ctype:
            return (Entity(id) for id in self._entities)
        if not isinstance(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            raise ValueError('Unknown component type. Register it before use.')
        return (Entity(id) for id, c in enumerate(self._components[ctype])
                if c)
