use std::cmp;
use std::fmt::{Display, Error, Formatter};
use time::Duration;

use color::{self, Color};
use item::Item;
use graphics::Render;
use point::Point;
use ranged_int::RangedInt;


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Modifier {
    Death,
    Attribute{will: i32, state_of_mind: i32},
    Intoxication{state_of_mind: i32, tolerance_increase: i32},
    Panic(i32),
    Stun(i32),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IntoxicationState {
    Exhausted,
    DeliriumTremens,
    Withdrawal,
    Sober,
    High,
    VeryHigh,
    Overdosed,
}

impl IntoxicationState {
    pub fn from_int(value: i32) -> IntoxicationState {
        use self::IntoxicationState::*;
        match value {
            val if val <= 0 => Exhausted,
            1...5   => DeliriumTremens,
            6...15  => Withdrawal,
            16...20 => Sober,
            21...80 => High,
            81...99 => VeryHigh,
            _ => Overdosed,
        }
    }
}

impl Display for IntoxicationState {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::IntoxicationState::*;
        let s = match *self {
            Exhausted => "Exhausted",
            DeliriumTremens => "Delirium tremens",
            Withdrawal => "Withdrawal",
            Sober => "Sober",
            High => "High",
            VeryHigh => "High as a kite",
            Overdosed => "Overdosed",
        };
        f.write_str(s)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Bonus {
    None,
    SeeMonstersAndItems,
    UncoverMap,
}

pub struct Player {
    pub state_of_mind: RangedInt,
    pub will: RangedInt,
    pub tolerance: i32,
    intoxication_threshold: i32,
    pub panic: RangedInt,
    pub stun: RangedInt,

    pub pos: Point,
    pub inventory: Vec<Item>,
    pub anxiety_counter: RangedInt,
    pub bonus: Bonus,

    dead: bool,

    max_ap: i32,
    ap: i32,
}

impl Player {

    pub fn new(pos: Point) -> Player {
        Player {
            state_of_mind: RangedInt::new(20, 0, 100),
            will: RangedInt::new(2, 0, 10),
            tolerance: 0,
            intoxication_threshold: 20,
            panic: RangedInt::new(0, 0, 100),
            stun: RangedInt::new(0, 0, 100),
            pos: pos,
            inventory: vec![],
            anxiety_counter: RangedInt::new(0, 0, 10),
            dead: false,
            max_ap: 1,
            ap: 1,
            bonus: Bonus::None,
        }
    }

    pub fn move_to(&mut self, new_position: Point) {
        self.pos = new_position;
    }

    pub fn spend_ap(&mut self, count: i32) {
        assert!(count <= self.ap);
        self.ap -= count;
    }

    pub fn has_ap(&self, count: i32) -> bool {
        self.ap >= count
    }

    pub fn new_turn(&mut self) {
        self.stun -= 1;
        self.panic -= 1;
        self.state_of_mind -= 1;
        self.ap = self.max_ap;
    }

    pub fn alive(&self) -> bool {
        !self.dead && *self.will > 0 && *self.state_of_mind > 0 && *self.state_of_mind < 100
    }

    pub fn take_effect(&mut self, effect: Modifier) {
        use self::Modifier::*;
        //println!("Player was affected by: {:?}", effect);
        match effect {
            Death => self.dead = true,
            Attribute{will, state_of_mind} => {
                self.will += will;
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
                    self.state_of_mind += to_add;
                } else {
                    self.state_of_mind += state_of_mind;
                }
            }
            Intoxication{state_of_mind, tolerance_increase} => {
                let state_of_mind_bonus = cmp::max(10, (state_of_mind - self.tolerance));
                self.state_of_mind += state_of_mind_bonus;
                self.tolerance += tolerance_increase;
            }
            Panic(turns) => {
                self.panic += turns;
            }
            Stun(turns) => {
                self.stun += turns;
            }
        }
        match *self.state_of_mind {
            99 => self.bonus = Bonus::UncoverMap,
            98 => self.bonus = Bonus::SeeMonstersAndItems,
            _ => {}
        }
    }
}


impl Render for Player {
    fn render(&self, _dt: Duration) -> (char, Color, Option<Color>) {
        if self.alive() {
            ('@', color::player, None)
        } else {
            ('&', color::dead_player, None)
        }
    }
}
