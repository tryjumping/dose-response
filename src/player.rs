use std::cmp;

use color::{mod, Color};
use item::Item;
use graphics::Render;
use point::Point;
use ranged_int::RangedInt;


#[deriving(PartialEq, Show)]
pub enum Modifier {
    Death,
    Attribute{will: int, state_of_mind: int},
    Intoxication{state_of_mind: int, tolerance_increase: int},
    Panic(int),
    Stun(int),
}

pub struct Player {
    pub state_of_mind: RangedInt<int>,
    pub will: RangedInt<int>,
    pub tolerance: int,
    intoxication_threshold: int,
    pub panic: RangedInt<int>,
    pub stun: RangedInt<int>,

    pub pos: Point,
    pub inventory: Vec<Item>,
    pub anxiety_counter: RangedInt<int>,

    dead: bool,

    max_ap: int,
    ap: int,
}

impl Player {

    pub fn new(pos: Point) -> Player {
        Player {
            state_of_mind: RangedInt::new(20, (0, 100)),
            will: RangedInt::new(2, (0, 10)),
            tolerance: 0,
            intoxication_threshold: 20,
            panic: RangedInt::new(0, (0, 100)),
            stun: RangedInt::new(0, (0, 100)),
            pos: pos,
            inventory: vec![],
            anxiety_counter: RangedInt::new(0, (0, 10)),
            dead: false,
            max_ap: 1,
            ap: 1,
        }
    }

    pub fn move_to(&mut self, new_position: Point) {
        self.pos = new_position;
    }

    pub fn spend_ap(&mut self, count: int) {
        assert!(count <= self.ap);
        self.ap -= count;
    }

    pub fn has_ap(&self, count: int) -> bool {
        self.ap >= count
    }

    pub fn new_turn(&mut self) {
        self.stun.add(-1);
        self.panic.add(-1);
        self.state_of_mind.add(-1);
        self.ap = self.max_ap;
    }

    pub fn alive(&self) -> bool {
        !self.dead && *self.will > 0 && *self.state_of_mind > 0 && *self.state_of_mind < 100
    }

    pub fn take_effect(&mut self, effect: Modifier) {
        use self::Modifier::*;
        println!("Player was affected by: {}", effect);
        match effect {
            Death => self.dead = true,
            Attribute{will, state_of_mind} => {
                self.will.add(will);
                // NOTE: this is a bit complicated because we want to make sure
                // that don't get intoxicated by this. It should be a no-op,
                // then. But we want to get you fully satiated even if that
                // means using only a part of the value and also, any negative
                // effects should be used in full.
                let to_add = if self.intoxication_threshold > *self.state_of_mind {
                    cmp::min(state_of_mind, self.intoxication_threshold - *self.state_of_mind)
                } else {
                    0
                };
                if state_of_mind > 0 {
                    self.state_of_mind.add(to_add);
                } else {
                    self.state_of_mind.add(state_of_mind);
                }
            }
            Intoxication{state_of_mind, tolerance_increase} => {
                let state_of_mind_bonus = cmp::max(10, (state_of_mind - self.tolerance));
                self.state_of_mind.add(state_of_mind_bonus);
                self.tolerance += tolerance_increase;
            }
            Panic(turns) => {
                self.panic.add(turns);
            }
            Stun(turns) => {
                self.stun.add(turns);
            }
        }
    }
}


impl Drop for Player {
    // Implementing Drop to prevent Player from being Copy:
    fn drop(&mut self) {}
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
