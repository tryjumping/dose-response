use tcod;


pub struct PathFinder {
    priv tcod_path: tcod::TCOD_path_t,
}

impl PathFinder {
    pub fn new(map: tcod::TCOD_map_t, sx: int, sy: int, dx: int, dy: int)
               -> Option<PathFinder> {
        assert!(sx >= 0 && sy >= 0 && dx >= 0 && dy >= 0);
        let path = tcod::path_new_using_map(map, 1.0);
        match tcod::path_compute(path, sx, sy, dx, dy) {
            true => Some(PathFinder{tcod_path: path}),
            false => None,
        }
    }
}

impl Drop for PathFinder {
    fn drop(&self) {
        tcod::path_delete(self.tcod_path);
    }
}
