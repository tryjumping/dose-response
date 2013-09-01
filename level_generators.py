from random import random

def forrest_level(w, h):
    empty_ratio = 0.6
    result = []
    for x in xrange(w):
        for y in xrange(h):
            rand = random()
            if rand < empty_ratio:
                tile_kind = 'empty'
            elif rand < 0.99:
                tile_kind = 'wall'
            else:
                tile_kind = 'dose'
                if random() < 0.23:
                    tile_kind = 'stronger_dose'
            if tile_kind == 'empty' and random() < 0.05:
                tile_kind = 'monster'
            result.append((x, y, tile_kind))
    return result
