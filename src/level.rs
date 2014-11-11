use engine::Display;
use world::col as color;

#[deriving(PartialEq, Clone, Rand)]
pub enum Tile {
    Empty,
    Tree,
}


pub struct Level {
    width: int,
    height: int,
    map: Vec<Tile>,
}

impl Level {
    pub fn new(width: int, height: int) -> Level {
        assert!(width > 0 && height > 0);
        let mut map = Vec::with_capacity((width * height) as uint);
        for _ in range(0, width * height) {
            map.push(Empty);
        }
        Level {
            width: width,
            height: height,
            map: map,
        }
    }

    pub fn render(&self, display: &mut Display) {
        let (mut x, mut y) = (0, 0);
        for &tile in self.map.iter() {
            let glyph = match tile {
                Empty => '#',
                Tree => '.',
            };
            display.draw_char(0, x, y, glyph, color::tree_1, color::background);
            x += 1;
            if x >= self.width {
                x = 0;
                y += 1;
            }
        }
    }
}
