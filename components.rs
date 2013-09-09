use engine::{Color};

pub struct Position {x: int, y: int}
pub struct Health(int);
pub struct Tile{level: uint, glyph: char, color: Color}

pub struct GameObject {
    position: Option<Position>,
    health: Option<Health>,
    tile: Option<Tile>,
}

impl GameObject {
    pub fn new() -> GameObject {
        GameObject {
            position: None,
            health: None,
            tile: None,
        }
    }
}