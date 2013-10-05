use std::vec;
use std::iter;
use tcod;

struct Map {
    surface: ~[Walkability],
    // NOTE: assuming up to two entities in a single place right now.
    entities_1: ~[Option<(int, Walkability)>],
    entities_2: ~[Option<(int, Walkability)>],
    width: uint,
    height: uint,
}

struct Path {
    priv path: tcod::TCOD_path_t,
}

#[deriving(Clone, Eq)]
pub enum Walkability {
    Walkable,
    Solid,
}

struct EntityIterator {
    priv e1: Option<(int, Walkability)>,
    priv e2: Option<(int, Walkability)>,
}

impl iter::Iterator<int> for EntityIterator {
    fn next(&mut self) -> Option<int> {
        match self.e1 {
            Some((id, _)) => {
                self.e1 = self.e2;
                self.e2 = None;
                Some(id)
            }
            None => match self.e2 {
                Some((id, _)) => {
                    self.e2 = None;
                    Some(id)
                }
                None => None,
            }
        }
    }
}

impl Map {
    pub fn new(width: uint, height: uint) -> Map {
        Map{
            surface: vec::from_elem(width * height, Solid),
            entities_1: vec::from_elem(width * height, None),
            entities_2: vec::from_elem(width * height, None),
            width: width,
            height: height,
        }
    }

    fn index_from_coords(&self, x: int, y: int) -> int {
        assert!(x >= 0 && (x as uint) < self.width);
        assert!(y >= 0 && (y as uint) < self.height);
        y * (self.width as int) + x
    }

    pub fn set_walkability(&mut self, x: int, y: int, walkable: Walkability) {
        self.surface[self.index_from_coords(x, y)] = walkable;
    }

    pub fn place_entity(&mut self, x: int, y: int, entity: int, walkable: Walkability) {
        let idx = self.index_from_coords(x, y);
        // XXX: this is shit. If we ever need to support more than 2 entities/items
        // at the same place, we need to swap this for a proper data structure.
        match (self.entities_1[idx], self.entities_2[idx]) {
            (None, None) => {
                self.entities_1[idx] = Some((entity, walkable))
            }
            (None, Some((id, _))) if id == entity => {
                self.entities_2[idx] = Some((entity, walkable))
            }
            (Some((id, _)), None) if id == entity => {
                self.entities_1[idx] = Some((entity, walkable))
            }
            (None, Some(*)) => {
                self.entities_1[idx] = Some((entity, walkable))
            }
            (Some(*), None) => {
                self.entities_2[idx] = Some((entity, walkable))
            }
            (Some(*), Some(*)) => fail!("All entity slots on position %?, %? are full", x, y),
        }
    }

    pub fn is_walkable(&self, x: int, y: int) -> bool {
        let idx = self.index_from_coords(x, y);
        if self.surface[idx] == Solid { return false }
        match self.entities_1[idx] {
            Some((_, Solid)) => return false,
            _ => (),
        }
        match self.entities_2[idx] {
            Some((_, Solid)) => return false,
            _ => return true,
        }
    }

    pub fn move_entity(&mut self, id: int, from: (int, int), to: (int, int)) {
        let from_idx = match from {(x, y) => self.index_from_coords(x, y)};
        match self.entities_1[from_idx] {
            Some((e_id, walkable)) if e_id == id => {
                self.entities_1[from_idx] = None;
                match to {(x, y) => self.place_entity(x, y, id, walkable)};
            }
            _ => match self.entities_2[from_idx] {
                Some((e_id, walkable)) if e_id == id => {
                    self.entities_2[from_idx] = None;
                    match to {(x, y) => self.place_entity(x, y, id, walkable)};
                }
                _ => fail!("Entity %? not found on position %?", id, from),
            }
        }
    }

    pub fn entities_on_pos(&self, x: int, y: int) -> EntityIterator {
        let idx = self.index_from_coords(x, y);
        EntityIterator{e1: self.entities_1[idx], e2: self.entities_2[idx]}
    }

    pub fn find_path(&self, from: (int, int), to: (int, int)) -> Option<Path> {
        let cb = |xf: int, yf: int, xt: int, yt: int| {
            // The points should not be the same and should be neighbors
            assert!((xf, yf) != (xt, yt) && ((xf-xt) * (yf-yt)).abs() <= 1);
            if self.is_walkable(xt as  int, yt as int) {
                1.0
            } else if (xt, yt) == to { // Treat the destination as walkable so
                // we always find a path to it (if there is one). The user can
                // deal with the fact that it's blocked.
                1.0
            } else {
                0.0
            }
        };
        let path = tcod::path_new_using_function(self.width as int, self.height as int,
                                                 cb, 1.0);
        let (sx, sy) = from;
        let (dx, dy) = to;
        match tcod::path_compute(path, sx, sy, dx, dy) {
            true => Some(Path{path: path}),
            false => None,
        }
    }
}

impl Path {
    pub fn walk(&mut self) -> Option<(int, int)> {
        match tcod::path_size(self.path) {
            0 => None,
            _ => tcod::path_walk(self.path, true),
        }
    }
}

impl Drop for Path {
    fn drop(&mut self) {
        tcod::path_delete(self.path);
    }
}


#[cfg(test)]
mod test {
    use super::{Map, Walkable, Solid};

    #[test]
    fn test_empty_map_isnt_walkable() {
        let map = Map::new(5, 5);
        assert!(!map.is_walkable(0, 0));
        assert!(!map.is_walkable(4, 4));
    }

    #[test]
    fn test_setting_walkability() {
        let mut map = Map::new(5, 5);
        assert_eq!(map.is_walkable(0, 0), false);
        assert_eq!(map.is_walkable(1, 1), false);
        map.set_walkability(1, 1, Walkable);
        assert_eq!(map.is_walkable(0, 0), false);
        assert_eq!(map.is_walkable(1, 1), true);
        map.set_walkability(1, 1, Walkable);
        assert_eq!(map.is_walkable(0, 0), false);
        assert_eq!(map.is_walkable(1, 1), true);
        map.set_walkability(1, 1, Solid);
        assert_eq!(map.is_walkable(0, 0), false);
        assert_eq!(map.is_walkable(1, 1), false);
    }

    #[test]
    fn test_path_finding() {
        let map = Map::new(5, 5);
        let p = map.find_path((0, 0), (3, 3));
        assert!(p.is_some());
    }

}
