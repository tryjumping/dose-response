use std::collections::HashMap;
use std::rand::Rng;

use color::{mod, Color};
use engine::Display;
use graphics::{mod, Render};
use item::Item;
use monster::Monster;
use point::Point;


#[deriving(PartialEq, Show)]
pub struct Cell {
    pub tile: Tile,
    pub items: Vec<Item>,
}


#[deriving(PartialEq, Rand, Show)]
pub enum Tile {
    Empty,
    Tree,
}


impl Render for Tile {
    fn render(&self) -> (char, Color, Color) {
        use self::Tile::*;
        let bg = color::background;
        match *self {
            Empty => ('.', color::empty_tile, bg),
            // TODO: this should be random for different tiles
            Tree => ('#', color::tree_1, bg),
        }
    }
}


pub struct Level {
    width: int,
    height: int,
    pub monsters: HashMap<Point, uint>,
    map: Vec<Cell>,
}

impl Level {
    pub fn new(width: int, height: int) -> Level {
        assert!(width > 0 && height > 0);
        Level {
            width: width,
            height: height,
            monsters: HashMap::new(),
            map: Vec::from_fn((width * height) as uint,
                              |_| Cell{tile: Tile::Empty, items: vec![]}),
        }
    }

    fn index(&self, (x, y): Point) -> uint {
        assert!(x >= 0 && y >= 0 && x < self.width && y < self.height);
        (y * self.width + x) as uint
    }

    pub fn cell(&self, pos: Point) -> &Cell {
        let index = self.index(pos);
        &self.map[index]
    }

    fn cell_mut(&mut self, pos: Point) -> &mut Cell {
        let index = self.index(pos);
        &mut self.map[index]
    }

    pub fn set_tile(&mut self, pos: Point, tile: Tile) {
        self.cell_mut(pos).tile = tile;
    }

    pub fn set_monster(&mut self, pos: Point, monster_index: uint, monster: &Monster) {
        assert!(monster.position == pos);
        self.monsters.insert(pos, monster_index);
    }

    pub fn monster_on_pos(&self, pos: Point) -> Option<uint> {
        self.monsters.get(&pos).map(|&ix| ix)
    }

    pub fn add_item(&mut self, pos: Point, item: Item) {
        self.cell_mut(pos).items.push(item);
    }

    pub fn size(&self) -> (int, int) {
        (self.width, self.height)
    }

    pub fn walkable(&self, pos: Point) -> bool {
        let (x, y) = pos;
        let within_bounds = x >= 0 && y >= 0 && x < self.width && y < self.height;
        within_bounds && self.cell(pos).tile == Tile::Empty && self.monster_on_pos(pos).is_none()
    }

    pub fn remove_monster(&mut self, monster_index: uint, monster: &Monster) {
        if let Some(removed_index) = self.monsters.remove(&monster.position) {
            assert!(monster_index == removed_index,
                    "The monster ID removed from the level must be correspond to the monster");
        }
    }

    pub fn move_monster(&mut self, monster: &mut Monster, destination: Point) {
        // There can be only one monster on each cell. Bail if the destination
        // is already occupied:
        assert!(!self.monsters.contains_key(&destination));
        if let Some(monster_index) = self.monsters.remove(&monster.position) {
            monster.position = destination;
            self.monsters.insert(destination, monster_index);
        } else {
            panic!("Moving a monster that doesn't exist");
        }
    }

    pub fn pickup_item(&mut self, pos: Point) -> Option<Item> {
        self.cell_mut(pos).items.pop()
    }

    pub fn random_neighbour_position<T: Rng>(&self, rng: &mut T, (x, y): Point) -> Point {
        let neighbors = [
            (x,   y-1),
            (x,   y+1),
            (x-1, y),
            (x+1, y),
            (x-1, y-1),
            (x+1, y-1),
            (x-1, y+1),
            (x+1, y+1),
        ];
        let mut walkables = vec![];
        for &pos in neighbors.iter() {
            let (x, y) = pos;
            if self.walkable(pos) {
                walkables.push(pos)
            }
        }
        match rng.choose(walkables.slice(0, walkables.len())) {
            Some(&random_pos) => random_pos,
            None => (x, y)  // Nowhere to go
        }
    }

    pub fn render(&self, display: &mut Display) {
        let (mut x, mut y) = (0, 0);
        for cell in self.map.iter() {
            graphics::draw(display, (x, y), &cell.tile);
            for item in cell.items.iter() {
                graphics::draw(display, (x, y), item);
            }
            x += 1;
            if x >= self.width {
                x = 0;
                y += 1;
            }
        }
    }
}
