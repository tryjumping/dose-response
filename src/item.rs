

use self::Kind::*;

use color::{self, Color};
use graphics::Render;
use player::Modifier;
use time::Duration;


#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum Kind {
    Food,
    Dose,
    StrongDose,
    CardinalDose,
    DiagonalDose,
}

impl Kind {
    pub fn iter() -> KindIterator {
        KindIterator { current: Some(self::Kind::Food) }
    }
}

pub struct KindIterator {
    current: Option<Kind>,
}

impl Iterator for KindIterator {
    type Item = Kind;

    fn next(&mut self) -> Option<Self::Item> {
        use self::Kind::*;
        let current = self.current;
        self.current = match current {
            Some(Food) => Some(Dose),
            Some(Dose) => Some(StrongDose),
            Some(StrongDose) => Some(CardinalDose),
            Some(CardinalDose) => Some(DiagonalDose),
            Some(DiagonalDose) => None,
            None => None,
        };
        current
    }
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
            Dose | StrongDose | CardinalDose | DiagonalDose => true,
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
            CardinalDose => ('+', color::shattering_dose, None),
            DiagonalDose => ('x', color::shattering_dose, None),
        }
    }
}
