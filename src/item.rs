use std::time::Duration;

use color::{self, Color};
use graphics::Render;
use player::Modifier;

use self::Kind::*;


#[derive(Clone, Copy, PartialEq, Rand, Show)]
pub enum Kind {
    Dose,
    Food,
}


#[derive(Copy, PartialEq, Show)]
pub struct Item {
    pub kind: Kind,
    pub modifier: Modifier,
    pub irresistible: int,
}


impl Render for Item {
    fn render(&self, _dt: Duration) -> (char, Color, Option<Color>) {
        match self.kind {
            Dose => {
                if let Modifier::Intoxication{state_of_mind, ..} = self.modifier {
                    if state_of_mind <= 100 {
                        ('i', color::dose, None)
                    } else {
                        ('I', color::dose_glow, None)
                    }
                } else {
                    unreachable!();
                }
            },
            Food => ('%', color::food, None),
        }
    }
}
