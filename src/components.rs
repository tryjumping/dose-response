use std::cmp::max;
use emhyr::{Entity, Component};

use engine::{Color};
use point::Point;


#[component]
pub struct AI{
    pub behaviour: ai::Behaviour,
    pub state: ai::State,
}

#[component]
pub struct AcceptsUserInput;

#[component]
pub struct Addiction{
    pub tolerance: int,
    pub drop_per_turn: int,
    pub last_turn: int,
}

#[component]
pub struct AnxietyKillCounter{
    pub count: int,
    pub threshold: int,
}

#[component]
pub struct AttackTarget(pub Entity);

#[component]
pub enum AttackType {
    Kill,
    Stun {pub duration: int},
    Panic{pub duration: int},
    ModifyAttributes,
}

#[component]
pub struct AttributeModifier{
    pub state_of_mind: int,
    pub will: int,
}

#[component]
pub struct Attributes{
    pub state_of_mind: int,
    pub will: int,
}

#[component]
pub struct Background;

#[component]
pub struct Bump(pub Entity);

// TODO: maybe we should rename "repetitions" to "transitions" instead. Because
// to change from the starting colour to the new one should take Count(2) reps.
#[component]
pub struct ColorAnimation{
    pub from: Color,
    pub to: Color,
    pub repetitions: Repetitions,
    pub transition_duration: Sec,
    pub current: ColorAnimationState,
}

#[deriving(PartialEq, Clone, Show)]
pub struct ColorAnimationState {
    pub color: Color,
    pub fade_direction: ColorFadeDirection,
    pub elapsed_time: Sec,
}

#[component]
pub struct Corpse{
    pub glyph: char,
    pub color: Color,
    pub solid: bool,
}

#[component]
pub struct Destination {pub x: int, pub y: int}

impl Point for Destination {
    fn coordinates(&self) -> (int, int) {
        (self.x, self.y)
    }
}

#[component]
pub struct Dose{
    pub tolerance_modifier: int,
    pub resist_radius: int,
}

#[component]
pub struct Edible;

#[component]
pub struct Exploration{pub radius: int}

#[component]
pub struct Explored;

#[component]
pub struct ExplosionEffect{pub radius: int}

#[component]
pub struct FadingOut;

#[component]
pub struct InventoryItem{pub owner: Entity}

#[component]
pub struct Monster{pub kind: MonsterKind}

#[component]
pub struct Panicking{
    pub turn: int,
    pub duration: int}

#[component]
pub struct Pickable;

#[component]
pub struct Position {
    pub x: int,
    pub y: int,
}

impl Point for Position {
    fn coordinates(&self) -> (int, int) {
        (self.x, self.y)
    }
}

#[component]
pub struct Solid;

#[component]
pub struct Stunned{
    pub turn: int,
    pub duration: int,
}

#[component]
pub struct Tile{
    pub level: uint,
    pub glyph: char,
    pub color: Color,
}

#[component]
pub struct Turn{
    pub side: Side,
    pub ap: int,
    pub max_ap: int,
    pub spent_this_tick: int,
}

#[component]
pub struct UsingItem{pub item: Entity}


#[deriving(PartialEq, Clone, Show)]
pub enum Side {
    Player,
    Computer,
}

#[deriving(PartialEq, Clone, Show)]
pub enum MonsterKind {
    Anxiety,
    Depression,
    Hunger,
    Voices,
    Shadows,
}

#[deriving(PartialEq, Clone, Show)]
pub enum Repetitions {
    Infinite,
    Count(int),
}

#[deriving(PartialEq, Clone, Show)]
pub struct Sec(pub f32);

#[deriving(PartialEq, Clone, Show)]
pub enum ColorFadeDirection {
    Forward,
    Backward,
}

pub mod ai {
    #[deriving(PartialEq, Clone, Show)]
    pub enum Behaviour {
        Individual,
        Pack,
    }

    #[deriving(PartialEq, Clone, Show)]
    pub enum State {
        Idle,
        Aggressive,
    }
}


impl Turn {
    pub fn spend_ap(&self, spend: int) -> Turn {
        assert!(spend <= self.ap);
        Turn{ap: self.ap - spend,
             spent_this_tick: self.spent_this_tick + spend,
             .. *self}
    }
}

impl Stunned {
    pub fn remaining(&self, current_turn: int) -> int {
        max((self.turn + self.duration) - current_turn, 0)
    }
}

impl Panicking {
    pub fn remaining(&self, current_turn: int) -> int {
        max((self.turn + self.duration) - current_turn, 0)
    }
}
