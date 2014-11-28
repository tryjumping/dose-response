use color::{mod, Color};
use graphics::Render;
use player::Modifier;

use self::Kind::*;


// TODO: we should probably drop the distinction between a normal and a strong
// dose on the type level. The rand gen will create them, but the actual
// differences should be encoded within their properties.
#[deriving(Clone, PartialEq, Rand, Show)]
pub enum Kind {
    Dose,
    StrongDose,
    Food,
}


#[deriving(PartialEq, Show)]
pub struct Item {
    pub kind: Kind,
    pub modifier: Modifier,
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
