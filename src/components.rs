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
pub struct Background;
pub struct Bump(entity_manager::ID);
pub struct Position {x: int, y: int}
pub struct Destination {x: int, y: int}
pub struct Solid;
pub struct Tile{level: uint, glyph: char, color: Color}
pub struct Turn{side: Side, ap: int, max_ap: int, spent_this_turn: int}

pub struct GameObject {
    ai: Option<AI>,
    accepts_user_input: Option<AcceptsUserInput>,
    attack_target: Option<AttackTarget>,
    attack_type: Option<AttackType>,
    background: Option<Background>,
    bump: Option<Bump>,
    position: Option<Position>,
    destination: Option<Destination>,
    path: Option<~Path>,
    solid: Option<Solid>,
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
            background: None,
            bump: None,
            position: None,
            destination: None,
            path: None,
            solid: None,
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
                        spent_this_turn: turn.spent_this_turn + spend,
                        .. turn});
            },
            None => fail!(),
        }
    }
}
