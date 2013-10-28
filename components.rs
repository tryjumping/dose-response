struct AI{behaviour: ai::Behaviour, state: ai::State}
struct AcceptsUserInput
struct Addiction{tolerance: int, drop_per_turn: int, last_turn: int}
struct AnxietyKillCounter{count: int, threshold: int}
struct AttackTarget(ID)
enum AttackType {Kill, Stun{duration: int}, Panic{duration: int}, ModifyAttributes}
struct AttributeModifier{state_of_mind: int, will: int}
struct Attributes{state_of_mind: int, will: int}
struct Background
struct Bump(ID)
struct Destination {x: int, y: int}
struct Dose{tolerance_modifier: int, resist_radius: int}
struct ExplosionEffect{radius: int}
struct Monster{kind: MonsterKind}
struct Panicking{turn: int, duration: int}
struct Position {x: int, y: int}
struct Solid
struct Stunned{turn: int, duration: int}
struct Tile{level: uint, glyph: char, color: Color}
struct Turn{side: Side, ap: int, max_ap: int, spent_this_tick: int}
