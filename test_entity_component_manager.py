import unittest

from entity_component_manager import EntityComponentManager, Entity, Component


class Position(Component):
    def __init__(self, x, y):
        self.x = x
        self.y = y

class Velocity(Component):
    def __init__(self, dx, dy):
        self.dx = dx
        self.dy = dy

class EmptyComponent(Component):
    pass


class TestEntity(unittest.TestCase):
    def test_equality(self):
        ecm = EntityComponentManager()
        e1 = Entity(ecm, 1)
        e1_prime = Entity(ecm, 1)
        e2 = Entity(ecm, 2)
        self.assertEqual(e1, e1_prime)
        self.assertEqual(e1_prime, e1)
        self.assertNotEqual(e1, e2)
        self.assertNotEqual(e2, e1)

    def test_hashes(self):
        ecm = EntityComponentManager()
        e1 = Entity(ecm, 1)
        e2 = Entity(ecm, 1)
        self.assertEqual(hash(e1), hash(e2))


class TestEntityComponentManager(unittest.TestCase):
    def setUp(self):
        self.ecm = EntityComponentManager()

    def test_adding_entities(self):
        e1 = self.ecm.new_entity()
        e2 = self.ecm.new_entity()
        e3 = self.ecm.new_entity()
        self.assertNotEqual(e1, e2)
        self.assertNotEqual(e2, e3)
        self.assertNotEqual(e3, e1)
        entities = set(self.ecm.entities())
        self.assertEqual(len(entities), 3)
        self.assertIn(e1, entities)
        self.assertIn(e2, entities)
        self.assertIn(e3, entities)

    def test_register_component_type(self):
        self.ecm.register_component_type(Position)
        self.ecm.register_component_type(Position)
        self.ecm.register_component_type(Velocity)
        self.ecm.register_component_type(EmptyComponent)

    def test_set_component(self):
        self.ecm.register_component_type(Position)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        with self.assertRaises(TypeError):
            self.ecm.set_component(e, {'x': 1, 'y': 2})
        with self.assertRaises(ValueError):
            self.ecm.set_component(e, Velocity(10, 20))

    def test_get_component(self):
        self.ecm.register_component_type(Position)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        c = self.ecm.get_component(e, Position)
        self.assertIsInstance(c, Position)
        self.assertEqual(c.x, 10)
        self.assertEqual(c.y, 20)

    def test_remove_component(self):
        self.ecm.register_component_type(Position)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.ecm.remove_component(e, Position)
        c = self.ecm.get_component(e, Position)
        self.assertIsNone(c)

    def test_entities_with_specified_component(self):
        self.ecm.register_component_type(Position)
        self.ecm.register_component_type(Velocity)
        self.ecm.register_component_type(EmptyComponent)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        f = self.ecm.new_entity()
        self.ecm.set_component(f, Velocity(5, 5))
        g = self.ecm.new_entity()
        self.ecm.set_component(g, Position(1, 1))
        self.ecm.set_component(g, Velocity(1, 1))
        position_entities = set(self.ecm.entities(Position))
        velocity_entities = set(self.ecm.entities(Velocity))
        self.assertIn(e, position_entities)
        self.assertNotIn(f, position_entities)
        self.assertIn(g, position_entities)
        self.assertNotIn(e, velocity_entities)
        self.assertIn(f, velocity_entities)
        self.assertIn(g, velocity_entities)


class EntityHelpers(unittest.TestCase):
    def setUp(self):
        self.ecm = EntityComponentManager()
        self.ecm.register_component_type(Position)
        self.ecm.register_component_type(Velocity)
        self.ecm.register_component_type(EmptyComponent)

    def test_has_component(self):
        e = self.ecm.new_entity()
        e.set(Position(10, 20))
        self.assertTrue(e.has(Position))
        self.assertFalse(e.has(Velocity))

    def test_set_component(self):
        e = self.ecm.new_entity()
        e.set(Position(10, 20))
        e.set(Velocity(5, 5))
        pos = self.ecm.get_component(e, Position)
        self.assertIsInstance(pos, Position)
        self.assertEqual(pos.x, 10)
        self.assertEqual(pos.y, 20)
        vel = self.ecm.get_component(e, Velocity)
        self.assertIsInstance(vel, Velocity)
        self.assertEqual(vel.dx, 5)
        self.assertEqual(vel.dy, 5)

    def test_get_component(self):
        e = self.ecm.new_entity()
        e.set(Position(10, 20))
        e.set(Velocity(5, 5))
        pos = e.get(Position)
        self.assertIsInstance(pos, Position)
        self.assertEqual(pos.x, 10)
        self.assertEqual(pos.y, 20)
        vel = e.get(Velocity)
        self.assertIsInstance(vel, Velocity)
        self.assertEqual(vel.dx, 5)
        self.assertEqual(vel.dy, 5)

    def test_get_all_components(self):
        e = self.ecm.new_entity()
        e.set(Position(10, 20))
        e.set(Velocity(5, 5))
        components = list(e.components())
        self.assertEqual(len(components), 2)
        if isinstance(components[0], Position):
            pos = components[0]
            vel = components[1]
        else:
            pos = components[1]
            vel = components[0]
        self.assertIsInstance(pos, Position)
        self.assertEqual(pos.x, 10)
        self.assertEqual(pos.y, 20)
        vel = e.get(Velocity)
        self.assertIsInstance(vel, Velocity)
        self.assertEqual(vel.dx, 5)
        self.assertEqual(vel.dy, 5)

    def test_remove_component(self):
        e = self.ecm.new_entity()
        e.set(Position(10, 20))
        e.set(Velocity(5, 5))
        self.assertTrue(e.has(Position))
        self.assertTrue(e.has(Velocity))
        e.remove(Position)
        self.assertFalse(e.has(Position))
        self.assertTrue(e.has(Velocity))


if __name__ == '__main__':
    unittest.main()
