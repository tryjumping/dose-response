"""
Implementation of an Entity/Component system.
"""
import sqlite3


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


text = unicode
entity = Entity


class Component(object):
    """
    Defines a new ECM component. Inherit from it and specify the fields and
    their types:

        class Position(Component):
            x = int
            y = int
            floor = int

        class Monster(Component):
            kind = unicode
            strength = int

        class Attacking(Component):
            target = entity

    Allowed types: bool, int, float, text, entity

    `text` and `entity` values are provided here.

    The component is immutable, you'll have to set the entity to a new version
    when updating it.
    """
    def __init__(self, *args):
        attrs = self.attrs()
        if len(args) != len(attrs):
            raise ValueError("The number of arguments and attributes doesn't match")
        for attr, arg in zip(attrs, args):
            setattr(self, attr, arg)

    @classmethod
    def attrs(cls):
        return [k for k in cls.__dict__.keys()
                if k[:2] != '__']

    def values(self):
        return (getattr(self, attr) for attr in self.__class__.attrs())



def table_from_ctype(ctype):
    return ctype.__name__.lower() + '_components'

def sql_from_type(t):
    map = {
        bool: 'INTEGER',
        int: 'INTEGER',
        float: 'REAL',
        text: 'TEXT',
        entity: 'INTEGER',
    }
    return map[t]

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
            %s
            FOREIGN KEY(entity_id) REFERENCES entities(id));
        '''
        if ctype in self._components:
            return
        attr_statements = ['%s %s,' % (attr, sql_from_type(getattr(ctype, attr)))
                           for attr
                           in ctype.attrs()]
        with self._con:
            self._con.execute(sql % (table_from_ctype(ctype),
                                     ''.join(attr_statements)))
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
            values = [id]
            values.extend(component.values())
            sql = 'insert into %s values(%s)'
            self._con.execute(sql % (table_from_ctype(ctype),
                                     ', '.join(['?']*len(values))),
                                     values)

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
        values = cur.fetchone()
        if values:
            return ctype(*values[1:])

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
                result.append(ctype(*c[1:]))
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
