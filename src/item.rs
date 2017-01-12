use time::Duration;

use color::{self, Color};
use graphics::Render;
use player::Modifier;

use self::Kind::*;


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Food,
    Dose,
    StrongDose,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Item {
    pub kind: Kind,
    pub modifier: Modifier,
    pub irresistible: i32,
}

impl Item {
    pub fn is_dose(&self) -> bool {
        match self.kind {
            Kind::Dose => true,
            Kind::StrongDose => true,
            Food => false,
        }
    }
}


impl Render for Item {
    fn render(&self, _dt: Duration) -> (char, Color, Option<Color>) {
        match self.kind {
            Food => ('%', color::food, None),
            Dose => ('i', color::dose, None),
            StrongDose => ('I', color::dose_glow, None),
        }
    }
}
