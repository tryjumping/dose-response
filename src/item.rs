use self::Kind::*;

use crate::{color::Color, graphic::Graphic, palette::Palette, player::Modifier};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    Food,
    Dose,
    CardinalDose,
    DiagonalDose,
    StrongDose,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let precision = f.precision().unwrap_or(1000);
        let s = if precision < 12 {
            match *self {
                Food => "Food",
                Dose => "Dose",
                CardinalDose => "Card. Dose",
                DiagonalDose => "Diag. Dose",
                StrongDose => "Strong Dose",
            }
        } else {
            self.name()
        };
        f.write_str(s)
    }
}

impl Kind {
    pub fn iter() -> KindIterator {
        KindIterator {
            current: Some(self::Kind::Food),
        }
    }

    pub fn name(&self) -> &str {
        match *self {
            Food => "Food",
            Dose => "Dose",
            CardinalDose => "Cardinal Dose",
            DiagonalDose => "Diagonal Dose",
            StrongDose => "Strong Dose",
        }
    }
}

#[derive(Copy, Clone)]
pub struct KindIterator {
    current: Option<Kind>,
}

impl Iterator for KindIterator {
    type Item = Kind;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        self.current = match current {
            Some(Food) => Some(Dose),
            Some(Dose) => Some(CardinalDose),
            Some(CardinalDose) => Some(DiagonalDose),
            Some(DiagonalDose) => Some(StrongDose),
            Some(StrongDose) => None,
            None => None,
        };
        current
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Item {
    pub kind: Kind,
    pub graphic: Graphic,
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

    pub fn graphic(&self) -> Graphic {
        self.graphic
    }

    pub fn color(&self, palette: &Palette) -> Color {
        match self.kind {
            Food => palette.food,
            Dose => palette.dose,
            StrongDose => palette.strong_dose,
            CardinalDose => palette.shattering_dose,
            DiagonalDose => palette.shattering_dose,
        }
    }
}
