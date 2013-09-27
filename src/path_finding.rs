use tcod;


pub struct PathFinder {
    priv tcod_path: tcod::TCOD_path_t,
}

impl PathFinder {
    pub fn new(map: tcod::TCOD_map_t, sx: int, sy: int, dx: int, dy: int)
               -> Option<PathFinder> {
        let (width, height) = tcod::map_size(map);
        if dx < 0 || dy < 0 || dx >= width as int || dy >= height as int { return None }

        let path = tcod::path_new_using_map(map, 1.0);
        match tcod::path_compute(path, sx, sy, dx, dy) {
            true => Some(PathFinder{tcod_path: path}),
            false => None,
        }
    }

    pub fn walk(&mut self) -> Option<(int, int)> {
        match tcod::path_size(self.tcod_path) {
            0 => None,
            1 => {
                // Return the final point even if it's blocked. The caller will
                // handle that.
                let (x, y) = tcod::path_get_destination(self.tcod_path);
                // Consume the last step regardless of its walkability status:
                tcod::path_walk(self.tcod_path, false);
                assert!(tcod::path_is_empty(self.tcod_path));
                Some((x, y))
            },
            _ => tcod::path_walk(self.tcod_path, true)
        }
    }
}

impl Drop for PathFinder {
    fn drop(&mut self) {
        tcod::path_delete(self.tcod_path);
    }
}
