use std::num;

use engine::{Color};
pub use map::Path;
pub use self::ai::AI;
use entity_manager;

#[deriving(Eq)]
pub enum Side {
    Player,
    Computer,
}

mod ai {
    pub enum Behaviour {
        Individual,
        Pack,
    }

    pub enum State {
        Idle,
        Aggressive,
    }

    pub struct AI{behaviour: Behaviour, state: State}
}

pub struct AcceptsUserInput;
pub struct AttackTarget(entity_manager::ID);
pub enum AttackType {
    Kill,
    Stun{duration: int},
    Panic{duration: int},
    ModifyAttributes{state_of_mind: int, will: int},
}
pub struct Attributes{state_of_mind: int, will: int}
pub struct Background;
pub struct Bump(entity_manager::ID);
pub struct Position {x: int, y: int}
pub struct Destination {x: int, y: int}
pub struct Solid;
pub struct Stunned{turn: int, duration: int}
pub struct Panicking{turn: int, duration: int}
pub struct Tile{level: uint, glyph: char, color: Color}
pub struct Turn{side: Side, ap: int, max_ap: int, spent_this_tick: int}

pub struct GameObject {
    ai: Option<AI>,
    accepts_user_input: Option<AcceptsUserInput>,
    attack_target: Option<AttackTarget>,
    attack_type: Option<AttackType>,
    attributes: Option<Attributes>,
    background: Option<Background>,
    bump: Option<Bump>,
    position: Option<Position>,
    destination: Option<Destination>,
    path: Option<~Path>,
    panicking: Option<Panicking>,
    solid: Option<Solid>,
    stunned: Option<Stunned>,
    tile: Option<Tile>,
    turn: Option<Turn>,
}

impl GameObject {
    pub fn new() -> GameObject {
        GameObject {
            ai: None,
            accepts_user_input: None,
            attack_target: None,
            attack_type: None,
            attributes: None,
            background: None,
            bump: None,
            position: None,
            destination: None,
            path: None,
            panicking: None,
            solid: None,
            stunned: None,
            tile: None,
            turn: None,
        }
    }

    pub fn spend_ap(&mut self, spend: int) {
        match self.turn {
            Some(turn) => {
                assert!(spend <= turn.ap);

                self.turn = Some(Turn{
                        ap: turn.ap - spend,
                        spent_this_tick: turn.spent_this_tick + spend,
                        .. turn});
            },
            None => fail!(),
        }
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
