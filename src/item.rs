use color::{mod, Color};
use graphics::Render;
use player::Modifier;

use self::Kind::*;


#[deriving(Clone, Copy, PartialEq, Rand, Show)]
pub enum Kind {
    Dose,
    Food,
}


#[deriving(Copy, PartialEq, Show)]
pub struct Item {
    pub kind: Kind,
    pub modifier: Modifier,
    pub irresistible: int,
}


impl Render for Item {
    fn render(&self) -> (char, Color, Color) {
        let bg = color::background;
        match self.kind {
            Dose => {
                if let Modifier::Intoxication{state_of_mind, ..} = self.modifier {
                    if state_of_mind <= 100 {
                        ('i', color::dose, bg)
                    } else {
                        ('I', color::dose_glow, bg)
                    }
                } else {
                    unreachable!();
                }
            },
            Food => ('%', color::food, bg),
        }
    }
}
