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
    monsters: HashMap<(int, int), uint>,
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

    fn index<P: Point>(&self, pos: P) -> uint {
        let (x, y) = pos.coordinates();
        assert!(x >= 0 && y >= 0 && x < self.width && y < self.height);
        (y * self.width + x) as uint
    }

    pub fn cell<P: Point>(&self, pos: P) -> &Cell {
        let index = self.index(pos);
        &self.map[index]
    }

    fn cell_mut<P: Point>(&mut self, pos: P) -> &mut Cell {
        let index = self.index(pos);
        &mut self.map[index]
    }

    pub fn set_tile<P: Point>(&mut self, pos: P, tile: Tile) {
        self.cell_mut(pos).tile = tile;
    }

    pub fn set_monster<P: Point>(&mut self, pos: P, monster_index: uint, monster: &Monster) {
        assert!(monster.position == pos.coordinates());
        self.monsters.insert(pos.coordinates(), monster_index);
    }

    pub fn monster_on_pos<P: Point>(&self, pos: P) -> Option<uint> {
        self.monsters.get(&pos.coordinates()).map(|&ix| ix)
    }

    pub fn add_item<P: Point>(&mut self, pos: P, item: Item) {
        self.cell_mut(pos).items.push(item);
    }

    pub fn size(&self) -> (int, int) {
        (self.width, self.height)
    }

    pub fn walkable<P: Point>(&self, pos: P) -> bool {
        self.cell(pos).tile == Tile::Empty
    }

    pub fn remove_monster(&mut self, monster_index: uint, monster: &Monster) {
        if let Some(removed_index) = self.monsters.remove(&monster.position) {
            assert!(monster_index == removed_index,
                    "The monster ID removed from the level must be correspond to the monster");
        }
    }

    pub fn move_monster<P: Point>(&mut self, monster: &mut Monster, destination: P) {
        let dest = destination.coordinates();
        // There can be only one monster on each cell. Bail if the destination
        // is already occupied:
        assert!(!self.monsters.contains_key(&dest))
        if let Some(monster_index) = self.monsters.remove(&monster.position) {
            monster.position = dest;
            self.monsters.insert(dest, monster_index);
        } else {
            panic!("Moving a monster that doesn't exist");
        }
    }

    pub fn pickup_item<P: Point>(&mut self, pos: P) -> Option<Item> {
        self.cell_mut(pos).items.pop()
    }

    pub fn random_neighbour_position<T: Rng, P: Point>(&self, rng: &mut T, pos: P) -> (int, int) {
        let (x, y) = pos.coordinates();
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
            let within_bounds = x >= 0 && y >= 0 && x < self.width && y < self.height;
            if within_bounds && self.cell(pos).tile == Tile::Empty {
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
