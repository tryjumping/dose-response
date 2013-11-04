use std::cast;
use std::vec;
use std::iter;
use tcod;
use std::libc::*;
use std::util::Void;

struct Map {
    surface: ~[(Walkability, Exploration)],
    // NOTE: assuming up to two entities in a single place right now.
    entities_1: ~[Option<(int, Walkability)>],
    entities_2: ~[Option<(int, Walkability)>],
    width: int,
    height: int,
}

#[deriving(Clone, Eq)]
pub enum Walkability {
    Walkable,
    Solid,
}

#[deriving(Clone, Eq)]
pub enum Exploration {
    Explored,
    Unexplored,
}

struct EntityIterator {
    priv e1: Option<(int, Walkability)>,
    priv e2: Option<(int, Walkability)>,
}

impl iter::Iterator<(int, Walkability)> for EntityIterator {
    fn next(&mut self) -> Option<(int, Walkability)> {
        match self.e1 {
            Some((id, walkability)) => {
                self.e1 = self.e2;
                self.e2 = None;
                Some((id, walkability))
            }
            None => match self.e2 {
                Some((id, walkability)) => {
                    self.e2 = None;
                    Some((id, walkability))
                }
                None => None,
            }
        }
    }
}

impl Map {
    pub fn new(width: int, height: int) -> Map {
        let cell_count = (width * height) as uint;
        Map{
            surface: vec::from_elem(cell_count, (Solid, Unexplored)),
            entities_1: vec::from_elem(cell_count, None),
            entities_2: vec::from_elem(cell_count, None),
            width: width,
            height: height,
        }
    }

    fn index_from_coords(&self, x: int, y: int) -> int {
        assert!(x >= 0 && x < self.width);
        assert!(y >= 0 && y < self.height);
        y * self.width + x
    }

    pub fn set_walkability(&mut self, pos: (int, int), walkable: Walkability) {
        match pos {
            (x, y) => {
                let idx = self.index_from_coords(x, y);
                match self.surface[idx] {
                    (_, explored) => self.surface[idx] = (walkable, explored)
                }
            }
        }
    }

    pub fn set_explored(&mut self, pos: (int, int), explored: Exploration) {
        match pos {
            (x, y) => {
                let idx = self.index_from_coords(x, y);
                match self.surface[idx] {
                    (walkable, _) => self.surface[idx] = (walkable, explored)
                }
            }
        }
    }

    pub fn is_explored(&self, pos: (int, int)) -> bool {
        let (x, y) = pos;
        let out_of_bounds = x < 0 || x >= self.width || y < 0 || y >= self.height;
        if out_of_bounds { return false }

        let idx = self.index_from_coords(x, y);
        match self.surface[idx] {
            (_, Explored) => true,
            (_, Unexplored) => false,
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
        let out_of_bounds = x < 0 || x >= self.width || y < 0 || y >= self.height;
        if out_of_bounds { return false }

        let idx = self.index_from_coords(x, y);
        match self.surface[idx] {
            (Solid, _) => return false,
            _ => {}
        }
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

    pub fn remove_entity(&mut self, id: int, pos: (int, int)) {
        let idx = match pos {(x, y) => self.index_from_coords(x, y)};
        match self.entities_1[idx] {
            Some((e_id, _walkable)) if e_id == id => {
                self.entities_1[idx] = None;
            }
            _ => match self.entities_2[idx] {
                Some((e_id, _walkable)) if e_id == id => {
                    self.entities_2[idx] = None;
                }
                _ => fail!("Entity %? not found on position %?", id, pos),
            }
        }
    }

    pub fn entities_on_pos(&self, pos: (int, int)) -> EntityIterator {
        let (x, y) = pos;
        let out_of_bounds = x < 0 || x >= self.width || y < 0 || y >= self.height;
        if out_of_bounds { return EntityIterator{e1: None, e2: None} }
        let idx = self.index_from_coords(x, y);
        EntityIterator{e1: self.entities_1[idx], e2: self.entities_2[idx]}
    }

    // TODO: a better (safer, more inclined with how Rust would work) interface
    // for this would be to return the iterator over the path points at the
    // given moment.
    // The iterator would contain an imutable borrowed pointer to the map so the
    // map wouldn't be accessible until it goes out of scope, but it's enough to:
    // 1. iterate over the path as it is now
    // 2. get the first element of the path and walk there
    // In the second case, the user would call `map.find_path` again the next
    // time the entity was supposed to move. This would be costly in the naive
    // implementation (finding the path again each time) but we could make it
    // smarter by caching/recalculating it internally. Point is, that would not
    // leak outside of `Map`.
    pub fn find_path(&self, from: (int, int), to: (int, int)) -> Option<~Path> {
        let (sx, sy) = from;
        let (dx, dy) = to;
        if dx < 0 || dy < 0 || dx >= self.width || dy >= self.height { return None; }
        let mut path_obj = ~Path{map: Handle::new(self), tcod_res: None, from: from, to: to};
        let mut path = tcod::Path::new_using_function(self.width, self.height,
                                                      cb, path_obj, 1.0);
        match path.find(sx, sy, dx, dy) {
            true => {
                path_obj.tcod_res = Some(path);
                Some(path_obj)
            }
            false => {
                None
            }
        }
    }

    fn return_handle(&self) -> Handle<Map> {
        Handle::new(self)
    }
}

struct Path {
    priv map: Handle<Map>,
    priv tcod_res: Option<tcod::Path>,
    from: (int, int),
    to: (int, int),
}

impl Path {
    pub fn walk(&mut self) -> Option<(int, int)> {
        if self.tcod_res.is_some() {
            let recalculate = true;
            self.tcod_res.get_mut_ref().walk(recalculate)
        } else {
            None
        }
    }

    pub fn len(&self) -> int {
        match self.tcod_res {
            Some(ref r) => r.len(),
            None => 0,
        }
    }
}


struct Handle<T> {
    priv ptr: *Void,
}

impl<T> Handle<T> {
    fn new(resource: &T) -> Handle<T> {
        Handle{ptr: resource as *T as *Void}
    }

    unsafe fn as_ref<'r>(&'r self) -> &'r T {
        cast::transmute(self.ptr)
    }
}

struct PathData{dx: int, dy: int, map: *mut Map}

extern fn cb(xf: c_int, yf: c_int, xt: c_int, yt: c_int, path_data_ptr: *c_void) -> c_float {
    // The points should be right next to each other:
    assert!((xf, yf) != (xt, yt) && ((xf-xt) * (yf-yt)).abs() <= 1);
    let path: &Path = unsafe { cast::transmute(path_data_ptr) };

    let (dx, dy) = path.to;
    // Succeed if we're at the destination even if it's not walkable:
    if (dx as c_int, dy as c_int) == (xt, yt) {
        1.0
    } else if unsafe { path.map.as_ref().is_walkable((xt as int, yt as int))} {
        1.0
    } else {
        0.0
    }
}


extern fn dummy_cb(_xf: c_int, _yf: c_int, _xt: c_int, _yt: c_int, _path_data_ptr: *c_void) -> c_float {
    1.0
}


#[cfg(test)]
mod test {
    use super::{Map, Walkable, Solid, Explored, Unexplored};

    #[test]
    fn handle() {
        let mut map = Map::new(5, 5);
        let h = map.return_handle();
        assert_eq!(unsafe {h.as_ref()}.is_walkable((1, 1)), false);
        map.set_walkability((1, 1), Walkable);
        assert_eq!(unsafe {h.as_ref()}.is_walkable((1, 1)), true);
    }

    #[test]
    fn empty_map_isnt_walkable() {
        let map = Map::new(5, 5);
        assert!(!map.is_walkable((0, 0)));
        assert!(!map.is_walkable((4, 4)));
    }

    #[test]
    fn setting_walkability() {
        let mut map = Map::new(5, 5);
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
    fn exploration() {
        let mut map = Map::new(5, 5);
        assert_eq!(map.is_explored((0, 0)), false);
        assert_eq!(map.is_explored((1, 1)), false);
        map.set_explored((1, 1), Explored);
        assert_eq!(map.is_explored((0, 0)), false);
        assert_eq!(map.is_explored((1, 1)), true);
        map.set_explored((1, 1), Unexplored);
        assert_eq!(map.is_explored((0, 0)), false);
        assert_eq!(map.is_explored((1, 1)), false);
    }

    #[test]
    fn path_finding() {
        let mut map = Map::new(5, 5);
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
    fn path_finding_with_blocked_destination() {
        let mut map = Map::new(5, 5);
        map.set_walkability((0, 0), Walkable);
        map.set_walkability((1, 1), Solid);
        let p = map.find_path((0, 0), (1, 1));
        assert!(p.is_some());
    }

    #[test]
    fn path_finding_with_blocked_path() {
        let mut map = Map::new(5, 5);
        map.set_walkability((0, 0), Walkable);
        map.set_walkability((3, 3), Walkable);
        let p = map.find_path((0, 0), (3, 3));
        assert!(p.is_none());
    }

    #[test]
    fn path_ref_safety() {
        let path = {
            let mut map = Map::new(2, 2);
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

    #[test]
    fn placing_entity() {
        let mut map = Map::new(2, 2);
        map.set_walkability((0, 0), Walkable);
        map.set_walkability((1, 1), Walkable);
        map.place_entity(10, (0, 0), Solid);
        assert_eq!(map.is_walkable((0, 0)), false);
        map.place_entity(11, (1, 1), Walkable);
        assert_eq!(map.is_walkable((1, 1)), true);
        map.place_entity(12, (0, 1), Walkable);
        assert_eq!(map.is_walkable((0, 1)), false);
    }

    #[test]
    fn placing_multiple_entities() {
        let mut map = Map::new(2, 2);
        map.set_walkability((0, 0), Walkable);
        map.place_entity(10, (0, 0), Walkable);
        map.place_entity(11, (0, 0), Walkable);
        assert_eq!(map.is_walkable((0, 0)), true);

        map.set_walkability((0, 1), Walkable);
        map.place_entity(12, (0, 1), Walkable);
        map.place_entity(13, (0, 1), Solid);
        assert_eq!(map.is_walkable((0, 1)), false);

        map.set_walkability((1, 0), Walkable);
        map.place_entity(14, (1, 0), Solid);
        map.place_entity(15, (1, 0), Walkable);
        assert_eq!(map.is_walkable((1, 0)), false);
    }

    #[test]
    fn update_entities_walkability() {
        let mut map = Map::new(2, 2);
        map.set_walkability((0, 0), Walkable);

        map.place_entity(10, (0, 0), Walkable);
        assert_eq!(map.is_walkable((0, 0)), true);

        map.place_entity(10, (0, 0), Solid);
        assert_eq!(map.is_walkable((0, 0)), false);

        map.place_entity(10, (0, 0), Walkable);
        assert_eq!(map.is_walkable((0, 0)), true);
    }

    #[test]
    fn move_entity() {
        let mut map = Map::new(2, 2);
        map.set_walkability((0, 0), Walkable);
        map.set_walkability((1, 1), Walkable);
        map.place_entity(10, (0, 0), Solid);
        map.place_entity(11, (0, 0), Walkable);
        assert_eq!(map.is_walkable((0, 0)), false);
        assert_eq!(map.is_walkable((1, 1)), true);

        map.move_entity(10, (0, 0), (1, 1));
        assert_eq!(map.is_walkable((0, 0)), true);
        assert_eq!(map.is_walkable((1, 1)), false);
    }

    #[test]
    #[should_fail]
    fn move_invalid_entity() {
        let mut map = Map::new(2, 2);
        map.set_walkability((0, 0), Walkable);
        map.set_walkability((1, 1), Walkable);
        map.place_entity(10, (0, 0), Solid);
        map.place_entity(11, (0, 0), Walkable);
        assert_eq!(map.is_walkable((0, 0)), false);
        assert_eq!(map.is_walkable((1, 1)), true);

        map.move_entity(12, (0, 0), (1, 1));
    }

    #[test]
    fn test_entities_on_pos() {
        let mut map = Map::new(2, 2);

        map.place_entity(10, (0, 0), Solid);
        map.place_entity(11, (0, 0), Walkable);
        assert_eq!(map.entities_on_pos((0, 0)).len(), 2);
        let mut two_entities_iterator = map.entities_on_pos((0, 0));
        assert_eq!(two_entities_iterator.next(), Some((10, Solid)));
        assert_eq!(two_entities_iterator.next(), Some((11, Walkable)));
        assert_eq!(two_entities_iterator.next(), None);

        map.place_entity(12, (0, 1), Walkable);
        assert_eq!(map.entities_on_pos((0, 1)).len(), 1);
        let mut one_entity_iterator = map.entities_on_pos((0, 1));
        assert_eq!(one_entity_iterator.next(), Some((12, Walkable)));
        assert_eq!(one_entity_iterator.next(), None);

        assert_eq!(map.entities_on_pos((1, 1)).len(), 0);
        let mut zero_entities_iterator = map.entities_on_pos((1, 1));
        assert_eq!(zero_entities_iterator.next(), None);
    }

    #[test]
    fn remove_entity() {
        let mut map = Map::new(2, 2);
        map.set_walkability((0, 0), Walkable);
        map.place_entity(10, (0, 0), Solid);
        map.place_entity(11, (0, 0), Walkable);
        assert_eq!(map.is_walkable((0, 0)), false);
        assert_eq!(map.entities_on_pos((0, 0)).len(), 2);

        map.remove_entity(10, (0, 0));
        assert_eq!(map.is_walkable((0, 0)), true);
        assert_eq!(map.entities_on_pos((0, 0)).len(), 1);

        map.remove_entity(11, (0, 0));
        assert_eq!(map.is_walkable((0, 0)), true);
        assert_eq!(map.entities_on_pos((0, 0)).len(), 0);
    }
}
