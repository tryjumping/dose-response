use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Graphic {
    Empty,
    Tree1,
    Tree2,
    Tree3,
    Tree4,
    Ground, // TODO multiple tiles

    Player,
    Npc,
    Corpse,

    Anxiety,
    Depression,
    Hunger,
    Shadows,
    Voices,

    Dose,
    StrongDose,
    CardinalDose,
    DiagonalDose,
    Food,

    Signpost,
}

impl Into<char> for Graphic {
    fn into(self) -> char {
        use Graphic::*;
        match self {
            Empty => ' ',
            Tree1 => '#',
            Tree2 => '#',
            Tree3 => '#',
            Tree4 => '#',
            Ground => '.', // TODO multiple tiles
            Player => '@',
            Npc => '@',
            Corpse => '&',
            Anxiety => 'a',
            Depression => 'D',
            Hunger => 'h',
            Shadows => 'S',
            Voices => 'v',
            Dose => 'i',
            StrongDose => 'I',
            CardinalDose => '+',
            DiagonalDose => 'x',
            Food => '%',
            Signpost => '!',
        }
    }
}
