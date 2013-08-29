import lib.libtcodpy as tcod


def find(fov_map, position, destination, path_cb=None):
    if path_cb:
        path = tcod.path_new_using_function(80, 50, path_cb, userdata=None, dcost=1.0)
    else:
        path = tcod.path_new_using_map(fov_map, dcost=1.0)
    tcod.path_compute(path, position.x, position.y, destination.x, destination.y)
    assert tcod.path_size(path) < 1000, "Found path is too long"
    if tcod.path_is_empty(path):
        destroy(path)
        return None
    else:
        return path

def empty(path_id):
    assert path_id is not None
    tcod.path_is_empty(path_id)

def destroy(path_id):
    assert path_id is not None
    tcod.path_delete(path_id)

def length(path_id):
    assert path_id is not None
    return tcod.path_size(path_id)
