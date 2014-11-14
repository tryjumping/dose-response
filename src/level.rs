use engine::Display;
use point::Point;
use world::col as color;


trait ToGlyph {
    fn to_glyph(&self) -> char;
}


#[deriving(PartialEq, Clone, Rand)]
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


pub struct Player {
    pos: (int, int),
}


impl ToGlyph for Player {
    fn to_glyph(&self) -> char {
        '@'
    }
}


pub struct Level {
    width: int,
    height: int,
    player: Option<Player>,
    map: Vec<Tile>,
}

impl Level {
    pub fn new(width: int, height: int) -> Level {
        assert!(width > 0 && height > 0);
        Level {
            width: width,
            height: height,
            player: Some(Player{pos: (40, 25)}),
            map: Vec::from_elem((width * height) as uint, Empty),
        }
    }

    pub fn set_tile<P: Point>(&mut self, pos: P, tile: Tile) {
        let (x, y) = pos.coordinates();
        self.map[(y * self.width + x) as uint] = tile;
    }

    pub fn size(&self) -> (int, int) {
        (self.width, self.height)
    }

    pub fn render(&self, display: &mut Display) {
        let (mut x, mut y) = (0, 0);
        for &tile in self.map.iter() {
            display.draw_char(0, x, y, tile.to_glyph(), color::tree_1, color::background);
            x += 1;
            if x >= self.width {
                x = 0;
                y += 1;
            }
        }
        match self.player {
            Some(player) => {
                let (x, y) = player.pos;
                display.draw_char(2, x, y, player.to_glyph(), color::player, color::background);
            },
            None => {}
        }
    }
}
