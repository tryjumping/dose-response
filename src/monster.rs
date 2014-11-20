use color::{mod, Color};
use level::{ToColor, ToGlyph};


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
