use std::rand::Rng;

use super::Action;
use color::{mod, Color};
use level::Level;
use graphics::Render;
use point::{mod, Point};


#[deriving(PartialEq, Show)]
pub struct Monster {
    pub kind: Kind,
    pub position: (int, int),
    pub dead: bool,
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
    pub fn attack_damage(&self) -> Damage {
        use self::Kind::*;
        match self.kind {
            Anxiety => Damage::AttributeLoss{will: 1, state_of_mind: 0},
            Depression => Damage::Death,
            Hunger => Damage::AttributeLoss{will: 0, state_of_mind: 20},
            Shadows => Damage::Panic(4),
            Voices => Damage::Stun(4),
        }
    }

    pub fn act<P: Point, R: Rng>(&self, player_pos: P, level: &Level, rng: &mut R) -> Action {
        if self.dead {
            panic!(format!("{} is dead, cannot run actions on it.", self));
        }
        let distance = point::tile_distance(&self.position, &player_pos);
        // TODO: track the state of the AI (agressive/idle) and switch between
        // them as the distance change.
        if distance == 1 {
            Action::Attack(player_pos.coordinates(), self.attack_damage())
        } else if distance < 5 {
            // Follow the player:
            Action::Move(player_pos.coordinates())
        } else {
            // Move randomly about
            let new_pos = level.random_neighbour_position(rng, self.position);
            Action::Move(new_pos)
        }
    }
}

impl Render for Monster {
    fn render(&self) -> (char, Color, Color) {
        use self::Kind::*;
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
