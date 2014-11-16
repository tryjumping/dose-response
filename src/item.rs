use engine::Color;
use level::{ToColor, ToGlyph};
use world::col as color;


#[deriving(PartialEq, Show)]
pub enum Item {
    Dose,
    StrongDose,
    Food,
}

impl ToGlyph for Item {
    fn to_glyph(&self) -> char {
        match *self {
            Dose => 'i',
            StrongDose => 'I',
            Food => '%',
        }
    }
}

impl ToColor for Item {
    fn to_color(&self) -> Color {
        match *self {
            Dose => color::dose,
            StrongDose => color::dose,
            Food => color::food,
        }
    }
}
