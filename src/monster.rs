use std::rand::Rng;

use super::Action;
use color::{mod, Color};
use level::Level;
use graphics::Render;
use player::Modifier;
use point::{mod, Point};


use self::Kind::*;
use self::AIState::*;


#[deriving(PartialEq, Show)]
pub struct Monster {
    id: uint,
    pub kind: Kind,
    pub position: Point,
    pub dead: bool,
    pub die_after_attack: bool,
    pub ai_state: AIState,

    pub max_ap: int,
    ap: int,
}


#[deriving(Clone, PartialEq, Rand, Show)]
pub enum Kind {
    Anxiety,
    Depression,
    Hunger,
    Shadows,
    Voices,
}

#[deriving(PartialEq, Show)]
pub enum AIState {
    Idle,
    Chasing,
}

impl Monster {
    pub fn new(kind: Kind, position: Point) -> Monster {
        let die_after_attack = match kind {
            Shadows | Voices => true,
            Anxiety | Depression | Hunger => false,
        };
        let max_ap = match kind {
            Depression => 2,
            Anxiety | Hunger | Shadows | Voices => 1,
        };
        Monster {
            id: 0,
            kind: kind,
            position: position,
            dead: false,
            die_after_attack: die_after_attack,
            ai_state: Idle,
            ap: 0,
            max_ap: max_ap,
        }
    }

    pub fn id(&self) -> uint {
        self.id
    }

    pub unsafe fn set_id(&mut self, id: uint) {
        self.id = id;
    }

    pub fn attack_damage(&self) -> Modifier {
        use player::Modifier::*;
        match self.kind {
            Anxiety => Attribute{will: -1, state_of_mind: 0},
            Depression => Death,
            Hunger => Attribute{will: 0, state_of_mind: -20},
            Shadows => Panic(4),
            Voices => Stun(4),
        }
    }

    pub fn act<R: Rng>(&mut self, player_pos: Point, level: &Level, rng: &mut R) -> Action {
        if self.dead {
            panic!(format!("{} is dead, cannot run actions on it.", self));
        }
        let distance = point::tile_distance(self.position, player_pos);
        let ai_state = if distance <= 5 {
            Chasing
        } else {
            Idle
        };
        self.ai_state = ai_state;
        match self.ai_state {
            Chasing => {
                if distance == 1 {
                    Action::Attack(player_pos, self.attack_damage())
                } else {
                    Action::Move(player_pos)
                }
            }
            Idle => {
                // Move randomly about
                let new_pos = level.random_neighbour_position(rng, self.position);
                Action::Move(new_pos)
            }
        }
    }

    pub fn spend_ap(&mut self, count: int) {
        if !(count <= self.ap) {
            println!("bad assert: {}", self);
        }
        assert!(count <= self.ap);
        self.ap -= count;
    }

    pub fn has_ap(&self, count: int) -> bool {
        self.ap >= count
    }

    pub fn new_turn(&mut self) {
        self.ap = self.max_ap;
    }
}

impl Drop for Monster {
    // Implementing a destructor to prevent Moster from being Copy:
    fn drop(&mut self) {}
}

impl Render for Monster {
    fn render(&self) -> (char, Color, Color) {
        let bg = color::background;
        match self.kind {
            Anxiety => ('a', color::anxiety, bg),
            Depression => ('D', color::depression, bg),
            Hunger => ('h', color::hunger, bg),
            Shadows => ('S', color::voices, bg),
            Voices => ('v', color::shadows, bg),
        }
    }
}
