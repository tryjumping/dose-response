"""
Implementation of an Entity/Component system.
"""
import sqlite3


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

    def __repr__(self):
        return "<Entity id=%d>" % self._id

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

def table_from_ctype(ctype):
    return ctype.__name__.lower() + '_components'

class EntityComponentManager(object):

    def __init__(self, autoregister_components=False):
        self._autoregister = autoregister_components
        self._con = sqlite3.connect(':memory:')
        with self._con:
            self._con.executescript(
                'create table entities(id INTEGER PRIMARY KEY);')
        self._components = set()


    def new_entity(self):
        cur = self._con.cursor()
        cur.execute("insert into entities values (null)")
        id = cur.lastrowid
        self._con.commit()
        return Entity(self, id)

    def remove_entity(self, entity):
        with self._con:
            self._con.execute('delete from entities where id=?', (entity._id,))

    def register_component_type(self, ctype):
        if not issubclass(ctype, Component):
            raise TypeError('The type must be a Component instance')
        sql = '''
        create table %s(
            entity_id INTEGER,
            FOREIGN KEY(entity_id) REFERENCES entities(id));
        '''
        if ctype in self._components:
            return
        with self._con:
            self._con.execute(sql % table_from_ctype(ctype))
        self._components.add(ctype)

    def set_component(self, entity, component):
        if not isinstance(component, Component):
            raise TypeError('The component must be a Component instance')
        ctype = component.__class__
        if ctype not in self._components:
            if self._autoregister:
                self.register_component_type(ctype)
            else:
                raise ValueError('Unknown component type. Register it before use.')
        id = entity._id
        with self._con:
            sql = 'insert into %s values(?)'
            self._con.execute(sql % table_from_ctype(ctype), (id,))

    def get_component(self, entity, ctype):
        if not issubclass(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return None
            else:
                raise ValueError('Unknown component type. Register it before use.')
        cur = self._con.cursor()
        sql = 'select * from %s where entity_id=?'
        cur.execute(sql % table_from_ctype(ctype), (entity._id,))
        return cur.fetchone()

    def remove_component(self, entity, ctype):
        if not issubclass(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return None
            else:
                raise ValueError('Unknown component type. Register it before use.')
        with self._con:
            sql = 'delete from %s where entity_id=?'
            self._con.execute(sql % table_from_ctype(ctype), (entity._id,))

    def components(self, entity):
        cur = self._con.cursor()
        sql = 'select * from %s where entity_id=?'
        result = []
        for ctype in self._components:
            cur.execute(sql % table_from_ctype(ctype), (entity._id,))
            c = cur.fetchone()
            if c:
                result.append(c)
        return result

    def entities(self, ctype=None):
        if not ctype:
            return (Entity(self, id) for (id,) in self._con.execute("select id from entities"))
        if not issubclass(ctype, Component):
            raise TypeError('The component must be a Component instance')
        if ctype not in self._components:
            if self._autoregister:
                return ()
            else:
                raise ValueError('Unknown component type. Register it before use.')
        cur = self._con.cursor()
        sql = 'select entity_id from %s'
        cur.execute(sql % table_from_ctype(ctype))
        return (Entity(self, id) for (id,) in cur.fetchall())
