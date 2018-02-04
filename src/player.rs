use color::{self, Color};
use formula::{self, ANXIETIES_PER_WILL, SOBRIETY_COUNTER, WILL, WITHDRAWAL};
use graphics::Render;
use item::Item;
use monster::{Monster, CompanionBonus};
use point::Point;
use ranged_int::Ranged;
use std::fmt::{Display, Error, Formatter};
use std::time::Duration;


#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
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

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
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

    pub fn is_sober(&self) -> bool {
        !self.is_high()
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

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Bonus {
    None,
    SeeMonstersAndItems,
    UncoverMap,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CauseOfDeath {
    Exhausted,
    Overdosed,
    LostWill,
    Killed,
}

/// Values related to the Player the AI routines might want to
/// investigate.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PlayerInfo {
    pub pos: Point,
    pub mind: Mind,
    pub max_ap: i32,
    pub will: i32,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub mind: Mind,
    pub will: Ranged,
    pub tolerance: i32,
    pub panic: Ranged,
    pub stun: Ranged,

    pub pos: Point,
    pub inventory: Vec<Item>,
    pub anxiety_counter: Ranged,
    // TODO: merge this with the other bonuses
    pub bonus: Bonus,
    pub bonuses: Vec<CompanionBonus>,
    /// How many turns after max Will to achieve victory
    pub sobriety_counter: Ranged,
    pub current_high_streak: i32,
    pub longest_high_streak: i32,

    pub dead: bool,
    pub invincible: bool,
    pub perpetrator: Option<Monster>,

    // TODO: Use a Ranged here?
    pub base_max_ap: i32,
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
            perpetrator: None,
            base_max_ap: formula::PLAYER_BASE_AP,
            ap: formula::PLAYER_BASE_AP,
            bonus: Bonus::None,
            bonuses: Vec::with_capacity(10),
            sobriety_counter: Ranged::new_min(SOBRIETY_COUNTER),
            current_high_streak: 0,
            longest_high_streak: 0,
        }
    }

    pub fn info(&self) -> PlayerInfo {
        PlayerInfo {
            max_ap: self.base_max_ap,
            mind: self.mind,
            pos: self.pos,
            will: self.will.to_int(),
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

            let mind_drop = formula::mind_drop_per_turn(&self.bonuses);
            self.mind = formula::mind_take_turn(self.mind, mind_drop);
            self.ap = self.max_ap();
        }
    }

    pub fn max_ap(&self) -> i32 {
        if self.bonuses.contains(&CompanionBonus::DoubleActionPoints) {
            self.base_max_ap * 2
        } else {
            self.base_max_ap
        }
    }

    pub fn alive(&self) -> bool {
        self.invincible || formula::cause_of_death(self).is_none()
    }

    pub fn take_effect(&mut self, effect: Modifier) {
        use self::Modifier::*;
        match effect {
            Death => self.dead = true,
            Attribute {
                will,
                state_of_mind,
            } => {
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
                self.mind = formula::intoxicate(self.mind, self.tolerance, state_of_mind);
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
