use color::{mod, Color};
use level::{ToColor, ToGlyph};


#[deriving(PartialEq, Show)]
pub enum Item {
    Dose,
    StrongDose,
    Food,
}

impl ToGlyph for Item {
    fn to_glyph(&self) -> char {
        use self::Item::*;

        match *self {
            Dose => 'i',
            StrongDose => 'I',
            Food => '%',
        }
    }
}

impl ToColor for Item {
    fn to_color(&self) -> Color {
        use self::Item::*;

        match *self {
            Dose => color::dose,
            StrongDose => color::dose,
            Food => color::food,
        }
    }
}
