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
