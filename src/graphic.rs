use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Graphic {
    Empty,
    Tree1,
    Tree2,
    Tree3,
    Tree4,
    Tree5,
    Tree6,
    Tree7,
    Tree8,
    Tree9,
    Tree10,

    Ground1,
    Ground2,
    Ground3,
    Ground4,
    Ground5,

    Twigs1,
    Twigs2,
    Twigs3,
    Twigs4,
    Twigs5,
    Twigs6,
    Twigs7,
    Twigs8,
    Twigs9,
    Twigs10,
    Twigs11,

    Grass1,
    Grass2,
    Grass3,
    Grass4,
    Grass5,
    Grass6,
    Grass7,
    Grass8,
    Grass9,

    Leaves1,
    // TODO: commenting out for now, looks weird in the game
    //Leaves2,
    Leaves3,
    Leaves4,
    Leaves5,

    //Player,
    // NOTE: used for the victory NPC
    Npc,

    // PCs
    CharacterTrousers,
    CharacterSkirt,

    // Tribal NPC set
    CharacterTribalStaffTrousers,
    CharacterTribalStaffBelly,
    CharacterTribalMoon,

    // Sparse NPC set
    // NOTE: not currently used
    CharacterSparseWideArmsWaist,
    CharacterSparseWideSkirt,
    CharacterSparseArmsStraight,

    // Full NPC set
    // NOTE: not currently used
    CharacterFullHandsWaist,
    CharacterFullHandsStaight,
    CharacterFullSuit,

    // Animal Set
    Bird1,
    Bird2,
    Fox,
    Snake,
    Bat,

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

    FoodAcornWide,
    FoodAcornThin,
    FoodCarrotWide,
    FoodCarrotSideways,
    FoodCarrotThin,
    FoodTurnipSmallLeaves,
    FoodTurnipBigLeaves,
    FoodTurnipHeart,
    FoodStriped,

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
            Tree5 => '#',
            Tree6 => '#',
            Tree7 => '#',
            Tree8 => '#',
            Tree9 => '#',
            Tree10 => '#',

            Twigs1 => '.',
            Twigs2 => '.',
            Twigs3 => '.',
            Twigs4 => '.',
            Twigs5 => '.',
            Twigs6 => '.',
            Twigs7 => '.',
            Twigs8 => '.',
            Twigs9 => '.',
            Twigs10 => '.',
            Twigs11 => '.',

            Ground1 => '.',
            Ground2 => '.',
            Ground3 => '.',
            Ground4 => '.',
            Ground5 => '.',

            Grass1 => '.',
            Grass2 => '.',
            Grass3 => '.',
            Grass4 => '.',
            Grass5 => '.',
            Grass6 => '.',
            Grass7 => '.',
            Grass8 => '.',
            Grass9 => '.',

            Leaves1 => '.',
            //Leaves2 => '.',
            Leaves3 => '.',
            Leaves4 => '.',
            Leaves5 => '.',

            //Player => '@',

            // PCs
            CharacterTrousers => '@',
            CharacterSkirt => '@',

            Npc => '@',

            // Tribal NPC set
            CharacterTribalStaffTrousers => '@',
            CharacterTribalStaffBelly => '@',
            CharacterTribalMoon => '@',

            // Sparse NPC set
            CharacterSparseWideArmsWaist => '@',
            CharacterSparseWideSkirt => '@',
            CharacterSparseArmsStraight => '@',

            // Full NPC set
            CharacterFullHandsWaist => '@',
            CharacterFullHandsStaight => '@',
            CharacterFullSuit => '@',

            // Animal Set
            Bird1 => '@',
            Bird2 => '@',
            Fox => '@',
            Snake => '@',
            Bat => '@',

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

            FoodAcornWide => '%',
            FoodAcornThin => '%',
            FoodCarrotWide => '%',
            FoodCarrotSideways => '%',
            FoodCarrotThin => '%',
            FoodTurnipSmallLeaves => '%',
            FoodTurnipBigLeaves => '%',
            FoodTurnipHeart => '%',
            FoodStriped => '%',

            Signpost => '!',
        }
    }
}
