use std::vec;
use std::iter;
use tcod;
use std::libc::*;

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
    pub fn new(width: uint, height: uint) -> @mut Map {
        @mut Map{
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

    pub fn set_walkability(&mut self, pos: (int, int), walkable: Walkability) {
        match pos {
            (x, y) => self.surface[self.index_from_coords(x, y)] = walkable
        }
    }

    pub fn place_entity(&mut self, entity: int, pos: (int, int), walkable: Walkability) {
        let (x, y) = pos;
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

    pub fn is_walkable(&self, pos: (int, int)) -> bool {
        let (x, y) = pos;
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
                match to {(x, y) => self.place_entity(id, (x, y), walkable)};
            }
            _ => match self.entities_2[from_idx] {
                Some((e_id, walkable)) if e_id == id => {
                    self.entities_2[from_idx] = None;
                    match to {(x, y) => self.place_entity(id, (x, y), walkable)};
                }
                _ => fail!("Entity %? not found on position %?", id, from),
            }
        }
    }

    pub fn entities_on_pos(&self, pos: (int, int)) -> EntityIterator {
        let (x, y) = pos;
        let idx = self.index_from_coords(x, y);
        EntityIterator{e1: self.entities_1[idx], e2: self.entities_2[idx]}
    }

    pub fn find_path(@mut self, from: (int, int), to: (int, int)) -> Option<Path> {
        let (sx, sy) = from;
        let (dx, dy) = to;
        if dx < 0 || dy < 0 || dx >= self.width as int || dy >= self.height as int { return None; }
        let path = tcod::path_new_using_function(self.width as int, self.height as int,
                                                 cb, self, 1.0);
        match tcod::path_compute(path, sx, sy, dx, dy) {
            true => Some(Path{path: path}),
            false => None,
        }
    }
}

extern fn cb(xf: c_int, yf: c_int, xt: c_int, yt: c_int, path_data_ptr: *c_void) -> c_float {
    use std::cast;
    // The points should be right next to each other:
    assert!((xf, yf) != (xt, yt) && ((xf-xt) * (yf-yt)).abs() <= 1);
    let pd: &Map = unsafe { cast::transmute(path_data_ptr) };
    if pd.is_walkable((xt as  int, yt as int)) {
        1.0
    } else {
        0.0
    }
}


impl Path {
    pub fn walk(&mut self) -> Option<(int, int)> {
        match tcod::path_size(self.path) {
            0 => None,
            // Treat the destination as walkable so we always find a path to it
            // (if there is one). The user can deal with the fact that it's
            // blocked.
            1 => {
                tcod::path_walk(self.path, false); // Consume the last step
                Some(tcod::path_get_destination(self.path))
            }
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
        assert!(!map.is_walkable((0, 0)));
        assert!(!map.is_walkable((4, 4)));
    }

    #[test]
    fn test_setting_walkability() {
        let map = Map::new(5, 5);
        assert_eq!(map.is_walkable((0, 0)), false);
        assert_eq!(map.is_walkable((1, 1)), false);
        map.set_walkability((1, 1), Walkable);
        assert_eq!(map.is_walkable((0, 0)), false);
        assert_eq!(map.is_walkable((1, 1)), true);
        map.set_walkability((1, 1), Walkable);
        assert_eq!(map.is_walkable((0, 0)), false);
        assert_eq!(map.is_walkable((1, 1)), true);
        map.set_walkability((1, 1), Solid);
        assert_eq!(map.is_walkable((0, 0)), false);
        assert_eq!(map.is_walkable((1, 1)), false);
    }

    #[test]
    fn test_path_finding() {
        let map = Map::new(5, 5);
        // Add a walkable path from (0, 0) to (3, 3)
        map.set_walkability((0, 0), Walkable);
        map.set_walkability((1, 1), Walkable);
        map.set_walkability((1, 2), Walkable);
        map.set_walkability((1, 3), Walkable);
        map.set_walkability((2, 4), Walkable);
        map.set_walkability((3, 3), Walkable);
        let p = map.find_path((0, 0), (3, 3));
        assert!(p.is_some());
    }

    #[test]
    fn test_path_finding_with_blocked_destination() {
        let map = Map::new(5, 5);
        map.set_walkability((0, 0), Walkable);
        map.set_walkability((1, 1), Solid);
        let p = map.find_path((0, 0), (1, 1));
        assert!(p.is_some());
    }

    #[test]
    fn test_path_finding_with_blocked_path() {
        let map = Map::new(5, 5);
        map.set_walkability((0, 0), Walkable);
        map.set_walkability((3, 3), Walkable);
        let p = map.find_path((0, 0), (3, 3));
        assert!(p.is_none());
    }

    #[test]
    fn test_path_ref_safety() {
        let path = {
            let map = Map::new(2, 2);
            map.set_walkability((0, 0), Walkable);
            map.set_walkability((1, 1), Walkable);
            map.find_path((0,0), (1,1))
        };
        assert!(path.is_some());
        let mut p = path.unwrap();
        p.walk();
        p.walk();
        p.walk();
        p.walk();
    }
}
