use std::rand::Rng;

use super::Action;
use color::{mod, Color};
use level::Level;
use graphics::Render;
use point;


use self::Kind::*;


#[deriving(PartialEq, Show)]
pub struct Monster {
    pub kind: Kind,
    pub position: (int, int),
    pub dead: bool,
    pub die_after_attack: bool,

    max_ap: int,
    ap: int,
    spent_this_tick: int,
}


#[deriving(PartialEq, Show)]
pub enum Kind {
    Anxiety,
    Depression,
    Hunger,
    Shadows,
    Voices,
}

impl Monster {
    pub fn new(kind: Kind, position: (int, int)) -> Monster {
        let die_after_attack = match kind {
            Shadows | Voices => true,
            Anxiety | Depression | Hunger => false,
        };
        Monster {
            kind: kind,
            position: position,
            dead: false,
            die_after_attack: die_after_attack,
            ap: 0,
            max_ap: 0,
            spent_this_tick: 0,
        }
    }

    pub fn attack_damage(&self) -> Damage {
        match self.kind {
            Anxiety => Damage::AttributeLoss{will: 1, state_of_mind: 0},
            Depression => Damage::Death,
            Hunger => Damage::AttributeLoss{will: 0, state_of_mind: 20},
            Shadows => Damage::Panic(4),
            Voices => Damage::Stun(4),
        }
    }

    pub fn act<R: Rng>(&self, player_pos: (int, int), level: &Level, rng: &mut R) -> Action {
        if self.dead {
            panic!(format!("{} is dead, cannot run actions on it.", self));
        }
        let distance = point::tile_distance(self.position, player_pos);
        // TODO: track the state of the AI (agressive/idle) and switch between
        // them as the distance change.
        if distance == 1 {
            Action::Attack(player_pos, self.attack_damage())
        } else if distance < 5 {
            // Follow the player:
            Action::Move(player_pos)
        } else {
            // Move randomly about
            let new_pos = level.random_neighbour_position(rng, self.position);
            Action::Move(new_pos)
        }
    }

    pub fn spend_ap(&mut self, count: int) {
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
            Voices => ('V', color::shadows, bg),
        }
    }
}

#[deriving(PartialEq, Show)]
pub enum Damage {
    Death,
    AttributeLoss{will: int, state_of_mind: int},
    Panic(int),
    Stun(int),
}
