use level::ToGlyph;
use point::Point;


pub struct Player {
    pos: (int, int),
    alive: bool,
    ap: int,
    max_ap: int,
}

impl Player {

    pub fn new<P: Point>(pos: P) -> Player {
        Player {
            pos: pos.coordinates(),
            alive: true,
            ap: 1,
            max_ap: 1,
        }
    }

    pub fn move_to<P: Point>(&mut self, new_position: P) {
        self.pos = new_position.coordinates();
    }

    pub fn spend_ap(&mut self, count: int) {
        assert!(count <= self.ap);
        self.ap -= count;
    }

    pub fn has_ap(&self, count: int) -> bool {
        self.ap >= count
    }

    pub fn new_turn(&mut self) {
        self.ap = self.max_ap;
    }

    pub fn alive(&self) -> bool {
        self.alive
    }

    pub fn die(&mut self) {
        self.alive = false;
    }
}

impl Point for Player {
    fn coordinates(&self) -> (int, int) { self.pos }
}


impl ToGlyph for Player {
    fn to_glyph(&self) -> char {
        if self.alive {
            '@'
        } else {
            '&'
        }
    }
}
