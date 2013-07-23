from collections import namedtuple
import cProfile
import itertools
from random import seed, random, choice
import sys

import ecm_artemis


Position = namedtuple('Position', 'x, y, floor')
Solid = namedtuple('Solid', '')
Tile = namedtuple('Tile', 'level, color, glyph')
Explorable = namedtuple('Explorable', 'explored')
Monster = namedtuple('Monster', 'kind')
AI = namedtuple('AI', 'kind')
MoveDestination = namedtuple('MoveDestination', 'x, y, floor')
TemporaryComponent = namedtuple('TemporaryComponent', 'x, y, z')

def initialise_map(ecm, w, h):
    for x in xrange(w):
        for y in xrange(h):
            e = ecm.new_entity()
            e.add(Position(x, y, 0))
            e.add(Solid())
            e.add(Tile(5, 10, '#'))
            e.add(Explorable(explored=False))
            if random() < 0.6:
                e = ecm.new_entity()
                e.add(Position(x, y, 0))
                e.add(Tile(6, 15, 'a'))
                e.add(Monster('anxiety'))
                e.add(AI('idle'))
                e.add(Solid())

def tile_system(e, pos, tile):
    if not all((e.has(c) for c in (Tile, Position))):
        return
    explored = e.has(Explorable) and e.get(Explorable).explored
    if explored:
        if e.has(Explorable):
            e.set(Explorable(explored=True))
    pass  # Render the tile

def blocked_tile(pos, ecm):
    entities_on_position = (e for e
                            in ecm.entities_by_component_value(Position,
                                                               x=pos.x,
                                                               y=pos.y,
                                                               floor=pos.floor))
    return any((entity.has(Solid) for entity in entities_on_position))

def ai_system(e, ai, pos, ecm, px, py):
    if not all((e.has(c) for c in (AI, Position))):
        return
    neighbor_vectors = ((-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1),
                        (0, 1), (1, 1))
    destinations = [Position(pos.x + dx, pos.y + dy, pos.floor) for dx, dy
                    in neighbor_vectors]
    player_pos = Position(px, px, pos.floor)
    if player_pos in destinations:
        dest = player_pos
    else:
        e.set(ai._replace(kind='idle'))
        destinations = [dest for dest in destinations
                        if not blocked_tile(dest, ecm)]
        if destinations:
            dest = choice(destinations)
        else:
            dest = None
    if dest:
        e.set(MoveDestination(dest.x, dest.y, dest.floor))
        e.set(ai._replace(kind='aggressive'))
    else:
        e.set(MoveDestination(pos.x, pos.y, pos.floor))


def initial_fillup_benchmark(ecm, w, h):
    initialise_map(ecm, w, h)

def entities_by_components_excluded_benchmark(ecm, count):
    for _ in xrange(count):
        for e in ecm.entities(Position, Tile):
            pass

def entities_by_components_included_benchmark(ecm, count):
    for _ in xrange(count):
        for e, pos, tile in ecm.entities(Position, Tile, include_components=True):
            pass

def tile_system_benchmark(ecm, count):
    for _ in xrange(count):
        for e, pos, tile in ecm.entities(Position, Tile, include_components=True):
            tile_system(e, pos, tile)

def ai_system_benchmark(ecm, count):
    for _ in xrange(count):
        for e, ai, pos in ecm.entities(AI, Position, include_components=True):
            ai_system(e, ai, pos, ecm, 40, 20)

def has_component_benchmark(entities, count):
    for _ in xrange(count):
        for e in entities:
            e.has(Monster)

def get_component_benchmark(entities, count):
    for _ in xrange(count):
        for e in entities:
            e.get(Monster)

def modify_component_benchmark(entities, count):
    for _ in xrange(count):
        for e in entities:
            e.set(TemporaryComponent(0, 0, 0))
            e.remove(TemporaryComponent)

def query_by_indexed_component_value_benchmark(ecm, positions, count):
    for _ in xrange(count):
        for p in positions:
            for e in ecm.entities_by_component_value(Position,
                                                     x=p.x,
                                                     y=p.y,
                                                     floor=p.floor):
                pass

if __name__ == '__main__':
    implementations = {
        'artemis': ecm_artemis.EntityComponentManager,
    }
    impl = 'artemis'
    available_benchmarks = ('primitives', 'compound', 'all')
    benchmarks = 'primitives'
    if len(sys.argv) > 1:
        if sys.argv[1] in available_benchmarks:
            benchmarks = sys.argv[1]
        else:
            print "Enter one of the available benchmarks: %s" % available_benchmarks.__repr__()
            exit(1)
    if len(sys.argv) > 2:
        if sys.argv[2] in implementations.keys():
            impl = sys.argv[2]
        else:
            print "Enter one of the available implementations: %s" % list(implementations.keys())
            exit(1)
    # Setup
    seed(3141)  # Make sure repeated runs use the same random seed
    EntityComponentManager = implementations[impl]
    game = EntityComponentManager(autoregister_components=True)
    game.register_component_type(Position, (int, int, int), index=True)
    initialise_map(game, 80, 50)
    entities = list(game.entities())[:1000]
    positions = list(itertools.islice((e.get(Position) for e
                                       in entities if e.has(Position)), 0, 10))

    if not benchmarks == 'compound':
        print '\n\nHas component benchmark:\n'
        cProfile.run('has_component_benchmark(entities, 100)', sort='cumulative')

        print '\n\nGet component benchmark:\n'
        cProfile.run('get_component_benchmark(entities, 100)', sort='cumulative')

        print '\n\nUpdate component benchmark:\n'
        cProfile.run('modify_component_benchmark(entities, 50)', sort='cumulative')

        print '\n\nEntities by components (excluded) Benchmark:\n'
        cProfile.run('entities_by_components_excluded_benchmark(game, 100)',
                     sort='cumulative')

        print '\n\nEntities by components (included) Benchmark:\n'
        cProfile.run('entities_by_components_included_benchmark(game, 100)',
                     sort='cumulative')

        print '\n\nQuery by indexed component value benchmark:\n'
        cProfile.run('query_by_indexed_component_value_benchmark(game, positions, 10000)',
                     sort='cumulative')

    if not benchmarks == 'primitives':
        print 'Initial Fillup Benchmark:\n'
        ecm = EntityComponentManager(autoregister_components=True)
        cProfile.run('initial_fillup_benchmark(ecm, 80, 50)', sort='cumulative')
        print "Expected: below 100ms"

        print '\n\nTile system benchmark:\n'
        cProfile.run('tile_system_benchmark(game, 60)', sort='cumulative')
        print "Expected: below 800ms"

        print '\n\nAI system benchmark:\n'
        cProfile.run('ai_system_benchmark(game, 60)', sort='cumulative')
        print "Expected: below 2000ms"
