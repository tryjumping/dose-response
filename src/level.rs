use engine::{Color, Display};
use point::Point;
use world::col as color;


trait ToGlyph {
    fn to_glyph(&self) -> char;
}

trait ToColor {
    fn to_color(&self) -> Color;
}


#[deriving(PartialEq, Show)]
pub struct Cell {
    tile: Tile,
    monster: Option<Monster>,
    items: Vec<Item>,
}


#[deriving(PartialEq, Show)]
pub enum Monster {
    Anxiety,
    Depression,
    Hunger,
    Shadows,
    Voices,
}

impl ToGlyph for Monster {
    fn to_glyph(&self) -> char {
        match *self {
            Anxiety => 'a',
            Depression => 'D',
            Hunger => 'h',
            Shadows => 'S',
            Voices => 'V',
        }
    }
}

impl ToColor for Monster {
    fn to_color(&self) -> Color {
        match *self {
            Anxiety => color::anxiety,
            Depression => color::depression,
            Hunger => color::hunger,
            Shadows => color::voices,
            Voices => color::shadows,
        }
    }
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


pub struct Player {
    pos: (int, int),
}

impl Point for Player {
    fn coordinates(&self) -> (int, int) { self.pos }
}


impl ToGlyph for Player {
    fn to_glyph(&self) -> char {
        '@'
    }
}


#[deriving(PartialEq, Show)]
pub enum Item {
    Dose,
    StrongDose,
    Food,
}

impl ToGlyph for Item {
    fn to_glyph(&self) -> char {
        match *self {
            Dose => 'i',
            StrongDose => 'I',
            Food => '%',
        }
    }
}

impl ToColor for Item {
    fn to_color(&self) -> Color {
        match *self {
            Dose => color::dose,
            StrongDose => color::dose,
            Food => color::food,
        }
    }
}


pub struct Level {
    width: int,
    height: int,
    player: Player,
    map: Vec<Cell>,
}

impl Level {
    pub fn new(width: int, height: int) -> Level {
        assert!(width > 0 && height > 0);
        Level {
            width: width,
            height: height,
            player: Player{pos: (40, 25)},
            map: Vec::from_fn((width * height) as uint,
                              |_| Cell{tile: Empty, monster: None, items: vec![]}),
        }
    }

    pub fn set_tile<P: Point>(&mut self, pos: P, tile: Tile) {
        let (x, y) = pos.coordinates();
        self.map[(y * self.width + x) as uint].tile = tile;
    }

    pub fn set_monster<P: Point>(&mut self, pos: P, monster: Monster) {
        let (x, y) = pos.coordinates();
        self.map[(y * self.width + x) as uint].monster = Some(monster);
    }

    pub fn add_item<P: Point>(&mut self, pos: P, item: Item) {
        let (x, y) = pos.coordinates();
        self.map[(y * self.width + x) as uint].items.push(item);
    }

    pub fn size(&self) -> (int, int) {
        (self.width, self.height)
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn move_player<P: Point>(&mut self, new_pos: P) {
        self.player.pos = new_pos.coordinates()
    }

    pub fn render(&self, display: &mut Display) {
        let (mut x, mut y) = (0, 0);
        for cell in self.map.iter() {
            display.draw_char(x, y, cell.tile.to_glyph(), cell.tile.to_color(), color::background);
            for item in cell.items.iter() {
                display.draw_char(x, y, item.to_glyph(), item.to_color(), color::background);
            }
            if let Some(monster) = cell.monster {
                display.draw_char(x, y, monster.to_glyph(), monster.to_color(), color::background);
            }
            x += 1;
            if x >= self.width {
                x = 0;
                y += 1;
            }
        }
        let (x, y) = self.player.pos;
        display.draw_char(x, y, self.player.to_glyph(), color::player, color::background);
    }
}
