use std::collections::HashMap;
use std::rand::{mod, Rng};
use std::time::Duration;

use color::{mod, Color};
use engine::Display;
use graphics::{mod, Animation, Render};
use item::{mod, Item};
use monster::Monster;
use player::Bonus;
use point::{mod, Point};


#[deriving(Show)]
pub struct Cell {
    pub tile: Tile,
    pub items: Vec<Item>,
    pub explored: bool,
}


#[deriving(Copy, Clone, PartialEq, Rand, Show)]
pub enum TileKind {
    Empty,
    Tree,
}

#[deriving(Copy, Show)]
pub struct Tile {
    pub kind: TileKind,
    fg_color: Color,
    animation: Animation,
    animation_state: (Duration, Color),
}

impl Tile {
    pub fn new(kind: TileKind) -> Tile {
        let color = match kind {
            TileKind::Empty => color::empty_tile,
            TileKind::Tree => {
                let options = [color::tree_1, color::tree_2, color::tree_3];
                *rand::task_rng().choose(&options).unwrap()
            }
        };
        Tile {
            kind: kind,
            fg_color: color,
            animation: Animation::None,
            animation_state: (Duration::zero(), color),
        }
    }

    pub fn set_animation(&mut self, animation: Animation) {
        self.animation = animation;
        match self.animation {
            Animation::None => {}
            Animation::ForegroundCycle{from, ..} => {
                self.animation_state = (Duration::zero(), from);
            }
        }
    }

    pub fn update(&mut self, dt: Duration) {
        match self.animation {
            Animation::None => {}
            Animation::ForegroundCycle{from, to, duration} => {
                let (old_time, old_color) = self.animation_state;
                let t = old_time + dt;
                let c = Color {
                    r: old_color.r + duration.num_milliseconds() as u8,
                    g: old_color.g + duration.num_milliseconds() as u8,
                    b: old_color.b + duration.num_milliseconds() as u8,
                };
                self.animation_state = (t, c);
            }
        }
    }
}


impl Render for Tile {
    fn render(&self, _dt: Duration) -> (char, Color, Option<Color>) {
        use self::TileKind::*;
        use graphics::Animation::*;
        let glyph = match self.kind {
            Empty => '.',
            Tree => '#',
        };
        match self.animation {
            None => (glyph, self.fg_color, Option::None),
            ForegroundCycle{..} => {
                let (_, color) = self.animation_state;
                (glyph, color, Option::None)
            }
        }
    }
}


#[deriving(Copy)]
pub enum Walkability {
    WalkthroughMonsters,
    BlockingMonsters,
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
                              |_| Cell{
                                  tile: Tile::new(TileKind::Empty),
                                  items: vec![],
                                  explored: false,
                              }),
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

    pub fn nearest_dose(&self, center: Point, radius: int) -> Option<(Point, Item)> {
        let mut doses = vec![];
        for pos in point::points_within_radius(center, radius) {
            // Make sure we don't go out of bounds with self.cell(pos):
            if !self.walkable(pos, Walkability::WalkthroughMonsters) {
                continue
            }
            for &item in self.cell(pos).items.iter() {
                match item.kind {
                    item::Kind::Dose => {
                        doses.push((pos, item));
                    }
                    _ => {}
                }
            }
        }
        doses.into_iter().min_by(|&(p, _)| point::tile_distance(center, p))
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

    pub fn walkable(&self, pos: Point, walkability: Walkability) -> bool {
        let (x, y) = pos;
        let within_bounds = x >= 0 && y >= 0 && x < self.width && y < self.height;
        let walkable = match walkability {
            Walkability::WalkthroughMonsters => true,
            Walkability::BlockingMonsters => self.monster_on_pos(pos).is_none(),
        };
        within_bounds && self.cell(pos).tile.kind == TileKind::Empty && walkable
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

    pub fn random_neighbour_position<T: Rng>(&self, rng: &mut T, (x, y): Point,
                                             walkability: Walkability) -> Point {
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
            if self.walkable(pos, walkability) {
                walkables.push(pos)
            }
        }
        match rng.choose(walkables.slice(0, walkables.len())) {
            Some(&random_pos) => random_pos,
            None => (x, y)  // Nowhere to go
        }
    }

    pub fn explore(&mut self, center: Point, radius: int) {
        for (x, y) in point::points_within_radius(center, radius) {
            if x >= 0 && y >= 0 && x < self.width && y < self.height {
                self.cell_mut((x, y)).explored = true;
            }
        }
    }

    pub fn iter(&self) -> Cells {
        Cells {
            index: 0,
            width: self.width,
            inner: self.map.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> CellsMut {
        CellsMut {
            index: 0,
            width: self.width,
            inner: self.map.iter_mut(),
        }
    }

    pub fn render(&self, display: &mut Display,
                  dt: Duration,
                  ex_center: Point, ex_radius: int,
                  bonus: Bonus) {
        for ((x, y), cell) in self.iter() {
            let in_fov = point::distance((x, y), ex_center) < (ex_radius as f32);

            // Render the tile
            if in_fov {
                graphics::draw(display, dt, (x, y), &cell.tile);
            } else if cell.explored || bonus == Bonus::UncoverMap {
                // TODO: need to supply the dark bg here?
                graphics::draw(display, dt, (x, y), &cell.tile);
                for item in cell.items.iter() {
                    graphics::draw(display, dt, (x, y), item);
                }
                display.set_background(x, y, color::dim_background);
            }

            // Render the items
            if in_fov || cell.explored || bonus == Bonus::SeeMonstersAndItems || bonus == Bonus::UncoverMap {
                for item in cell.items.iter() {
                    graphics::draw(display, dt, (x, y), item);
                }
            }
        }
    }
}

pub struct CellsMut<'a> {
    index: int,
    width: int,
    inner: ::std::slice::MutItems<'a, Cell>,
}

impl<'a> Iterator<(Point, &'a mut Cell)> for CellsMut<'a> {
    fn next(&mut self) -> Option<(Point, &'a mut Cell)> {
        let (x, y) = (self.index % self.width, self.index / self.width);
        self.index += 1;
        match self.inner.next() {
            Some(cell) => {
                Some(((x, y), cell))
            }
            None => None,
        }
    }
}

pub struct Cells<'a> {
    index: int,
    width: int,
    inner: ::std::slice::Items<'a, Cell>,
}

impl<'a> Iterator<(Point, &'a Cell)> for Cells<'a> {
    fn next(&mut self) -> Option<(Point, &'a Cell)> {
        let (x, y) = (self.index % self.width, self.index / self.width);
        self.index += 1;
        match self.inner.next() {
            Some(cell) => {
                Some(((x, y), cell))
            }
            None => None,
        }
    }
}
