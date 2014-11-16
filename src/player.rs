use level::ToGlyph;
use point::Point;


pub struct Player {
    pos: (int, int),
}

impl Player {

    pub fn new<P: Point>(pos: P) -> Player {
        Player {
            pos: pos.coordinates(),
        }
    }

    pub fn move_to<P: Point>(&mut self, new_position: P) {
        self.pos = new_position.coordinates();
    }
}

impl Point for Player {
    fn coordinates(&self) -> (int, int) { self.pos }
}


impl ToGlyph for Player {
    fn to_glyph(&self) -> char {
        '@'
    }
}
