use crate::{
    animation,
    color::Color,
    formula::{self, ANXIETIES_PER_WILL, WILL, WITHDRAWAL},
    graphic::Graphic,
    item::Item,
    monster::{CompanionBonus, Monster},
    palette::Palette,
    point::Point,
    ranged_int::Ranged,
};

use std::fmt::{Display, Error, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Modifier {
    Death,
    // TODO: probably rename `state_of_mind` to something like hunger
    // or satiation or maybe split it in two. It's a bit confusing now
    // since the two users of this are the Food item (which never
    // increases past Sober) and Hunger (which is only negative and
    // works even when high).
    Attribute {
        will: i32,
        state_of_mind: i32,
    },
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
        matches!(self, Mind::High(_))
    }

    pub fn is_sober(&self) -> bool {
        !self.is_high()
    }
}

impl Display for Mind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use self::Mind::*;
        let s = match *self {
            Withdrawal(_) => "Withdrawal",
            Sober(_) => "Sober",
            High(_) => "High",
        };
        f.write_str(s)
    }
}

impl Default for Mind {
    fn default() -> Self {
        Self::Sober(Ranged::default())
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Bonus {
    None,
    SeeMonstersAndItems,
    UncoverMap,
}

impl Default for Bonus {
    fn default() -> Self {
        Self::None
    }
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Player {
    pub mind: Mind,
    pub will: Ranged,
    pub tolerance: i32,
    pub panic: Ranged,
    pub stun: Ranged,

    pub pos: Point,
    pub motion_animation: animation::Move,
    pub color_index: usize,
    pub graphic: Graphic,
    pub inventory: Vec<Item>,
    pub anxiety_counter: Ranged,
    // TODO: merge this with the other bonuses
    pub bonus: Bonus,
    pub bonuses: Vec<CompanionBonus>,
    pub current_high_streak: i32,
    pub longest_high_streak: i32,

    /// How many times has the player been reset.
    pub reset_count: i32,

    pub dead: bool,
    pub invincible: bool,
    pub perpetrator: Option<Monster>,

    ap: i32,
}

impl Player {
    pub fn new(pos: Point, invincible: bool) -> Player {
        let mut player = Player::default();
        player.reset();
        player.pos = pos;
        player.invincible = invincible;
        player.will = Ranged::new(formula::PLAYER_STARTING_WILL, WILL);
        player.tolerance = 0;
        player.inventory = vec![];
        player.color_index = 0;
        player.graphic = Graphic::CharacterSkirt;
        player.current_high_streak = 0;
        player.longest_high_streak = 0;
        player.reset_count = 0;

        player
    }

    pub fn reset(&mut self) {
        log::info!("Resetting player to the initial state.");

        self.mind = Mind::Withdrawal(Ranged::new_max(WITHDRAWAL));

        if self.will.to_int() < formula::PLAYER_STARTING_WILL {
            self.will = Ranged::new(formula::PLAYER_STARTING_WILL, WILL);
        } else {
            // NOTE: we're not resetting Will to a lower value because
            // if the player carries doses in the inventory, those
            // would suddenly start going off.
        }

        self.panic = Ranged::new_min(formula::PANIC_TURNS);
        self.stun = Ranged::new_min(formula::STUN_TURNS);
        self.anxiety_counter = Ranged::new_min(ANXIETIES_PER_WILL);
        self.dead = false;
        self.perpetrator = None;
        self.ap = formula::PLAYER_BASE_AP;
        self.bonus = Bonus::None;
        self.bonuses = Vec::with_capacity(10);

        self.reset_count += 1;
    }

    pub fn info(&self) -> PlayerInfo {
        PlayerInfo {
            max_ap: self.max_ap(),
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
        self.panic -= count;
        self.stun -= count;
    }

    pub fn has_ap(&self, count: i32) -> bool {
        self.ap >= count
    }

    pub fn new_turn(&mut self) {
        if self.alive() {
            let mind_drop = formula::mind_drop_per_turn(&self.bonuses);
            self.mind = formula::mind_take_turn(self.mind, mind_drop);
            self.ap = self.max_ap();
        }
    }

    pub fn max_ap(&self) -> i32 {
        formula::player_max_ap(&self.bonuses)
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
                self.mind = formula::process_hunger(self.mind, state_of_mind);
            }
            Intoxication {
                state_of_mind,
                tolerance_increase,
            } => {
                self.mind = formula::intoxicate(self.mind, self.tolerance, state_of_mind);
                self.tolerance += tolerance_increase;
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

    pub fn color(&self, palette: &Palette) -> Color {
        if self.alive() {
            palette.player(self.color_index)
        } else {
            palette.dead_player
        }
    }

    pub fn graphic(&self) -> Graphic {
        match (self.alive(), self.mind) {
            (true, Mind::High(val)) => {
                if val.percent() >= 0.75 {
                    Graphic::Bat
                } else if val.percent() >= 0.5 {
                    Graphic::Snake
                } else if val.percent() >= 0.25 {
                    Graphic::Fox
                } else {
                    Graphic::Bird1
                }
            }
            (true, _) => self.graphic,
            (false, _) => Graphic::Corpse,
        }
    }
}
