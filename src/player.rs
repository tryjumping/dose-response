use color::{mod, Color};
use item::Item;
use level::Render;
use monster::Damage;
use point::Point;


pub struct Player {
    pub will: int,
    pub panic: int,
    pub stun: int,
    pub inventory: Vec<Item>,

    pos: (int, int),
    dead: bool,
    ap: int,
    max_ap: int,
    pub state_of_mind: int,
}

impl Player {

    pub fn new<P: Point>(pos: P) -> Player {
        Player {
            pos: pos.coordinates(),
            dead: false,
            ap: 1,
            max_ap: 1,
            state_of_mind: 20,
            will: 2,
            panic: 0,
            stun: 0,
            inventory: vec![],
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
        !self.dead && self.will > 0 && self.state_of_mind > 0
    }

    pub fn take_damage(&mut self, damage: Damage) {
        use monster::Damage::*;
        println!("Player took damage: {}", damage);
        match damage {
            Death => self.dead = true,
            AttributeLoss{will, state_of_mind} => {
                self.state_of_mind -= state_of_mind;
                self.will -= will;
            }
            Panic(turns) => self.panic += turns,
            Stun(turns) => self.stun += turns,
        }
    }
}

impl Point for Player {
    fn coordinates(&self) -> (int, int) { self.pos }
}


impl Render for Player {
    fn render(&self) -> (char, Color, Color) {
        if self.alive() {
            ('@', color::player, color::background)
        } else {
            ('&', color::dead_player, color::background)
        }
    }
}
