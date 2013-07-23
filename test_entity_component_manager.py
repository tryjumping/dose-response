from collections import namedtuple
import unittest
import sys

import ecm_artemis

Position = namedtuple('Position', 'x y')

Velocity = namedtuple('Velocity', 'dx dy')

Empty = namedtuple('Empty', [])

Attacking = namedtuple('Attacking', 'target')


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
        self.ecm.register_component_type(Position, (int, int))
        self.ecm.register_component_type(Position, (int, int))
        self.ecm.register_component_type(Velocity, (int, int))
        self.ecm.register_component_type(Empty, ())

    def test_set_component(self):
        self.ecm.register_component_type(Position, (int, int))
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        with self.assertRaises(TypeError):
            self.ecm.set_component(e, {'x': 1, 'y': 2})

    def test_get_component(self):
        self.ecm.register_component_type(Position, (int, int))
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        c = self.ecm.get_component(e, Position)
        self.assertIsInstance(c, Position)
        self.assertEqual(c.x, 10)
        self.assertEqual(c.y, 20)

    def test_update_component(self):
        self.ecm.register_component_type(Position, (int, int))
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.assertEqual(self.ecm.get_component(e, Position), Position(10, 20))
        self.ecm.set_component(e, Position(1, 1))
        self.assertEqual(self.ecm.get_component(e, Position), Position(1, 1))
        self.ecm.set_component(e, Position(2, 3))
        self.assertEqual(self.ecm.get_component(e, Position), Position(2, 3))


    def test_remove_component(self):
        self.ecm.register_component_type(Position, (int, int))
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.ecm.remove_component(e, Position)
        c = self.ecm.get_component(e, Position)
        self.assertIsNone(c)

    def test_remove_indexed_component(self):
        self.ecm.register_component_type(Position, (int, int), index=True)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.ecm.remove_component(e, Position)
        c = self.ecm.get_component(e, Position)
        self.assertIsNone(c)

    def test_remove_component_entity_doesnt_have(self):
        self.ecm.register_component_type(Position, (int, int), index=True)
        self.ecm.register_component_type(Velocity, (int, int), index=True)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.ecm.remove_component(e, Velocity)
        c = self.ecm.get_component(e, Velocity)
        self.assertIsNone(c)
        c = self.ecm.get_component(e, Position)
        self.assertIsNotNone(c)

    def test_entities_with_specified_component(self):
        self.ecm.register_component_type(Position, (int, int))
        self.ecm.register_component_type(Velocity, (int, int))
        self.ecm.register_component_type(Empty, ())
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

    def test_query_entities_with_multiple_components(self):
        self.ecm.register_component_type(Position, (int, int))
        self.ecm.register_component_type(Velocity, (int, int))
        self.ecm.register_component_type(Empty, ())
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        f = self.ecm.new_entity()
        g = self.ecm.new_entity()
        self.ecm.set_component(g, Position(1, 1))
        moving_entities = set(self.ecm.entities(Position, Velocity))
        self.assertEqual(len(moving_entities), 0)
        # Now add some velocities
        self.ecm.set_component(e, Velocity(2, 2))
        self.ecm.set_component(f, Velocity(5, 5))
        self.ecm.set_component(g, Velocity(1, 1))
        moving_entities = set(self.ecm.entities(Position, Velocity))
        self.assertEqual(len(moving_entities), 2)
        self.assertIn(e, moving_entities)
        self.assertIn(g, moving_entities)
        self.assertNotIn(f, moving_entities)

    def test_component_inclusion_when_querying_entities(self):
        self.ecm.register_component_type(Position, (int, int))
        self.ecm.register_component_type(Velocity, (int, int))
        self.ecm.register_component_type(Empty, ())
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.ecm.set_component(e, Velocity(2, 2))
        f = self.ecm.new_entity()
        self.ecm.set_component(f, Velocity(5, 5))
        g = self.ecm.new_entity()
        self.ecm.set_component(g, Velocity(1, 1))
        moving_entities = list(self.ecm.entities(Position, Velocity,
                                                 include_components=True))
        self.assertEqual(len(moving_entities), 1)
        for entity, pos, vel in moving_entities:
            self.assertEqual(entity, e)
            self.assertEqual(pos, Position(10, 20))
            self.assertEqual(vel, Velocity(2, 2))

    def test_query_entities_with_component_value(self):
        self.ecm.register_component_type(Position, (int, int))
        self.ecm.register_component_type(Velocity, (int, int))
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        f = self.ecm.new_entity()
        self.ecm.set_component(f, Position(10, 20))
        self.ecm.set_component(f, Velocity(5, 5))
        g = self.ecm.new_entity()
        self.ecm.set_component(g, Position(1, 1))
        self.ecm.set_component(g, Velocity(1, 1))
        colliding_entities = set(
            self.ecm.entities_by_component_value(Position, x=10, y=20))
        self.assertEqual(len(colliding_entities), 2)
        for entity in colliding_entities:
            self.assertEqual(self.ecm.get_component(entity, Position),
                             Position(10, 20))
        position_entities = set(self.ecm.entities(Position))
        velocity_entities = set(self.ecm.entities(Velocity))
        self.assertIn(e, colliding_entities)
        self.assertIn(f, colliding_entities)
        self.assertNotIn(g, colliding_entities)

    def test_query_entities_with_indexed_component_value(self):
        self.ecm.register_component_type(Position, (int, int), index=True)
        self.ecm.register_component_type(Velocity, (int, int), index=True)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        f = self.ecm.new_entity()
        self.ecm.set_component(f, Position(10, 20))
        self.ecm.set_component(f, Velocity(5, 5))
        g = self.ecm.new_entity()
        self.ecm.set_component(g, Position(1, 1))
        self.ecm.set_component(g, Velocity(1, 1))
        colliding_entities = set(
            self.ecm.entities_by_component_value(Position, x=10, y=20))
        self.assertEqual(len(colliding_entities), 2)
        for entity in colliding_entities:
            self.assertEqual(self.ecm.get_component(entity, Position),
                             Position(10, 20))
        position_entities = set(self.ecm.entities(Position))
        velocity_entities = set(self.ecm.entities(Velocity))
        self.assertIn(e, colliding_entities)
        self.assertIn(f, colliding_entities)
        self.assertNotIn(g, colliding_entities)

    def test_indexed_components_are_updated(self):
        self.ecm.register_component_type(Position, (int, int), index=True)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(1, 2))
        entities = list(self.ecm.entities_by_component_value(Position, x=1))
        self.assertEqual(len(entities), 1)
        self.ecm.set_component(e, Position(2, 10))
        entities = list(self.ecm.entities_by_component_value(Position, x=1))
        self.assertEqual(len(entities), 0)
        entities = list(self.ecm.entities_by_component_value(Position, x=2))
        self.assertEqual(len(entities), 1)

    def test_automatic_component_registration(self):
        self.ecm = EntityComponentManager(autoregister_components=True)
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.ecm.set_component(e, Velocity(5, 5))
        position_entities = set(self.ecm.entities(Position))
        velocity_entities = set(self.ecm.entities(Velocity))
        self.assertIn(e, position_entities)
        self.assertIn(e, velocity_entities)

    def test_remove_entity(self):
        self.ecm.register_component_type(Position, (int, int))
        self.ecm.register_component_type(Velocity, (int, int))
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.ecm.set_component(e, Velocity(5, 5))
        f = self.ecm.new_entity()
        self.assertEqual(len(set(self.ecm.entities())), 2)
        self.ecm.remove_entity(e)
        entities = set(self.ecm.entities())
        self.assertEqual(len(entities), 1)
        self.assertIn(f, entities)
        self.assertNotIn(e, entities)
        self.ecm.remove_entity(f)
        empty = set(self.ecm.entities())
        self.assertEqual(len(empty), 0)

    def test_remove_entity_with_indexed_component(self):
        self.ecm.register_component_type(Position, (int, int), index=True)
        self.ecm.register_component_type(Velocity, (int, int))
        e = self.ecm.new_entity()
        self.ecm.set_component(e, Position(10, 20))
        self.ecm.set_component(e, Velocity(5, 5))
        f = self.ecm.new_entity()
        self.assertEqual(len(set(self.ecm.entities())), 2)
        self.ecm.remove_entity(e)
        entities = set(self.ecm.entities())
        self.assertEqual(len(entities), 1)
        self.assertIn(f, entities)
        self.assertNotIn(e, entities)
        self.ecm.remove_entity(f)
        empty = set(self.ecm.entities())
        self.assertEqual(len(empty), 0)

    def test_component_with_entity_reference(self):
        self.ecm.register_component_type(Attacking, (entity,))
        e = self.ecm.new_entity()
        f = self.ecm.new_entity()
        self.ecm.set_component(f, Attacking(e))
        self.assertEqual(len(set(self.ecm.entities())), 2)
        target = self.ecm.get_component(f, Attacking).target
        self.assertEqual(e, target)


class EntityHelpers(unittest.TestCase):
    def setUp(self):
        self.ecm = EntityComponentManager()
        self.ecm.register_component_type(Position, (int, int))
        self.ecm.register_component_type(Velocity, (int, int))
        self.ecm.register_component_type(Empty, ())

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

    def test_update_component(self):
        e = self.ecm.new_entity()
        e.set(Position(10, 20))
        e.set(Velocity(5, 5))
        e.update(Position, x=lambda n: n+5, y=lambda n: n+5)
        e.update(Velocity, dx=lambda n: n/5)
        self.assertEqual(e.get(Position), Position(15, 25))
        self.assertEqual(e.get(Velocity), Velocity(1, 5))


if __name__ == '__main__':
    global Entity
    global EntityComponentManager
    global text
    global entity

    print '\n\nTesting the Artemis-like implementation:\n'
    Entity = ecm_artemis.Entity
    EntityComponentManager = ecm_artemis.EntityComponentManager
    text = ecm_artemis.text
    entity = ecm_artemis.entity

    suite = unittest.TestLoader().loadTestsFromModule(sys.modules[__name__])
    unittest.TextTestRunner().run(suite)
