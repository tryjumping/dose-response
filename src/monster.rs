use engine::Color;
use level::{ToColor, ToGlyph};
use world::col as color;


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
        match *self {
            Anxiety => color::anxiety,
            Depression => color::depression,
            Hunger => color::hunger,
            Shadows => color::voices,
            Voices => color::shadows,
        }
    }
}
