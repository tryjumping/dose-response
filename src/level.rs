use std::collections::HashMap;
use std::rand::Rng;

use color::{mod, Color};
use engine::Display;
use item::Item;
use monster::Monster;
use player::Player;
use point::Point;


pub trait ToGlyph {
    fn to_glyph(&self) -> char;
}

pub trait ToColor {
    fn to_color(&self) -> Color;
}


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


impl ToGlyph for Tile {
    fn to_glyph(&self) -> char {
        match *self {
            Empty => '.',
            Tree => '#',
        }
    }
}

impl ToColor for Tile {
    fn to_color(&self) -> Color {
        match *self {
            Empty => color::empty_tile,
            // TODO: this should be random for different tiles
            Tree => color::tree_1,
        }
    }
}


pub struct Level {
    width: int,
    height: int,
    player: Player,
    monsters: HashMap<(int, int), Monster>,
    map: Vec<Cell>,
}

impl Level {
    pub fn new(width: int, height: int) -> Level {
        assert!(width > 0 && height > 0);
        Level {
            width: width,
            height: height,
            player: Player::new((40, 25)),
            monsters: HashMap::new(),
            map: Vec::from_fn((width * height) as uint,
                              |_| Cell{tile: Empty, items: vec![]}),
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

    pub fn set_monster<P: Point>(&mut self, pos: P, monster: Monster) {
        self.monsters.insert(pos.coordinates(), monster);
    }

    pub fn monster<P: Point>(&self, pos: P) -> Option<&Monster> {
        self.monsters.get(&pos.coordinates())
    }

    pub fn add_item<P: Point>(&mut self, pos: P, item: Item) {
        self.cell_mut(pos).items.push(item);
    }

    pub fn size(&self) -> (int, int) {
        (self.width, self.height)
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn move_player<P: Point>(&mut self, new_pos: P) {
        self.player.move_to(new_pos);
    }

    pub fn kill_monster<P: Point>(&mut self, pos: P) -> Option<Monster> {
        self.monsters.remove(&pos.coordinates())
    }

    pub fn move_monster<P: Point, Q: Point>(&mut self, from: P, to: Q) {
        // There can be only one monster on each cell. Bail if the destination
        // is already occupied:
        if self.monsters.contains_key(&to.coordinates()) {
            return
        }
        if let Some(monster) = self.monsters.remove(&from.coordinates()) {
            self.monsters.insert(to.coordinates(), monster);
        }
    }

    pub fn pickup_item<P: Point>(&mut self, pos: P) -> Option<Item> {
        self.cell_mut(pos).items.pop()
    }

    pub fn monsters(&self) -> ::std::collections::hash_map::Entries<(int, int), Monster> {
        self.monsters.iter()
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
            let within_bounds = (x >= 0 && y >= 0 && x < self.width && y < self.height);
            if within_bounds && self.cell(pos).tile == Empty {
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
            display.draw_char(x, y, cell.tile.to_glyph(), cell.tile.to_color(), color::background);
            for item in cell.items.iter() {
                display.draw_char(x, y, item.to_glyph(), item.to_color(), color::background);
            }
            x += 1;
            if x >= self.width {
                x = 0;
                y += 1;
            }
        }
        for (&(x, y), monster) in self.monsters.iter() {
            display.draw_char(x, y, monster.to_glyph(), monster.to_color(), color::background);
        }
        let (x, y) = self.player.coordinates();
        display.draw_char(x, y, self.player.to_glyph(), color::player, color::background);
    }
}
