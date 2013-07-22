"""
Implementation of an Entity/Component system.
"""

def system(ecm, *ctypes):
    """
    A decorator for defining systems.

    ecm: an instance of EntityComponentManager
    ctypes: component types that the system cares about

    Calling the wrapped function will automatically fetch all the entities with
    the given components and pass them to the system along with any other
    arguments.

    Define a system:

    @system(ecm, Position, Tile)
    def rendering_system(e, pos, tile, ecm, screen_width, screen_height):
        pass

    (e, pos, tile, and ecm are passed automatically, screen_width and
    screen_height are additional arguments the system requires)

    Then call it:

        rendering_system(800, 600)

    (this will pass screen_width=800 and screen_height=600)
    """
    if len(ctypes) <= 0:
        raise AttributeError('You must specify at least one component type.')
    def system_caller(fn):
        def wrapped_system(*args):
            for e in ecm.entities(*ctypes):
                components = [e.get(c) for c in ctypes]
                def prepargs(components, ecm, args):
                    for c in components:
                        yield c
                    yield ecm
                    for a in args:
                        yield a
                if all(components):
                    fn(e, *prepargs(components, ecm, args))
            return wrapped_system
        return system_caller

def is_component_type(ctype):
    return (hasattr(ctype, '_fields') and hasattr(ctype, '_make') and
            hasattr(ctype, '_asdict'))

def is_component(c):
    return is_component_type(c.__class__)

class Entity(object):
    def __init__(self, ecm, id):
        self._ecm = ecm
        self._id = id

    @property
    def id(self):
        return self._id

    @property
    def ecm(self):
        return self._ecm

    def __eq__(self, other):
        return (isinstance(other, self.__class__) and
                (self.ecm == other.ecm) and (self.id == other.id))

    def __ne__(self, other):
        return not (self == other)

    def __hash__(self):
        return hash(self.ecm) + hash(self.id)

    def __repr__(self):
        return "<Entity id=%d>" % self.id

    def has(self, ctype):
        return self.ecm.get_component(self, ctype) is not None

    def set(self, component):
        return self.ecm.set_component(self, component)

    def get(self, ctype):
        return self.ecm.get_component(self, ctype)

    def add(self, component):
        return self.set(component)

    def components(self):
        return self.ecm.components(self)

    def remove(self, ctype):
        return self.ecm.remove_component(self, ctype)

    def update(self, ctype, **kwargs):
        c = self.get(ctype)
        if c:
            new_attrs = {k : fun(getattr(c, k))
                         for k, fun in kwargs.iteritems()}
            self.set(c._replace(**new_attrs))


text = str
entity = Entity


class EntityComponentManager(object):

    def __init__(self, autoregister_components=False):
        self._entities = set()
        self._last_entity_id = -1
        self._components = {}
        self._autoregister = autoregister_components
        self._indexes = {}
        self._component_value_indexes = {}

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

    def register_component_type(self, ctype, types, index=False):
        if not is_component_type(ctype):
            raise TypeError('The type must be a Component instance')
        if ctype in self._components:
            return
        self._components[ctype] = [None] * (self._last_entity_id + 1)
        self._indexes[ctype] = set()
        if index:
            self._component_value_indexes[ctype] = {}

    def set_component(self, entity, component):
        if not is_component(component):
            raise TypeError('The component must be a Component instance')
        ctype = component.__class__
        if ctype not in self._components:
            if self._autoregister:
                self.register_component_type(ctype, None)
            else:
                raise ValueError('Unknown component type. Register it before use.')
        components = self._components[ctype]
        id = entity._id
        prev_component = self._components[ctype][id]
        components[id] = component
        self._indexes[ctype].add(id)
        if ctype in self._component_value_indexes:
            index = self._component_value_indexes[ctype]
            for key in component.__dict__.iteritems():
                if prev_component:
                    prev_key = (key[0], getattr(prev_component, key[0]))
                    if prev_key in index:
                        index[prev_key].discard(id)
                if key in index:
                    index[key].add(id)
                else:
                    index[key] = set((id,))

    def get_component(self, entity, ctype):
        if ctype in self._components:
            return self._components[ctype][entity._id]

    def remove_component(self, entity, ctype):
        if not is_component_type(ctype):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return None
            else:
                raise ValueError('Unknown component type. Register it before use.')
        component = self._components[ctype][entity._id]
        self._components[ctype][entity._id] = None
        self._indexes[ctype].discard(entity._id)
        if ctype in self._component_value_indexes and component:
            for key in component.__dict__.iteritems():
                index = self._component_value_indexes[ctype]
                if key in index:
                    index[key].discard(entity._id)

    def components(self, entity):
        id = entity._id
        return (self._components[ctype][id] for ctype
                in self._components.keys()
                if self._components[ctype][id])

    def entities_by_component_value(self, ctype, **kwargs):
        if not is_component_type(ctype):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return ()
            else:
                raise ValueError('Unknown component type. Register it before use.')
        if ctype in self._component_value_indexes:
            partial_results = []
            index = self._component_value_indexes[ctype]
            for key in kwargs.iteritems():
                if key in index and len(index[key]) > 0:
                    partial_results.append(index[key])
                else:
                    return ()
            if len(partial_results) == 0:
                return ()
            elif len(partial_results) == 1:
                result = partial_results
            else:
                result = partial_results[0].copy()
                result.intersection_update(*partial_results[1:])
            return (Entity(self, id) for id in result)
        else:
            def component_matches(c, queries):
                for k, v in queries.iteritems():
                    if getattr(c, k) != v:
                        return False
                return True
            return (Entity(self, id) for id in self._indexes[ctype]
                    if component_matches(self._components[ctype][id], kwargs))

    def build_entity_and_components(self, entity, ctypes):
        yield entity
        for ctype in ctypes:
            yield self._components[ctype][entity._id]

    def entities(self, *args, **kwargs):
        include_components = kwargs.get('include_components')
        if not args:
            return (Entity(self, id) for id in self._entities)
        for ctype in args:
            if not is_component_type(ctype):
                raise TypeError('The component must be a Component instance')
            if ctype not in self._components:
                if self._autoregister:
                    return ()
                else:
                    raise ValueError('Unknown component type. Register it before use.')
        sets = (self._indexes[ctype] for ctype in args)
        result = next(sets).copy()
        result.intersection_update(*sets)
        entities = (Entity(self, id) for id in result)
        if include_components:
            return (self.build_entity_and_components(e, args) for e in entities)
        else:
            return entities
