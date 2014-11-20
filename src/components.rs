// use std::cmp::max;
// use std::time::Duration;

// use engine::{Color};
use point::Point;


// pub struct AI{
//     pub behaviour: ai::Behaviour,
//     pub state: ai::State,
// }

// pub struct AcceptsUserInput;

// pub struct Addiction{
//     pub tolerance: int,
//     pub drop_per_turn: int,
//     pub last_turn: int,
// }

// pub struct AnxietyKillCounter{
//     pub count: int,
//     pub threshold: int,
// }

// pub struct AttackTarget(pub Entity);

// pub enum AttackType {
//     Kill,
//     Stun {pub duration: int},
//     Panic{pub duration: int},
//     ModifyAttributes,
// }

// pub struct AttributeModifier{
//     pub state_of_mind: int,
//     pub will: int,
// }

// pub struct Attributes{
//     pub state_of_mind: int,
//     pub will: int,
// }

// pub struct Background;

// // pub struct Bump(pub Entity);

// // TODO: maybe we should rename "repetitions" to "transitions" instead. Because
// // to change from the starting colour to the new one should take Count(2) reps.
// pub struct ColorAnimation{
//     pub from: Color,
//     pub to: Color,
//     pub repetitions: Repetitions,
//     pub transition_duration: Duration,
//     pub current: ColorAnimationState,
// }

// #[deriving(PartialEq, Clone, Show)]
// pub struct ColorAnimationState {
//     pub color: Color,
//     pub fade_direction: ColorFadeDirection,
//     pub elapsed_time: Duration,
// }

// pub struct Corpse{
//     pub glyph: char,
//     pub color: Color,
//     pub solid: bool,
// }

// pub struct Destination {pub x: int, pub y: int}

// impl Point for Destination {
//     fn coordinates(&self) -> (int, int) {
//         (self.x, self.y)
//     }
// }

// pub struct Dose{
//     pub tolerance_modifier: int,
//     pub resist_radius: int,
// }

// pub struct Edible;

// pub struct Exploration{pub radius: int}

// pub struct Explored;

// pub struct ExplosionEffect{pub radius: int}

// pub struct FadingOut;

// // pub struct InventoryItem{pub owner: Entity}

// pub struct Monster{pub kind: MonsterKind}

// pub struct Panicking{
//     pub turn: int,
//     pub duration: int}

// pub struct Pickable;

pub struct Position {
    pub x: int,
    pub y: int,
}

impl Point for Position {
    fn coordinates(&self) -> (int, int) {
        (self.x, self.y)
    }
}

// pub struct Solid;

// pub struct Stunned{
//     pub turn: int,
//     pub duration: int,
// }

// pub struct Tile{
//     pub level: uint,
//     pub glyph: char,
//     pub color: Color,
// }

// pub struct Turn{
//     pub side: Side,
//     pub ap: int,
//     pub max_ap: int,
//     pub spent_this_tick: int,
// }

// // pub struct UsingItem{pub item: Entity}



// #[deriving(PartialEq, Clone, Show)]
// pub enum MonsterKind {
//     Anxiety,
//     Depression,
//     Hunger,
//     Voices,
//     Shadows,
// }

// #[deriving(PartialEq, Clone, Show)]
// pub enum Repetitions {
//     Infinite,
//     Count(int),
// }

// #[deriving(PartialEq, Clone, Show)]
// pub enum ColorFadeDirection {
//     Forward,
//     Backward,
// }

// pub mod ai {
//     #[deriving(PartialEq, Clone, Show)]
//     pub enum Behaviour {
//         Individual,
//         Pack,
//     }

//     #[deriving(PartialEq, Clone, Show)]
//     pub enum State {
//         Idle,
//         Aggressive,
//     }
// }


// impl Turn {
//     pub fn spend_ap(&self, spend: int) -> Turn {
//         assert!(spend <= self.ap);
//         Turn{ap: self.ap - spend,
//              spent_this_tick: self.spent_this_tick + spend,
//              .. *self}
//     }
// }

// impl Stunned {
//     pub fn remaining(&self, current_turn: int) -> int {
//         max((self.turn + self.duration) - current_turn, 0)
//     }
// }

// impl Panicking {
//     pub fn remaining(&self, current_turn: int) -> int {
//         max((self.turn + self.duration) - current_turn, 0)
//     }
// }
