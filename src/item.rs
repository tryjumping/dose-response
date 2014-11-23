use color::{mod, Color};
use graphics::Render;


#[deriving(Clone, PartialEq, Rand, Show)]
pub enum Item {
    Dose,
    StrongDose,
    Food,
}

impl Render for Item {
    fn render(&self) -> (char, Color, Color) {
        use self::Item::*;
        let bg = color::background;
        match *self {
            Dose => ('i', color::dose, bg),
            StrongDose => ('I', color::dose_glow, bg),
            Food => ('%', color::food, bg),
        }
    }
}
