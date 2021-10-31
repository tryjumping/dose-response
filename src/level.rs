use crate::{blocker, color::Color, graphic::Graphic, item::Item, palette::Palette, point};

use std::collections::HashMap;

use crate::point::Point;
use serde::{Deserialize, Serialize};

/// Position within a level. Ensured to be always within bounds.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelPosition {
    pos: point::Point,
}

impl From<LevelPosition> for Point {
    fn from(lp: LevelPosition) -> Point {
        lp.pos
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cell {
    pub tile: Tile,
    pub items: Vec<Item>,
    pub explored: bool,
    pub always_visible: bool,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TileKind {
    Empty,
    Tree,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub kind: TileKind,
    pub graphic: Graphic,
    pub color_index: usize,
}

impl Tile {
    pub fn new(kind: TileKind) -> Tile {
        let graphic = match kind {
            TileKind::Empty => Graphic::Ground1,
            TileKind::Tree => Graphic::Tree1,
        };
        Tile {
            kind,
            graphic,
            color_index: 0,
        }
    }

    pub fn color(&self, palette: &Palette) -> Color {
        match self.kind {
            TileKind::Empty => match self.graphic {
                Graphic::Ground2 => palette.empty_tile_ground,
                Graphic::Ground3 => palette.empty_tile_ground,
                Graphic::Ground5 => palette.empty_tile_ground,
                Graphic::Twigs1 => palette.empty_tile_twigs,
                Graphic::Twigs2 => palette.empty_tile_twigs,
                Graphic::Twigs3 => palette.empty_tile_twigs,
                Graphic::Twigs4 => palette.empty_tile_twigs,
                Graphic::Twigs5 => palette.empty_tile_twigs,
                Graphic::Twigs6 => palette.empty_tile_twigs,
                Graphic::Twigs7 => palette.empty_tile_twigs,
                Graphic::Twigs8 => palette.empty_tile_twigs,
                Graphic::Twigs9 => palette.empty_tile_twigs,
                Graphic::Twigs10 => palette.empty_tile_twigs,
                Graphic::Grass1 => palette.empty_tile_leaves,
                Graphic::Grass2 => palette.empty_tile_leaves,
                Graphic::Grass3 => palette.empty_tile_leaves,
                Graphic::Grass4 => palette.empty_tile_leaves,
                Graphic::Grass5 => palette.empty_tile_leaves,
                Graphic::Grass6 => palette.empty_tile_leaves,
                Graphic::Grass7 => palette.empty_tile_leaves,
                Graphic::Grass8 => palette.empty_tile_leaves,
                Graphic::Grass9 => palette.empty_tile_leaves,
                Graphic::Leaves1 => palette.empty_tile_leaves,
                Graphic::Leaves3 => palette.empty_tile_leaves,
                Graphic::Leaves4 => palette.empty_tile_leaves,
                Graphic::Leaves5 => palette.empty_tile_leaves,
                _ => palette.empty_tile_ground,
            },
            TileKind::Tree => palette.tree(self.color_index),
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
                    always_visible: false,
                })
                .collect(),
        }
    }

    /// Convert a bare Point into `LevelPosition`. Panics when the point
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
        self.monsters.get(&pos).copied()
    }

    pub fn add_item(&mut self, pos: LevelPosition, item: Item) {
        self.cell_mut(pos).items.push(item);
    }

    pub fn size(&self) -> point::Point {
        self.dimensions
    }

    pub fn walkable(&self, pos: LevelPosition, blockers: blocker::Blocker) -> bool {
        use self::TileKind::Empty;
        use crate::blocker::Blocker;
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
            log::error!(
                "Trying to move monster from {:?} to {:?}, but that's \
                 already occupied.",
                monster_position,
                destination
            );
            log::info!("Monster will not be moved.");
        } else if let Some(monster_index) = self.monsters.remove(&monster_position) {
            self.monsters.insert(destination, monster_index);
        } else {
            log::error!("Trying to move a monster that doesn't exist");
        }
    }

    pub fn iter(&self) -> Cells<'_> {
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
        self.inner.next().map(|cell| (level_position, cell))
    }
}
