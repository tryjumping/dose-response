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

    def add(self, component):
        return self._ecm.add_component(self, component)

    def update(self, component):
        return self._ecm.update_component(self, component)

    def set(self, component):
        return self._ecm.set_component(self, component)

    def get(self, ctype):
        return self._ecm.get_component(self, ctype)

    def components(self):
        return self._ecm.components(self)

    def remove(self, ctype):
        return self._ecm.remove_component(self, ctype)


text = str
entity = Entity


def table_from_ctype(ctype):
    return ctype.__name__.lower() + '_components'

sqltype = {
        bool: 'INTEGER',
        int: 'INTEGER',
        float: 'REAL',
        text: 'TEXT',
        entity: 'INTEGER',
    }

def is_component_type(ctype):
    return (hasattr(ctype, '_fields') and hasattr(ctype, '_make') and
            hasattr(ctype, '_asdict'))

def is_component(c):
    return is_component_type(c.__class__)

def valid_type(t):
    return t in sqltype

def component_types(component):
    return [getattr(component, field).__class__ for field in component._fields]

class EntityComponentManager(object):

    def __init__(self, autoregister_components=False):
        self._autoregister = autoregister_components
        self._con = sqlite3.connect(':memory:')
        self._con.text_factory = text
        with self._con:
            self._con.executescript(
                'create table entities(id INTEGER PRIMARY KEY);')
        self._components = {}
        self._insert_sql_statement = {}

    def new_entity(self):
        cur = self._con.cursor()
        cur.execute("insert into entities values (null)")
        id = cur.lastrowid
        self._con.commit()
        return Entity(self, id)

    def remove_entity(self, entity):
        with self._con:
            self._con.execute('delete from entities where id=?', (entity._id,))

    def register_component_type(self, ctype, types):
        if not is_component_type(ctype):
            raise TypeError('The type must be a valid component type')
        sql = '''
        create table %s(
            entity_id INTEGER,
            %s
            FOREIGN KEY(entity_id) REFERENCES entities(id));
        '''
        if ctype in self._components:
            return
        if not all((valid_type(t) for t in types)):
            raise TypeError(
                'The component types must be bool, int, float, text or entity')
        attr_statements = ['%s %s,' % (field, sqltype[type])
                           for field, type
                           in zip(ctype._fields, types)]
        table = table_from_ctype(ctype)
        with self._con:
            self._con.execute(sql % (table, '\n'.join(attr_statements)))
        self._components[ctype] = types
        placeholders = ''.join((' , ?' for _ in ctype._fields))
        sql = 'insert into %s values({entity} %s)' % (table, placeholders)
        self._insert_sql_statement[ctype] = sql

    def add_component(self, entity, component):
        if not is_component(component):
            raise TypeError('The component must be a Component instance')
        ctype = component.__class__
        if ctype not in self._components:
            if self._autoregister:
                self.register_component_type(ctype, component_types(component))
            else:
                raise ValueError('Unknown component type. Register it before use.')
        id = entity._id
        sql = self._insert_sql_statement[ctype]
        values = [val._id if isinstance(val, Entity) else val
                  for val in component]
        with self._con:
            sql = sql.format(table=table_from_ctype(ctype), entity=id)
            self._con.execute(sql, values)

    def update_component(self, entity, component):
        if not is_component(component):
            raise TypeError('The component must be a Component instance')
        ctype = component.__class__
        if ctype not in self._components:
            if self._autoregister:
                self.register_component_type(ctype, component_types(component))
            else:
                raise ValueError('Unknown component type. Register it before use.')
        id = entity._id
        placeholders = ', '.join(['%s = ?' % key for key
                                  in component._asdict().keys()])
        sql = 'update {table} set %s where entity_id = {entity}' % placeholders
        values = [val._id if isinstance(val, Entity) else val
                  for val in component]
        with self._con:
            sql = sql.format(table=table_from_ctype(ctype), entity=id)
            self._con.execute(sql, values)


    def set_component(self, entity, component):
        if not is_component(component):
            raise TypeError('The component must be a Component instance')
        ctype = component.__class__
        if ctype not in self._components:
            if self._autoregister:
                self.register_component_type(ctype, component_types(component))
            else:
                raise ValueError('Unknown component type. Register it before use.')
        id = entity._id
        existing_component = self.get_component(entity, ctype)
        if entity.has(ctype):
            self.update_component(entity, component)
        else:
            self.add_component(entity, component)

    def get_component(self, entity, ctype):
        if not is_component_type(ctype):
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
            types = self._components[ctype]
            values = values[1:]  # The first item is the entity_id foreign key
            # If any of the component attributes are entities, wrap them in Entity
            values = [Entity(self, val) if type == Entity else val
                      for val, type in zip(values, types)]
            return ctype._make(values)

    def remove_component(self, entity, ctype):
        if not is_component_type(ctype):
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
                result.append(ctype._make(c[1:]))
        return result

    def entities(self, ctype=None):
        if not ctype:
            return (Entity(self, id) for (id,)
                    in self._con.execute("select id from entities"))
        if not is_component_type(ctype):
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
