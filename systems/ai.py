from random import choice

from components import *
import location_utils as loc
from systems import path


def ai_system(e, ai, pos, ecm, player, fov_map, w, h):
    if not all((e.has(c) for c in (AI, Position))):
        return
    player_pos = player.get(Position)
    player_distance = loc.distance(pos, player_pos)
    if player_distance < 5:
        e.set(ai._replace(kind='aggressive'))
    if player_distance > 8:
        e.set(ai._replace(kind='idle'))
    ai = e.get(AI)
    destinations = loc.available_destinations(pos, ecm, w, h)
    if not destinations:
        dest = None
    elif ai.kind == 'aggressive':
        if loc.neighbor_pos(player_pos, pos):
            dest = player_pos
            e.remove(MovePath)
        else:
            if e.has(MovePath):
                # We need to generate a new path because the player has most
                # likely moved away
                path.destroy(e.get(MovePath).id)
            def path_func(x_from, y_from, x_to, y_to, user_data):
                if (x_to, y_to) == (player_pos.x, player_pos.y):
                    # The player must be reachable for the monster, otherwise
                    # the path will be never found.
                    return 1.0
                elif loc.blocked_tile(MoveDestination(x_to, y_to), ecm):
                    return 0.0
                else:
                    return 1.0
            path_id = path.find(fov_map, pos, player_pos, path_cb=path_func)
            if path_id is not None:
                e.set(MovePath(path_id))
            dest = None
        e.set(Attacking(player))
    elif ai.kind == 'idle':
        dest = choice(destinations)
    else:
        raise AssertionError('Unknown AI kind: "%s"' % ai.kind)

    if dest:
        e.set(MoveDestination._make(dest))
    else:
        e.set(MoveDestination._make(pos))
