use serde::{Deserialize, Serialize};

pub const TILE_SIZE: i32 = 10;

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
    CharacterBelly,

    // PCs
    CharacterTrousers,
    CharacterSkirt,

    // Tribal NPC set
    CharacterTribalStaffTrousers,
    CharacterTribalStaffBelly,
    CharacterTribalMoon,

    // Animal Set
    Bird1,
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

impl Default for Graphic {
    fn default() -> Self {
        Self::Empty
    }
}

pub fn tilemap_coords_px(_tilesize: u32, graphic: Graphic) -> Option<(i32, i32)> {
    use Graphic::*;
    let coords = match graphic {
        Empty => None,

        Tree1 => Some((3, 1)),
        Tree2 => Some((4, 1)),
        Tree3 => Some((5, 1)),
        Tree4 => Some((6, 1)),
        Tree5 => Some((7, 1)),
        Tree6 => Some((8, 1)),
        Tree7 => Some((9, 1)),
        Tree8 => Some((10, 1)),
        Tree9 => Some((3, 2)),
        Tree10 => Some((4, 2)),

        Twigs1 => Some((8, 4)),
        Twigs2 => Some((8, 6)),
        Twigs3 => Some((9, 6)),
        Twigs4 => Some((10, 6)),
        Twigs5 => Some((4, 8)),
        Twigs6 => Some((5, 8)),
        Twigs7 => Some((6, 8)),
        Twigs8 => Some((7, 8)),
        Twigs9 => Some((8, 8)),
        Twigs10 => Some((9, 8)),
        Twigs11 => Some((10, 8)),

        Ground1 => Some((1, 1)),
        Ground2 => Some((1, 2)),
        Ground3 => Some((1, 3)),
        Ground4 => Some((1, 4)),
        Ground5 => Some((1, 5)),

        Leaves1 => Some((5, 3)),
        //Leaves2 => Some((4, 5)),
        Leaves3 => Some((5, 5)),
        Leaves4 => Some((6, 5)),
        Leaves5 => Some((7, 5)),

        Grass1 => Some((8, 3)),
        Grass2 => Some((9, 3)),
        Grass3 => Some((10, 3)),
        Grass4 => Some((8, 5)),
        Grass5 => Some((9, 5)),
        Grass6 => Some((10, 5)),
        Grass7 => Some((8, 7)),
        Grass8 => Some((9, 7)),
        Grass9 => Some((10, 7)),

        Corpse => Some((3 + 5, 13 - 3)),

        Anxiety => Some((0, 10)),
        Hunger => Some((1, 10)),
        Depression => Some((2, 10)),
        Shadows => Some((3, 10)),
        Voices => Some((4, 10)),

        Dose => Some((0, 11)),
        CardinalDose => Some((1, 11)),
        DiagonalDose => Some((2, 11)),
        StrongDose => Some((3, 11)),

        FoodAcornWide => Some((2 + 3, 12 - 3)),
        FoodAcornThin => Some((3 + 3, 12 - 3)),
        FoodCarrotWide => Some((1 + 3, 12 - 3)),
        FoodCarrotSideways => Some((0 + 3, 12 - 3)),
        FoodCarrotThin => Some((6 + 3, 12 - 3)),
        FoodTurnipSmallLeaves => Some((5 + 3, 12 - 3)),
        FoodTurnipBigLeaves => Some((7 + 3, 12 - 3)),
        FoodTurnipHeart => Some((8 + 3, 12 - 3)),
        FoodStriped => Some((4 + 3, 12 - 3)),

        // PCs
        CharacterTrousers => Some((0 + 5, 13 - 3)),
        CharacterSkirt => Some((1 + 5, 13 - 3)),

        // NPC
        CharacterBelly => Some((2 + 5, 13 - 3)),

        // Tribal NPC set
        CharacterTribalStaffTrousers => Some((0 + 5, 14 - 3)),
        CharacterTribalStaffBelly => Some((1 + 5, 14 - 3)),
        CharacterTribalMoon => Some((2 + 5, 14 - 3)),

        // Animal Set
        Bird1 => Some((0 + 8, 15 - 4)),
        Fox => Some((1 + 8, 15 - 4)),
        Snake => Some((2 + 8, 15 - 4)),
        Bat => Some((3 + 8, 15 - 4)),

        Signpost => Some((12 - 1, 8)),
    };
    coords.map(|(tile_x, tile_y)| (tile_x * TILE_SIZE, tile_y * TILE_SIZE))
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

            CharacterBelly => '@',

            // Tribal NPC set
            CharacterTribalStaffTrousers => '@',
            CharacterTribalStaffBelly => '@',
            CharacterTribalMoon => '@',

            // Animal Set
            Bird1 => '@',
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
