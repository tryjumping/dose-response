from components import Position, Solid

def distance(p1, p2):
    """
    Return distance between two points on the tile grid.
    """
    assert p1 and p2, "Must be valid positions"
    x1, y1 = p1
    x2, y2 = p2
    return max(abs(x1 - x2), abs(y1 - y2))

def equal_pos(p1, p2):
    """
    Return True when the two positions are equal.
    """
    return distance(p1, p2) == 0

def neighbor_pos(p1, p2):
    """
    Return True when the two position are touching.
    """
    return distance(p1, p2) <= 1

def available_destinations(pos, ecm, w, h):
    """
    Return blocks neigbouring the given position that can be walked into.
    """
    neighbor_vectors = ((-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1),
                        (0, 1), (1, 1))
    destinations = [(pos.x + dx, pos.y + dy)
                    for dx, dy in neighbor_vectors]
    return [dest for dest in destinations
            if not blocked_tile(dest, ecm) and within_rect(dest, 0, 0, w, h)]

def entities_on_position(pos, ecm):
    """
    Return all other entities with the same position.
    """
    x, y = pos
    return (entity for entity
            in ecm.entities_by_component_value(Position, x=x, y=y))


def entities_nearby(pos, radius, ecm, pred=None):
    """Return all entities within the specified radius matching the given
    predicate.
    """
    if pred is None:
        pred = lambda x: True
    ox, oy = pos
    coords_within_radius = [(x, y)
                            for x in range(ox - radius, ox + radius + 1)
                            for y in range(oy - radius, oy + radius + 1)]
    for p in coords_within_radius:
        for e in entities_on_position(p, ecm):
            if pred(e):
                yield e

def blocked_tile(pos, ecm):
    """
    True if the tile is non-empty or there's a bloking entity on it.
    """
    # TODO: add a fov_system that updates the FOV blocked status and use that
    # for a faster lookup
    return any((entity.has(Solid) for entity
                in entities_on_position(pos, ecm)))

def within_rect(pos, origin_x, origin_y, w, h):
    """
    True if the tile is within the rectangle of the specified coordinates and
    dimension.
    """
    x, y = pos
    assert origin_x <= w and origin_y <= h
    return (origin_x <= x < origin_x + w) and (origin_y <= y < origin_y + h)
