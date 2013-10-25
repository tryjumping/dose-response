struct AI{behaviour: ai::Behaviour, state: ai::State}
struct AcceptsUserInput
struct Addiction{tolerance: int, drop_per_turn: int, last_turn: int}
struct AnxietyKillCounter{count: int, threshold: int}
struct AttackTarget(entity_manager::ID)
enum AttackType {Kill, Stun{duration: int}, Panic{duration: int}, ModifyAttributes}
struct AttributeModifier{state_of_mind: int, will: int}
struct Attributes{state_of_mind: int, will: int}
struct Background
struct Bump(entity_manager::ID)
struct ExplosionEffect{radius: int}
struct Monster{kind: MonsterKind}
struct Position {x: int, y: int}
struct Destination {x: int, y: int}
struct Dose{tolerance_modifier: int, resist_radius: int}
struct Solid
struct Stunned{turn: int, duration: int}
struct Panicking{turn: int, duration: int}
struct Path(~map::Path) //noeq
struct Tile{level: uint, glyph: char, color: Color}
struct Turn{side: Side, ap: int, max_ap: int, spent_this_tick: int}
