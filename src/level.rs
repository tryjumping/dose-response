use blocker;
use color::{self, Color};
use item::Item;
use point;

use std::collections::HashMap;

/// Position within a level. Ensured to be always within bounds.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelPosition {
    pos: point::Point,
}

impl Into<point::Point> for LevelPosition {
    fn into(self) -> point::Point {
        self.pos
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cell {
    pub tile: Tile,
    pub items: Vec<Item>,
    pub explored: bool,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TileKind {
    Empty,
    Tree,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub kind: TileKind,
    pub fg_color: Color,
}

impl Tile {
    pub fn new(kind: TileKind) -> Tile {
        let color = match kind {
            TileKind::Empty => color::empty_tile,
            TileKind::Tree => color::tree_1,
        };
        Tile {
            kind,
            fg_color: color,
        }
    }

    pub fn glyph(self) -> char {
        use self::TileKind::*;
        match self.kind {
            Empty => '.',
            Tree => '#',
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    dimensions: point::Point,
    pub monsters: HashMap<LevelPosition, usize>,
    map: Vec<Cell>,
}

impl Level {
    pub fn new(width: i32, height: i32) -> Level {
        let dimensions = (width, height).into();
        assert!(dimensions > (0, 0));
        let map_size = (width * height) as usize;
        Level {
            dimensions,
            monsters: HashMap::new(),
            map: (0..map_size)
                .map(|_| Cell {
                    tile: Tile::new(TileKind::Empty),
                    items: vec![],
                    explored: false,
                }).collect(),
        }
    }

    /// Convert a bare Point into LevelPosition. Panics when the point
    /// is not inside the level.
    pub fn level_position(&self, pos: point::Point) -> LevelPosition {
        assert!(pos.x >= 0);
        assert!(pos.y >= 0);
        assert!(pos.x < self.dimensions.x);
        assert!(pos.y < self.dimensions.y);
        LevelPosition { pos }
    }

    fn index(&self, pos: LevelPosition) -> usize {
        (pos.pos.y * self.dimensions.x + pos.pos.x) as usize
    }

    pub fn cell(&self, pos: LevelPosition) -> &Cell {
        let index = self.index(pos);
        &self.map[index]
    }

    pub fn cell_mut(&mut self, pos: LevelPosition) -> &mut Cell {
        let index = self.index(pos);
        &mut self.map[index]
    }

    pub fn set_tile(&mut self, pos: LevelPosition, tile: Tile) {
        self.cell_mut(pos).tile = tile;
    }

    pub fn set_monster(&mut self, monster_position: LevelPosition, monster_index: usize) {
        self.monsters.insert(monster_position, monster_index);
    }

    pub fn monster_on_pos(&self, pos: LevelPosition) -> Option<usize> {
        self.monsters.get(&pos).cloned()
    }

    pub fn add_item(&mut self, pos: LevelPosition, item: Item) {
        self.cell_mut(pos).items.push(item);
    }

    pub fn size(&self) -> point::Point {
        self.dimensions
    }

    pub fn walkable(&self, pos: LevelPosition, blockers: blocker::Blocker) -> bool {
        use self::TileKind::Empty;
        use blocker::Blocker;
        // We don't have the player's position here so we can't check that here.
        assert!(!blockers.contains(Blocker::PLAYER));
        let blocked_by_wall = blockers.contains(Blocker::WALL) && self.cell(pos).tile.kind != Empty;
        let blocked_by_monster =
            blockers.contains(Blocker::MONSTER) && self.monster_on_pos(pos).is_some();
        !(blocked_by_wall || blocked_by_monster)
    }

    pub fn move_monster(&mut self, monster_position: LevelPosition, destination: LevelPosition) {
        // There can be only one monster on each cell. Bail if the destination
        // is already occupied:
        if self.monsters.contains_key(&destination) {
            panic!(
                "Trying to move monster from {:?} to {:?}, but that's \
                 already occupied.",
                monster_position, destination
            );
        } else if let Some(monster_index) = self.monsters.remove(&monster_position) {
            self.monsters.insert(destination, monster_index);
        } else {
            panic!("Moving a monster that doesn't exist");
        }
    }

    pub fn iter(&self) -> Cells {
        Cells {
            index: 0,
            width: self.dimensions.x,
            inner: self.map.iter(),
        }
    }
}

pub struct Cells<'a> {
    index: i32,
    width: i32,
    inner: ::std::slice::Iter<'a, Cell>,
}

impl<'a> Iterator for Cells<'a> {
    type Item = (LevelPosition, &'a Cell);

    fn next(&mut self) -> Option<(LevelPosition, &'a Cell)> {
        let pos = (self.index % self.width, self.index / self.width).into();
        let level_position = LevelPosition { pos };
        self.index += 1;
        match self.inner.next() {
            Some(cell) => Some((level_position, cell)),
            None => None,
        }
    }
}
