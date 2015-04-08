use std::collections::HashMap;
use std::time::Duration;

use rand::{self, Rng};

use color::{self, Color};
use graphics::{self, Animation, Render};
use item::{self, Item};
use monster::Monster;
use point::{self, Point};


#[derive(Debug)]
pub struct Cell {
    pub tile: Tile,
    pub items: Vec<Item>,
    pub explored: bool,
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TileKind {
    Empty,
    Tree,
}

#[derive(Copy, Clone, Debug)]
pub struct Tile {
    pub kind: TileKind,
    fg_color: Color,
    animation: Animation,
    animation_state: (Duration, Color, FadeDirection),
}

#[derive(Copy, Clone, Debug)]
enum FadeDirection {
    Forward,
    Backward,
}

impl Tile {
    pub fn new(kind: TileKind) -> Tile {
        let color = match kind {
            TileKind::Empty => color::empty_tile,
            TileKind::Tree => {
                let options = [color::tree_1, color::tree_2, color::tree_3];
                *rand::thread_rng().choose(&options).unwrap()
            }
        };
        Tile {
            kind: kind,
            fg_color: color,
            animation: Animation::None,
            animation_state: (Duration::zero(), color, FadeDirection::Forward),
        }
    }

    pub fn set_animation(&mut self, animation: Animation) {
        self.animation = animation;
        match self.animation {
            Animation::None => {}
            Animation::ForegroundCycle{from, ..} => {
                self.animation_state = (Duration::zero(), from, FadeDirection::Forward);
            }
        }
    }

    pub fn update(&mut self, dt: Duration) {
        match self.animation {
            Animation::None => {}
            Animation::ForegroundCycle{from, to, duration} => {
                let (old_time, _color, old_direction) = self.animation_state;
                let mut t = old_time + dt;
                let mut direction = old_direction;

                if t > duration {
                    t = Duration::zero();
                    direction = match direction {
                        FadeDirection::Forward => FadeDirection::Backward,
                        FadeDirection::Backward => FadeDirection::Forward,
                    };
                }

                let progress = t.num_milliseconds() as f32 / duration.num_milliseconds() as f32;
                let c = match direction {
                    FadeDirection::Forward => graphics::fade_color(from, to, progress),
                    FadeDirection::Backward => graphics::fade_color(to, from, progress),
                };
                self.animation_state = (t, c, direction);
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
                let (_, color, _) = self.animation_state;
                (glyph, color, Option::None)
            }
        }
    }
}


#[derive(Copy, Clone)]
pub enum Walkability {
    WalkthroughMonsters,
    BlockingMonsters,
}


pub struct Level {
    width: i32,
    height: i32,
    pub monsters: HashMap<Point, usize>,
    map: Vec<Cell>,
}

impl Level {
    pub fn new(width: i32, height: i32) -> Level {
        assert!(width > 0 && height > 0);
        let map_size = (width * height) as usize;
        Level {
            width: width,
            height: height,
            monsters: HashMap::new(),
            map: (0..map_size).map(|_| Cell{
                tile: Tile::new(TileKind::Empty),
                items: vec![],
                explored: false,
            }).collect(),
        }
    }

    fn index(&self, (x, y): Point) -> usize {
        assert!(x >= 0 && y >= 0 && x < self.width && y < self.height);
        (y * self.width + x) as usize
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

    pub fn set_monster(&mut self, pos: Point, monster_index: usize, monster: &Monster) {
        assert!(monster.position == pos);
        self.monsters.insert(pos, monster_index);
    }

    pub fn nearest_dose(&self, center: Point, radius: i32) -> Option<(Point, Item)> {
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

    pub fn monster_on_pos(&self, pos: Point) -> Option<usize> {
        self.monsters.get(&pos).map(|&ix| ix)
    }

    pub fn add_item(&mut self, pos: Point, item: Item) {
        self.cell_mut(pos).items.push(item);
    }

    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub fn within_bounds(&self, (x, y): Point) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn walkable(&self, pos: Point, walkability: Walkability) -> bool {
        let walkable = match walkability {
            Walkability::WalkthroughMonsters => true,
            Walkability::BlockingMonsters => self.monster_on_pos(pos).is_none(),
        };
        self.within_bounds(pos) && self.cell(pos).tile.kind == TileKind::Empty && walkable
    }

    pub fn remove_monster(&mut self, monster_index: usize, monster: &Monster) {
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
        match rng.choose(&walkables) {
            Some(&random_pos) => random_pos,
            None => (x, y)  // Nowhere to go
        }
    }

    pub fn explore(&mut self, center: Point, radius: i32) {
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
}

pub struct CellsMut<'a> {
    index: i32,
    width: i32,
    inner: ::std::slice::IterMut<'a, Cell>,
}

impl<'a> Iterator for CellsMut<'a> {
    type Item = (Point, &'a mut Cell);

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
    index: i32,
    width: i32,
    inner: ::std::slice::Iter<'a, Cell>,
}

impl<'a> Iterator for Cells<'a> {
    type Item = (Point, &'a Cell);

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
