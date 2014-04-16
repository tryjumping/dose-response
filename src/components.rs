use emhyr::{Entity};

use engine::{Color};


#[deriving(Eq, Clone, Show)]
pub struct AI{
    pub behaviour: ai::Behaviour,
    pub state: ai::State,
}

#[deriving(Eq, Clone, Show)]
pub struct AcceptsUserInput;

#[deriving(Eq, Clone, Show)]
pub struct Addiction{
    pub tolerance: int,
    pub drop_per_turn: int,
    pub last_turn: int,
}

#[deriving(Eq, Clone, Show)]
pub struct AnxietyKillCounter{
    pub count: int,
    pub threshold: int,
}

#[deriving(Eq, Clone, Show)]
pub struct AttackTarget(Entity);

#[deriving(Eq, Clone, Show)]
pub enum AttackType {
    Kill,
    Stun {pub duration: int},
    Panic{pub duration: int},
    ModifyAttributes,
}

#[deriving(Eq, Clone, Show)]
pub struct AttributeModifier{
    pub state_of_mind: int,
    pub will: int,
}

#[deriving(Eq, Clone, Show)]
pub struct Attributes{
    pub state_of_mind: int,
    pub will: int,
}

#[deriving(Eq, Clone, Show)]
pub struct Background;

#[deriving(Eq, Clone, Show)]
pub struct Bump(Entity);

#[deriving(Eq, Clone, Show)]
pub struct ColorAnimation{
    pub color: Color,
    pub progress: f32,
    pub forward: bool,
}

#[deriving(Eq, Clone, Show)]
pub struct Corpse{
    pub glyph: char,
    pub color: Color,
    pub solid: bool,
}

#[deriving(Eq, Clone, Show)]
pub struct Destination {pub x: int, pub y: int}

#[deriving(Eq, Clone, Show)]
pub struct Dose{
    pub tolerance_modifier: int,
    pub resist_radius: int,
}

#[deriving(Eq, Clone, Show)]
pub struct Edible;

#[deriving(Eq, Clone, Show)]
pub struct Exploration{pub radius: int}

#[deriving(Eq, Clone, Show)]
pub struct Explored;

#[deriving(Eq, Clone, Show)]
pub struct ExplosionEffect{pub radius: int}

#[deriving(Eq, Clone, Show)]
pub struct FadeColor{
    pub from: Color,
    pub to: Color,
    pub duration_s: f32,
    pub repetitions: Repetitions,
}

#[deriving(Eq, Clone, Show)]
pub struct FadeOut{
    pub to: Color,
    pub duration_s: f32,
}

#[deriving(Eq, Clone, Show)]
pub struct FadingOut;

#[deriving(Eq, Clone, Show)]
pub struct InventoryItem{pub owner: Entity}

#[deriving(Eq, Clone, Show)]
pub struct Monster{pub kind: MonsterKind}

#[deriving(Eq, Clone, Show)]
pub struct Panicking{
    pub turn: int,
    pub duration: int}

#[deriving(Eq, Clone, Show)]
pub struct Pickable;

#[deriving(Eq, Clone, Show)]
pub struct Position {
    pub x: int,
    pub y: int,
}

#[deriving(Eq, Clone, Show)]
pub struct Solid;

#[deriving(Eq, Clone, Show)]
pub struct Stunned{
    pub turn: int,
    pub duration: int,
}

#[deriving(Eq, Clone, Show)]
pub struct Tile{
    pub level: uint,
    pub glyph: char,
    pub color: Color,
}

#[deriving(Eq, Clone, Show)]
pub struct Turn{
    pub side: Side,
    pub ap: int,
    pub max_ap: int,
    pub spent_this_tick: int,
}

#[deriving(Eq, Clone, Show)]
pub struct UsingItem{pub item: Entity}


#[deriving(Eq, Clone, Show)]
pub enum Side {
    Player,
    Computer,
}

#[deriving(Eq, Clone, Show)]
pub enum MonsterKind {
    Anxiety,
    Depression,
    Hunger,
    Voices,
    Shadows,
}

#[deriving(Eq, Clone, Show)]
pub enum Repetitions {
    Infinite,
    Count(int),
}

pub mod ai {
    #[deriving(Eq, Clone, Show)]
    pub enum Behaviour {
        Individual,
        Pack,
    }

    #[deriving(Eq, Clone, Show)]
    pub enum State {
        Idle,
        Aggressive,
    }
}
