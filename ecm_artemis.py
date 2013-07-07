"""
Implementation of an Entity/Component system.
"""


def is_component_type(ctype):
    return (hasattr(ctype, '_fields') and hasattr(ctype, '_make') and
            hasattr(ctype, '_asdict'))

def is_component(c):
    return is_component_type(c.__class__)

class Entity(object):
    def __init__(self, ecm, id):
        self._ecm = ecm
        self._id = id

    def __eq__(self, other):
        return (isinstance(other, self.__class__) and
                (self._ecm == other._ecm) and (self._id == other._id))

    def __hash__(self):
        return hash(self._ecm) + hash(self._id)

    def __repr__(self):
        return "<Entity id=%d>" % self._id

    def has(self, ctype):
        return self._ecm.get_component(self, ctype) is not None

    def set(self, component):
        return self._ecm.set_component(self, component)

    def get(self, ctype):
        return self._ecm.get_component(self, ctype)

    def add(self, component):
        return self.set(component)

    def components(self):
        return self._ecm.components(self)

    def remove(self, ctype):
        return self._ecm.remove_component(self, ctype)


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
        components[id] = component
        self._indexes[ctype].add(id)
        if ctype in self._component_value_indexes:
            # TODO: remove the previously indexed values
            for key in component.__dict__.iteritems():
                index = self._component_value_indexes[ctype]
                if key in index:
                    index[key].add(id)
                else:
                    index[key] = set((id,))

    def get_component(self, entity, ctype):
        if not is_component_type(ctype):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return None
            else:
                raise ValueError('Unknown component type. Register it before use.')
        return self._components[ctype][entity._id]

    def remove_component(self, entity, ctype):
        if not is_component_type(ctype):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return None
            else:
                raise ValueError('Unknown component type. Register it before use.')
        self._components[ctype][entity._id] = None
        self._indexes[ctype].remove(entity._id)
        if ctype in self._component_value_indexes:
            for key in component.__dict__.iteritems():
                index = self._component_value_indexes[ctype]
                if key in index:
                    index[key].remove(entity._id)

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
        def component_matches(c, queries):
            for k, v in queries.iteritems():
                if getattr(c, k) != v:
                    return False
            return True
        if ctype in self._component_value_indexes:
            partial_results = []
            for key in kwargs.iteritems():
                index = self._component_value_indexes[ctype]
                if key in index:
                    partial_results.append(index[key])
                else:
                    return ()
            if len(partial_results) == 0:
                return ()
            elif len(partial_results) == 1:
                result = partial_results
            else:
                result = set()
                result.update(partial_results[0])
                for p in partial_results[1:]:
                    result.intersection_update(p)
            return (Entity(self, id) for id in result)
        else:
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
        result = next(sets)
        for s in sets:
            result.intersection_update(s)
        entities = (Entity(self, id) for id in result)
        if include_components:
            return (self.build_entity_and_components(e, args) for e in entities)
        else:
            return entities
