use emhyr::{Entity};

use engine::{Color};


struct AI{behaviour: ai::Behaviour, state: ai::State}
struct AcceptsUserInput;
struct Addiction{tolerance: int, drop_per_turn: int, last_turn: int}
struct AnxietyKillCounter{count: int, threshold: int}
struct AttackTarget(Entity);
enum   AttackType {Kill, Stun{duration: int}, Panic{duration: int}, ModifyAttributes}
struct AttributeModifier{state_of_mind: int, will: int}
struct Attributes{state_of_mind: int, will: int}
struct Background;
struct Bump(Entity);
struct ColorAnimation{color: Color, progress: f32, forward: bool}
struct Corpse{glyph: char, color: Color, solid: bool}
#[deriving(Eq, Clone, Show)]
struct Destination {x: int, y: int}
struct Dose{tolerance_modifier: int, resist_radius: int}
struct Edible;
struct Exploration{radius: int}
struct Explored;
struct ExplosionEffect{radius: int}
struct FadeColor{from: Color, to: Color, duration_s: f32, repetitions: Repetitions}
struct FadeOut{to: Color, duration_s: f32}
struct FadingOut;
struct InventoryItem{owner: Entity}
struct Monster{kind: MonsterKind}
struct Panicking{turn: int, duration: int}
struct Pickable;
#[deriving(Eq, Clone, Show)]
struct Position {x: int, y: int}
struct Solid;
struct Stunned{turn: int, duration: int}
struct Tile{level: uint, glyph: char, color: Color}
struct Turn{side: Side, ap: int, max_ap: int, spent_this_tick: int}
struct UsingItem{item: Entity}


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
