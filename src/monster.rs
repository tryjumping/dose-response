use std::rand::Rng;

use super::Action;
use color::{mod, Color};
use level::{ToColor, ToGlyph, Level};
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
        let pos = pos.coordinates();
        let player_pos = player_pos.coordinates();
        if point::tile_distance(pos, player_pos) == 1 {
            Action::Attack(player_pos, self.attack_damage())
        } else {
            let new_pos = level.random_neighbour_position(rng, pos);
            Action::Move(new_pos)
        }
    }
}

impl ToGlyph for Monster {
    fn to_glyph(&self) -> char {
        use self::Monster::*;

        match *self {
            Anxiety => 'a',
            Depression => 'D',
            Hunger => 'h',
            Shadows => 'S',
            Voices => 'V',
        }
    }
}

impl ToColor for Monster {
    fn to_color(&self) -> Color {
        use self::Monster::*;

        match *self {
            Anxiety => color::anxiety,
            Depression => color::depression,
            Hunger => color::hunger,
            Shadows => color::voices,
            Voices => color::shadows,
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
