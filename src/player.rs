

use color::{self, Color};
use formula::{self, ANXIETIES_PER_WILL, SOBRIETY_COUNTER, WILL, WITHDRAWAL};
use graphics::Render;
use item::Item;
use point::Point;
use ranged_int::Ranged;
use std::fmt::{Display, Error, Formatter};
use time::Duration;


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Modifier {
    Death,
    // TODO: probably rename `state_of_mind` to something like hunger
    // or satiation or maybe split it in two. It's a bit confusing now
    // since the two users of this are the Food item (which never
    // increases past Sober) and Hunger (which is only negative and
    // works even when high).
    Attribute { will: i32, state_of_mind: i32 },
    Intoxication {
        state_of_mind: i32,
        tolerance_increase: i32,
    },
    Panic(i32),
    Stun(i32),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Mind {
    Withdrawal(Ranged),
    Sober(Ranged),
    High(Ranged),
}

impl Mind {
    pub fn is_high(&self) -> bool {
        match self {
            &Mind::High(_) => true,
            _ => false,
        }
    }
}

impl Display for Mind {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::Mind::*;
        let s = match *self {
            Withdrawal(_) => "Withdrawal",
            Sober(_) => "Sober",
            High(_) => "High",
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
    pub mind: Mind,
    pub will: Ranged,
    pub tolerance: i32,
    pub panic: Ranged,
    pub stun: Ranged,

    pub pos: Point,
    pub inventory: Vec<Item>,
    pub anxiety_counter: Ranged,
    pub bonus: Bonus,
    /// How many turns after max Will to achieve victory
    pub sobriety_counter: Ranged,
    pub current_high_streak: i32,
    pub longest_high_streak: i32,

    dead: bool,
    pub invincible: bool,

    // TODO: Use a Ranged here?
    max_ap: i32,
    ap: i32,
}

impl Player {
    pub fn new(pos: Point, invincible: bool) -> Player {
        Player {
            mind: Mind::Withdrawal(Ranged::new_max(WITHDRAWAL)),
            will: Ranged::new(2, WILL),
            tolerance: 0,
            panic: Ranged::new_min(formula::PANIC_TURNS),
            stun: Ranged::new_min(formula::STUN_TURNS),
            pos,
            inventory: vec![],
            anxiety_counter: Ranged::new_min(ANXIETIES_PER_WILL),
            dead: false,
            invincible,
            max_ap: 1,
            ap: 1,
            bonus: Bonus::None,
            sobriety_counter: Ranged::new_min(SOBRIETY_COUNTER),
            current_high_streak: 0,
            longest_high_streak: 0,
        }
    }

    pub fn move_to(&mut self, new_position: Point) {
        self.pos = new_position;
    }

    pub fn ap(&self) -> i32 {
        self.ap
    }

    pub fn spend_ap(&mut self, count: i32) {
        assert!(count <= self.ap);
        self.ap -= count;
    }

    pub fn has_ap(&self, count: i32) -> bool {
        self.ap >= count
    }

    pub fn new_turn(&mut self) {
        if self.alive() {
            self.stun -= 1;
            self.panic -= 1;
            self.mind = formula::mind_take_turn(self.mind);
            self.ap = self.max_ap;
        }
    }

    pub fn alive(&self) -> bool {
        let dead_mind = match self.mind {
            Mind::Withdrawal(val) if val.is_min() => true,  // Exhausted
            Mind::High(val) if val.is_max() => true,  // Overdosed
            _ => false,
        };
        self.invincible || !self.dead && *self.will > 0 && !dead_mind
    }

    pub fn take_effect(&mut self, effect: Modifier) {
        use self::Modifier::*;
        match effect {
            Death => self.dead = true,
            Attribute { will, state_of_mind } => {
                self.will += will;
                if !self.will.is_max() {
                    self.sobriety_counter.set_to_min();
                }
                self.mind = formula::process_hunger(self.mind, state_of_mind);
            }
            Intoxication {
                state_of_mind,
                tolerance_increase,
            } => {
                self.mind = formula::intoxicate(self.mind,
                                                self.tolerance,
                                                state_of_mind);
                self.tolerance += tolerance_increase;
                self.sobriety_counter.set_to_min();
            }
            Panic(turns) => {
                self.panic += turns;
            }
            Stun(turns) => {
                self.stun += turns;
            }
        }

        if let Some(bonus) = formula::mind_bonus(self.mind) {
            // TODO: this could disable the stronger bonus if you
            // first got UncoverMap and after that
            // SeeMonstersAndItems. We need to fix that.
            self.bonus = bonus;
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
