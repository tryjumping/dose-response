use std::cmp;
use std::fmt::{Display, Error, Formatter};
use time::Duration;

use color::{self, Color};
use item::Item;
use graphics::Render;
use point::Point;
use ranged_int::RangedInt;


const WILL_MAX: i32 = 5;
const ANXIETIES_PER_WILL: i32 = 7;
const WITHDRAWAL_MAX: i32 = 15;
const HIGH_MAX: i32 = 80;
const SOBER_MAX: i32 = 20;


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Modifier {
    Death,
    // TODO: probably rename `state_of_mind` to something like hunger
    // or satiation or maybe split it in two. It's a bit confusing now
    // since the two users of this are the Food item (which never
    // increases past Sober) and Hunger (which is only negative and
    // works even when high).
    Attribute{will: i32, state_of_mind: i32},
    Intoxication{state_of_mind: i32, tolerance_increase: i32},
    Panic(i32),
    Stun(i32),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Mind {
    Withdrawal(RangedInt),
    Sober(RangedInt),
    High(RangedInt),
}

impl Mind {
    pub fn update(&self) -> Self {
        use self::Mind::*;
        match *self {
            Withdrawal(value) => {
                Withdrawal(value - 1)
            }
            Sober(value) => {
                let new_value = value - 1;
                if new_value.is_min() {
                    Withdrawal(RangedInt::new(WITHDRAWAL_MAX, 0, WITHDRAWAL_MAX))
                } else {
                    Sober(new_value)
                }
            }
            High(value) => {
                let new_value = value - 1;
                if new_value.is_min() {
                    Withdrawal(RangedInt::new(WITHDRAWAL_MAX, 0, WITHDRAWAL_MAX))
                } else {
                    High(new_value)
                }
            }
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
    pub will: RangedInt,
    pub tolerance: i32,
    pub panic: RangedInt,
    pub stun: RangedInt,

    pub pos: Point,
    pub inventory: Vec<Item>,
    pub anxiety_counter: RangedInt,
    pub bonus: Bonus,
    /// How many turns after max Will to achieve victory
    pub sobriety_counter: RangedInt,

    dead: bool,

    max_ap: i32,
    ap: i32,
}

impl Player {

    pub fn new(pos: Point) -> Player {
        Player {
            mind: Mind::Withdrawal(RangedInt::new(WITHDRAWAL_MAX, 0, WITHDRAWAL_MAX)),
            will: RangedInt::new(2, 0, WILL_MAX),
            tolerance: 0,
            panic: RangedInt::new(0, 0, 100),
            stun: RangedInt::new(0, 0, 100),
            pos: pos,
            inventory: vec![],
            anxiety_counter: RangedInt::new(0, 0, ANXIETIES_PER_WILL),
            dead: false,
            max_ap: 1,
            ap: 1,
            bonus: Bonus::None,
            sobriety_counter: RangedInt::new(0, 0, 100),
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
        self.stun -= 1;
        self.panic -= 1;
        self.mind = self.mind.update();
        self.ap = self.max_ap;
    }

    pub fn alive(&self) -> bool {
        let dead_mind = match self.mind {
            Mind::Withdrawal(val) if val.is_min() => true,  // Exhausted
            Mind::High(val) if val.is_max() => true,  // Overdosed
            _ => false,
        };
        !self.dead && *self.will > 0 && !dead_mind
    }

    pub fn take_effect(&mut self, effect: Modifier) {
        use self::Modifier::*;
        match effect {
            Death => self.dead = true,
            Attribute{will, state_of_mind} => {
                self.will += will;
                if !self.will.is_max() {
                    self.sobriety_counter.set_to_min();
                }
                self.mind = match self.mind {
                    Mind::Withdrawal(val) => {
                        let new_val = val + state_of_mind;
                        if new_val.is_max() {
                            Mind::Sober(RangedInt::new(val.max() - *val + state_of_mind, 0, SOBER_MAX))
                        } else {
                            Mind::Withdrawal(new_val)
                        }
                    }
                    Mind::Sober(val) => Mind::Sober(val + state_of_mind),
                    Mind::High(val) => {
                        // NOTE: Food and Hunger are the only users of
                        // the attribute modifier so far.
                        //
                        // For hunger, we want it to go down even
                        // while High but it should not increase the
                        // intoxication value.
                        if state_of_mind > 0 {
                            Mind::High(val)
                        } else {
                            Mind::High(val + state_of_mind)
                        }
                    }
                };
            }
            Intoxication{state_of_mind, tolerance_increase} => {
                let state_of_mind_bonus = cmp::max(10, (state_of_mind - self.tolerance));
                self.mind = match self.mind {
                    Mind::Withdrawal(val) => {
                        let intoxication_gain = *val + state_of_mind_bonus - val.max();
                        if intoxication_gain <= 0 {
                            Mind::Withdrawal(val + state_of_mind_bonus)
                        } else if intoxication_gain <= SOBER_MAX{
                            Mind::Sober(RangedInt::new(intoxication_gain, 0, SOBER_MAX))
                        } else {
                            Mind::High(RangedInt::new(intoxication_gain - SOBER_MAX, 0, HIGH_MAX))
                        }
                    }
                    Mind::Sober(val) => {
                        if state_of_mind_bonus > val.max() - *val {
                            Mind::High(RangedInt::new(state_of_mind_bonus + *val - val.max(), 0, HIGH_MAX))
                        } else {
                            Mind::Sober(val + state_of_mind_bonus)
                        }
                    }
                    Mind::High(val) => Mind::High(val + state_of_mind_bonus),
                };
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
        match self.mind {
            Mind::High(val) if *val == val.max() - 1 => self.bonus = Bonus::UncoverMap,
            Mind::High(val) if *val == val.max() - 2 => self.bonus = Bonus::SeeMonstersAndItems,
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
