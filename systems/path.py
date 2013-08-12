import lib.libtcodpy as tcod


def path_find(fov_map, position, destination):
    path = tcod.path_new_using_map(fov_map, dcost=1.0)
    tcod.path_compute(path, position.x, position.y, destination.x, destination.y)
    if tcod.path_is_empty(path):
        path_destroy(path)
        return None
    else:
        return path

def path_empty(path_id):
    assert path_id is not None
    tcod.path_is_empty(path_id)

def path_destroy(path_id):
    assert path_id is not None
    tcod.path_delete(path_id)

def path_length(path_id):
    assert path_id is not None
    return tcod.path_size(path_id)
