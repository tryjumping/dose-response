use emhyr::{Entity};

use engine::{Color};


pub struct AI{behaviour: ai::Behaviour, state: ai::State}

pub struct AcceptsUserInput;

pub struct Addiction{tolerance: int, drop_per_turn: int, last_turn: int}

pub struct AnxietyKillCounter{count: int, threshold: int}

pub struct AttackTarget(Entity);

pub enum   AttackType {Kill, Stun{duration: int}, Panic{duration: int}, ModifyAttributes}

pub struct AttributeModifier{state_of_mind: int, will: int}

pub struct Attributes{state_of_mind: int, will: int}

pub struct Background;

pub struct Bump(Entity);

pub struct ColorAnimation{color: Color, progress: f32, forward: bool}

pub struct Corpse{glyph: char, color: Color, solid: bool}

#[deriving(Eq, Clone, Show)]
pub struct Destination {pub x: int, pub y: int}

pub struct Dose{tolerance_modifier: int, resist_radius: int}

pub struct Edible;

pub struct Exploration{radius: int}

pub struct Explored;

pub struct ExplosionEffect{radius: int}

pub struct FadeColor{from: Color, to: Color, duration_s: f32, repetitions: Repetitions}

pub struct FadeOut{to: Color, duration_s: f32}

pub struct FadingOut;

pub struct InventoryItem{owner: Entity}

pub struct Monster{kind: MonsterKind}

pub struct Panicking{turn: int, duration: int}

pub struct Pickable;

#[deriving(Eq, Clone, Show)]
pub struct Position {pub x: int, pub y: int}

pub struct Solid;

pub struct Stunned{turn: int, duration: int}

pub struct Tile{level: uint, glyph: char, color: Color}

pub struct Turn{side: Side, ap: int, max_ap: int, spent_this_tick: int}

pub struct UsingItem{item: Entity}


#[deriving(Eq)]
pub enum Side {
    Player,
    Computer,
}

#[deriving(Eq)]
pub enum MonsterKind {
    Anxiety,
    Depression,
    Hunger,
    Voices,
    Shadows,
}

#[deriving(Eq)]
pub enum Repetitions {
    Infinite,
    Count(int),
}

pub mod ai {
    #[deriving(Eq)]
    pub enum Behaviour {
        Individual,
        Pack,
    }

    #[deriving(Eq)]
    pub enum State {
        Idle,
        Aggressive,
    }

}
