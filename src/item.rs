use self::Kind::*;

use color::{self, Color};
use player::Modifier;

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    Food,
    Dose,
    StrongDose,
    CardinalDose,
    DiagonalDose,
}

impl Kind {
    pub fn iter() -> KindIterator {
        KindIterator {
            current: Some(self::Kind::Food),
        }
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

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
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

    pub fn glyph(&self) -> char {
        match self.kind {
            Food => '%',
            Dose => 'i',
            StrongDose => 'I',
            CardinalDose => '+',
            DiagonalDose => 'x',
        }
    }

    pub fn color(&self) -> Color {
        match self.kind {
            Food => color::food,
            Dose => color::dose,
            StrongDose => color::strong_dose,
            CardinalDose => color::shattering_dose,
            DiagonalDose => color::shattering_dose,
        }
    }
}
