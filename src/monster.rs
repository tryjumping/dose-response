use std::rand::Rng;

use super::Action;
use color::{mod, Color};
use level::{Level, Render};
use point::{mod, Point};


#[deriving(PartialEq, Show)]
pub enum Monster {
    Anxiety,
    Depression,
    Hunger,
    Shadows,
    Voices,
}

impl Monster {
    pub fn attack_damage(&self) -> Damage {
        use self::Monster::*;
        match *self {
            Anxiety => Damage::AttributeLoss{will: 1, state_of_mind: 0},
            Depression => Damage::Death,
            Hunger => Damage::AttributeLoss{will: 0, state_of_mind: 20},
            Shadows => Damage::Panic(4),
            Voices => Damage::Stun(4),
        }
    }

    pub fn act<P: Point, Q: Point, R: Rng>(&self, pos: P, player_pos: Q, level: &Level, rng: &mut R) -> Action {
        let distance = point::tile_distance(&pos, &player_pos);
        // TODO: track the state of the AI (agressive/idle) and switch between
        // them as the distance change.
        if distance == 1 {
            Action::Attack(player_pos.coordinates(), self.attack_damage())
        } else if distance < 5 {
            // Follow the player:
            Action::Move(player_pos.coordinates())
        } else {
            // Move randomly about
            let new_pos = level.random_neighbour_position(rng, pos);
            Action::Move(new_pos)
        }
    }
}

impl Render for Monster {
    fn render(&self) -> (char, Color, Color) {
        use self::Monster::*;
        let bg = color::background;
        match *self {
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
