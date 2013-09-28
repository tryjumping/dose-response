struct Map;

struct Path;

pub enum Walkability {
    Walkable,
    Solid,
}

struct EntityIterator;

impl Map {
    pub fn new(width: uint, height: uint) -> Map {
        Map
    }

    pub fn set_cell(&mut self, x: int, y: int, walkable: Walkability) {

    }

    pub fn place_entity(&mut self, x: int, y: int, entity: int, walkable: Walkability) {

    }

    pub fn is_walkable(&self, x: int, y: int) -> bool {
        false
    }

    pub fn entities_on_pos(&self, x: int, y: int) -> EntityIterator {
        EntityIterator
    }

    pub fn find_path(&self, from: (int, int), to: (int, int)) -> Path {
        Path
    }

    pub fn walk_path(&mut self, path: Path) -> Option<(int, int)> {
        None
    }
}
