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

#[deriving(Eq)]
pub enum MonsterKind {
    Anxiety,
    Depression,
    Hunger,
    Voices,
    Shadows,
}

pub struct AttributeModifier{
    state_of_mind: int,
    will: int
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
pub struct AnxietyKillCounter{count: int, threshold: int}
pub struct AttackTarget(entity_manager::ID);
pub enum AttackType {
    Kill,
    Stun{duration: int},
    Panic{duration: int},
    ModifyAttributes,
}
pub struct Attributes{state_of_mind: int, will: int}
pub struct Addiction{tolerance: int, drop_per_turn: int, last_turn: int}
pub struct Background;
pub struct Bump(entity_manager::ID);
pub struct ExplosionEffect{radius: int}
pub struct Monster{kind: MonsterKind}
pub struct Position {x: int, y: int}
pub struct Destination {x: int, y: int}
pub struct Dose{tolerance_modifier: int, resist_radius: int}
pub struct Solid;
pub struct Stunned{turn: int, duration: int}
pub struct Panicking{turn: int, duration: int}
pub struct Tile{level: uint, glyph: char, color: Color}
pub struct Turn{side: Side, ap: int, max_ap: int, spent_this_tick: int}

pub struct GameObject {
    ai: Option<AI>,
    accepts_user_input: Option<AcceptsUserInput>,
    addiction: Option<Addiction>,
    attack_target: Option<AttackTarget>,
    attack_type: Option<AttackType>,
    attributes: Option<Attributes>,
    anxiety_kill_counter: Option<AnxietyKillCounter>,
    background: Option<Background>,
    bump: Option<Bump>,
    monster: Option<Monster>,
    position: Option<Position>,
    destination: Option<Destination>,
    dose: Option<Dose>,
    explosion_effect: Option<ExplosionEffect>,
    attribute_modifier: Option<AttributeModifier>,
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
            addiction: None,
            attack_target: None,
            attack_type: None,
            attributes: None,
            anxiety_kill_counter: None,
            attribute_modifier: None,
            background: None,
            bump: None,
            dose: None,
            monster: None,
            position: None,
            destination: None,
            explosion_effect: None,
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
