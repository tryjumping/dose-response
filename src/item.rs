use color::{mod, Color};
use graphics::Render;

use self::Kind::*;


#[deriving(Clone, PartialEq, Rand, Show)]
pub enum Kind {
    Dose,
    StrongDose,
    Food,
}


#[deriving(PartialEq, Show)]
pub struct Item {
    pub kind: Kind,
}


impl Item {
    pub fn new(kind: Kind) -> Item {
        Item{
            kind: kind,
        }
    }
}


impl Render for Item {
    fn render(&self) -> (char, Color, Color) {
        let bg = color::background;
        match self.kind {
            Dose => ('i', color::dose, bg),
            StrongDose => ('I', color::dose_glow, bg),
            Food => ('%', color::food, bg),
        }
    }
}
