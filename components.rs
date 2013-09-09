use engine::{Color};

pub struct AcceptsUserInput;
pub struct Position {x: int, y: int}
pub struct Destination {x: int, y: int}
pub struct Health(int);
pub struct Tile{level: uint, glyph: char, color: Color}

pub struct GameObject {
    accepts_user_input: Option<AcceptsUserInput>,
    position: Option<Position>,
    destination: Option<Destination>,
    health: Option<Health>,
    tile: Option<Tile>,
}

impl GameObject {
    pub fn new() -> GameObject {
        GameObject {
            accepts_user_input: None,
            position: None,
            destination: None,
            health: None,
            tile: None,
        }
    }
}