use std::num;
use std::iter::{Map, Range};

use engine::{Color};
pub use map;

#[deriving(Eq)]
pub enum Side {
    Player,
    Computer,
}

#[deriving(Eq)]
pub enum MonsterKind {
    Anxiety,
    Depression,
    Hunger,
    Voices,
    Shadows,
}

pub mod ai {
    #[deriving(Eq)]
    pub enum Behaviour {
        Individual,
        Pack,
    }

    #[deriving(Eq)]
    pub enum State {
        Idle,
        Aggressive,
    }

}

impl Turn {
    pub fn spend_ap(&self, spend: int) -> Turn {
        assert!(spend <= self.ap);
        Turn{ap: self.ap - spend,
             spent_this_tick: self.spent_this_tick + spend,
             .. *self}
    }
}

impl Stunned {
    pub fn remaining(&self, current_turn: int) -> int {
        num::max((self.turn + self.duration) - current_turn, 0)
    }
}

impl Panicking {
    pub fn remaining(&self, current_turn: int) -> int {
        num::max((self.turn + self.duration) - current_turn, 0)
    }
}
