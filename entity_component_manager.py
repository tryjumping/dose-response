"""
Implementation of an Entity/Component system.
"""


class Component(object):
    def __init__(self):
        pass


class Entity(object):
    def __init__(self, ecm, id):
        self._ecm = ecm
        self._id = id

    def __eq__(self, other):
        return (isinstance(other, self.__class__) and
                (self._ecm == other._ecm) and (self._id == other._id))

    def __hash__(self):
        return hash(self._ecm) + hash(self._id)

    def has(self, ctype):
        return self._ecm.get_component(self, ctype) is not None

    def set(self, component):
        return self._ecm.set_component(self, component)

    def get(self, ctype):
        return self._ecm.get_component(self, ctype)

    def components(self):
        return self._ecm.components(self)

    def remove(self, ctype):
        return self._ecm.remove_component(self, ctype)

class EntityComponentManager(object):

    def __init__(self, autoregister_components=False):
        self._entities = set()
        self._last_entity_id = -1
        self._components = {}
        self._autoregister = autoregister_components

    def new_entity(self):
        id = self._last_entity_id + 1
        self._entities.add(id)
        self._last_entity_id = id
        for ctype in self._components:
            self._components[ctype].append(None)
            assert self._last_entity_id == len(self._components[ctype])-1
        return Entity(self, id)

    def remove_entity(self, entity):
        id = entity._id
        for c in entity.components():
            entity.remove(c.__class__)
        self._entities.remove(id)

    def register_component_type(self, ctype):
        if not issubclass(ctype, Component):
            raise TypeError('The type must be a Component instance')
        if ctype in self._components:
            return
        self._components[ctype] = [None] * (self._last_entity_id + 1)

    def set_component(self, entity, component):
        if not isinstance(component, Component):
            raise TypeError('The component must be a Component instance')
        ctype = component.__class__
        if ctype not in self._components:
            if self._autoregister:
                self.register_component_type(ctype)
            else:
                raise ValueError('Unknown component type. Register it before use.')
        components = self._components[ctype]
        id = entity._id
        components[id] = component

    def get_component(self, entity, ctype):
        if not issubclass(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return None
            else:
                raise ValueError('Unknown component type. Register it before use.')
        return self._components[ctype][entity._id]

    def remove_component(self, entity, ctype):
        if not issubclass(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return None
            else:
                raise ValueError('Unknown component type. Register it before use.')
        self._components[ctype][entity._id] = None

    def components(self, entity):
        id = entity._id
        return (self._components[ctype][id] for ctype
                in self._components.keys()
                if self._components[ctype][id])

    def entities(self, ctype=None):
        if not ctype:
            return (Entity(self, id) for id in self._entities)
        if not issubclass(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return ()
            else:
                raise ValueError('Unknown component type. Register it before use.')
        return (Entity(self, id) for id, c in enumerate(self._components[ctype])
                if c)
