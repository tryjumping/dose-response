extern mod extra;

use std::io;

use std::rand;
use std::rand::Rng;
use std::os;
use std::to_bytes::{ToBytes};
use entity_manager::EntityManager;

use c = components;
use engine::{Display, Color, MainLoopState, Key};
use extra::ringbuf::RingBuf;
use extra::container::Deque;
use extra::time;
use systems::{Command};

pub mod components;
mod engine;
pub mod entity_manager;
pub mod map;
mod systems;
pub mod tcod;
mod world_gen;


struct GameState {
    entities: EntityManager<c::GameObject>,
    commands: ~RingBuf<Command>,
    rng: rand::IsaacRng,
    logger: CommandLogger,
    map: map::Map,
    current_side: components::Side,
    current_turn: int,
    player_id: entity_manager::ID,
}

impl world_gen::WorldItem {
    fn to_glyph(self) -> char {
        match self {
            world_gen::Empty => '.',
            world_gen::Tree => '#',
            world_gen::Dose => 'i',
            world_gen::StrongDose => 'I',
            world_gen::Anxiety => 'a',
            world_gen::Depression => 'D',
            world_gen::Hunger => 'h',
            world_gen::Voices => 'v',
            world_gen::Shadows => 's',
        }
    }

    fn to_color(self) -> Color {
        match self {
            world_gen::Empty => col::empty_tile,
            world_gen::Tree => col::tree_1,
            world_gen::Dose => col::dose,
            world_gen::StrongDose => col::dose,

            world_gen::Anxiety => col::anxiety,
            world_gen::Depression => col::depression,
            world_gen::Hunger => col::hunger,
            world_gen::Voices => col::voices,
            world_gen::Shadows => col::shadows,
        }
    }

    fn is_solid(self) -> bool {
        match self {
            world_gen::Empty | world_gen::Dose | world_gen::StrongDose => false,
            _ => true,
        }
    }

    fn is_monster(self) -> bool {
        match self {
            world_gen::Anxiety |
            world_gen::Depression |
            world_gen::Hunger |
            world_gen::Voices |
            world_gen::Shadows => true,
            _ => false,
        }
    }
}

mod col {
    use engine::Color;

    pub static background: Color = Color(0, 0, 0);
    pub static dim_background: Color = Color(15, 15, 15);
    pub static foreground: Color = Color(255, 255, 255);
    pub static anxiety: Color = Color(191,0,0);
    pub static depression: Color = Color(111,63,255);
    pub static hunger: Color = Color(127,101,63);
    pub static voices: Color = Color(95,95,95);
    pub static shadows: Color = Color(95,95,95);
    pub static player: Color = Color(255,255,255);
    pub static empty_tile: Color = Color(223,223,223);
    pub static dose: Color = Color(114,184,255);
    pub static dose_glow: Color = Color(0,63,47);
    pub static tree_1: Color = Color(0,191,0);
    pub static tree_2: Color = Color(0,255,0);
    pub static tree_3: Color = Color(63,255,63);
}

fn initial_state(width: uint, height: uint, commands: ~RingBuf<Command>,
                 rng: rand::IsaacRng, logger: CommandLogger) -> ~GameState {
    let mut state = ~GameState {
        entities: EntityManager::new(),
        commands: commands,
        rng: rng,
        logger: logger,
        map: map::Map::new(width, height),
        current_side: c::Computer,
        current_turn: 0,
        player_id: entity_manager::ID(0),
    };
    let mut player = c::GameObject::new();
    player.accepts_user_input = Some(c::AcceptsUserInput);
    player.attack_type = Some(c::Kill);
    player.attributes = Some(c::Attributes{state_of_mind: 20, will: 2});
    player.addiction = Some(c::Addiction{
            tolerance: 0,
            drop_per_turn: 1,
            last_turn: 1,
        });
    player.anxiety_kill_counter = Some(c::AnxietyKillCounter{
            count: 0,
            threshold: 10});
    player.position = Some(c::Position{x: 10, y: 20});
    player.tile = Some(c::Tile{level: 2, glyph: '@', color: col::player});
    player.turn = Some(c::Turn{side: c::Player,
                               ap: 0,
                               max_ap: 1,
                               spent_this_tick: 0,
        });
    player.solid = Some(c::Solid);
    state.entities.add(player);
    assert!(state.entities.get_ref(state.player_id).is_some());

    let world = world_gen::forrest(&mut state.rng, width, height);
    for &(x, y, item) in world.iter() {
        let mut bg = c::GameObject::new();
        bg.position = Some(c::Position{x: x, y: y});
        bg.background = Some(c::Background);
        if item == world_gen::Tree {
            bg.tile = Some(c::Tile{level: 0, glyph: item.to_glyph(), color: item.to_color()});
            bg.solid = Some(c::Solid);
        } else { // put an empty item as the background
            bg.tile = Some(c::Tile{level: 0, glyph: world_gen::Empty.to_glyph(), color: world_gen::Empty.to_color()});
        }
        state.entities.add(bg);
        if item != world_gen::Tree && item != world_gen::Empty {
            let mut e = c::GameObject::new();
            e.position = Some(c::Position{x: x, y: y});
            let mut tile_level = 1;
            if item.is_monster() {
                let behaviour = match item {
                    world_gen::Hunger => c::ai::Pack,
                    _ => c::ai::Individual,
                };
                e.ai = Some(c::AI{behaviour: behaviour, state: c::ai::Idle});
                let max_ap = if item == world_gen::Depression { 2 } else { 1 };
                e.turn = Some(c::Turn{side: c::Computer,
                                      ap: 0,
                                      max_ap: max_ap,
                                      spent_this_tick: 0,
                    });
                e.solid = Some(c::Solid);
                match item {
                    world_gen::Anxiety => {
                        e.monster = Some(c::Monster{kind: c::Anxiety});
                        e.attack_type = Some(c::ModifyAttributes);
                        e.attribute_modifier = Some(
                            c::AttributeModifier{state_of_mind: 0, will: -1});
                    }
                    world_gen::Depression => {
                        e.monster = Some(c::Monster{kind: c::Depression});
                        e.attack_type = Some(c::Kill)
                    },
                    world_gen::Hunger => {
                        e.monster = Some(c::Monster{kind: c::Hunger});
                        e.attack_type = Some(c::ModifyAttributes);
                        e.attribute_modifier = Some(
                            c::AttributeModifier{state_of_mind: -20, will: 0})
                    }
                    world_gen::Voices => {
                        e.monster = Some(c::Monster{kind: c::Voices});
                        e.attack_type = Some(c::Stun{duration: 4})
                    },
                    world_gen::Shadows => {
                        e.monster = Some(c::Monster{kind: c::Shadows});
                        e.attack_type = Some(c::Panic{duration: 4})
                    },
                    _ => unreachable!(),
                };
                tile_level = 2;
            } else if item == world_gen::Dose {
                e.dose = Some(c::Dose{tolerance_modifier: 1});
                e.attribute_modifier = Some(c::AttributeModifier{
                        state_of_mind: 40 + state.rng.gen_integer_range(-10, 11),
                        will: 0,
                    });
                e.explosion_effect = Some(c::ExplosionEffect{radius: 4});
            } else if item == world_gen::StrongDose {
                e.dose = Some(c::Dose{tolerance_modifier: 2});
                e.attribute_modifier = Some(c::AttributeModifier{
                        state_of_mind: 90 + state.rng.gen_integer_range(-15, 16),
                        will: 0,
                    });
                e.explosion_effect = Some(c::ExplosionEffect{radius: 6});
            }
            e.tile = Some(c::Tile{level: tile_level, glyph: item.to_glyph(), color: item.to_color()});
            state.entities.add(e);
        }
    }

    // Initialise the map's walkability data
    for (e, id) in state.entities.iter() {
        match e.position {
            Some(c::Position{x, y}) => {
                let walkable = match e.solid {
                    Some(_) => map::Solid,
                    None => map::Walkable,
                };
                match e.background {
                    Some(_) => state.map.set_walkability((x, y), walkable),
                    None => state.map.place_entity(*id, (x, y), walkable),
                }
            },
            None => (),
        }
    }

    state
}

fn escape_pressed(keys: &RingBuf<Key>) -> bool {
    for &key in keys.iter() {
        if key.char as int == 27 { return true; }
    }
    false
}

fn process_input(keys: &mut RingBuf<Key>, commands: &mut RingBuf<Command>) {
    // TODO: switch to DList and consume it with `mut_iter`.
    loop {
        match keys.pop_front() {
            Some(key) => {
                match key.code {
                    // Up
                    14 => commands.push_back(systems::N),
                    // Down
                    17 => commands.push_back(systems::S),
                    // Left
                    15 => match (key.ctrl(), key.shift()) {
                        (false, true) => commands.push_back(systems::NW),
                        (true, false) => commands.push_back(systems::SW),
                        _ => commands.push_back(systems::W),
                    },
                    // Right
                    16 => match (key.ctrl(), key.shift()) {
                        (false, true) => commands.push_back(systems::NE),
                        (true, false) => commands.push_back(systems::SE),
                        _ => commands.push_back(systems::E),
                    },
                    _ => (),
                }
            },
            None => break,
        }
    }
}



fn update(state: &mut GameState,
          display: &mut Display,
          keys: &mut RingBuf<Key>) -> MainLoopState {
    if escape_pressed(keys) { return engine::Exit }

    process_input(keys, state.commands);
    for id in state.entities.id_iter() {
        if state.entities.get_ref(id).is_none() {
            loop
        }
        let ecm = &mut state.entities;
        systems::turn_tick_counter_system(id, ecm, state.current_side);
        systems::input_system(id, ecm, state.commands, state.logger, state.current_side);
        systems::ai::process(id, ecm, &mut state.rng, &state.map, state.current_side, state.player_id);
        systems::path_system(id, ecm, &mut state.map);
        systems::movement::run(id, ecm, &mut state.rng, &mut state.map);
        systems::interaction::run(id, ecm, &mut state.map);
        systems::bump_system(id, ecm);
        systems::combat::run(id, ecm, &mut state.map, state.current_turn);
        systems::addiction::run(id, ecm, &mut state.map, state.current_turn);
        systems::effect_duration::run(id, ecm, state.current_turn);
        systems::will::run(id, ecm, &mut state.map);
        systems::idle_ai_system(id, ecm, state.current_side);
        systems::player_dead_system(id, ecm, state.player_id);
        systems::tile_system(id, ecm, display);
    }
    systems::gui::process(&state.entities,
                          display,
                          state.player_id,
                          state.current_turn);
    systems::turn_system::run(&mut state.entities,
                              &mut state.current_side,
                              &mut state.current_turn);
    engine::Running
}

fn seed_from_str(source: &str) -> ~[u8] {
    match from_str::<int>(source) {
        Some(n) => n.to_bytes(true),
        None => fail!("The seed must be a number"),
    }
}

struct NullWriter;

impl io::Writer for NullWriter {
    fn write(&self, _v: &[u8]) {}
    fn seek(&self, _a: int, _s: io::SeekStyle) {}
    fn tell(&self) -> uint { 0 }
    fn flush(&self) -> int { 0 }
    fn get_type(&self) -> io::WriterType { io::File }
}

struct CommandLogger {
    priv writer: @io::Writer,
}

impl CommandLogger {
    fn log(&self, command: Command) {
        self.writer.write_line(command.to_str());
        self.writer.flush();
    }
}


fn main() {
    let (width, height) = (80, 50);
    let title = "Dose Response";
    let font_path = Path("./fonts/dejavu16x16_gs_tc.png");

    let mut rng = rand::IsaacRng::new();
    let seed: ~[u8];
    let writer: @io::Writer;
    let mut commands = ~RingBuf::new();

    match os::args().len() {
        1 => {  // Run the game with a new seed, create the replay log
            let seed_int = rng.gen_integer_range(0, 10000);
            seed = seed_int.to_bytes(true);
            let cur_time = time::now();
            let timestamp = time::strftime("%FT%T.", &cur_time) +
                (cur_time.tm_nsec / 1000000).to_str();
            let replay_dir = &Path("./replays/");
            let replay_path = &replay_dir.push("replay-" + timestamp);
            if !os::path_exists(replay_dir) {
                os::mkdir_recursive(replay_dir, 0b111101101);
            }
            match io::file_writer(replay_path, [io::Create, io::Append]) {
                Ok(w) => {
                    writer = w;
                    writer.write_line(seed_int.to_str());
                },
                Err(e) => fail!(fmt!("Failed to open the replay file: %s", e)),
            };
        },
        2 => {  // Replay the game from the entered log
            let replay_path = &Path(os::args()[1]);
            match io::read_whole_file_str(replay_path) {
                Ok(contents) => {
                    let mut lines_it = contents.any_line_iter();
                    match lines_it.next() {
                        Some(seed_str) => seed = seed_from_str(seed_str),
                        None => fail!(fmt!("The replay file is empty")),
                    }
                    for line in lines_it {
                        match from_str(line) {
                            Some(command) => commands.push_back(command),
                            None => fail!(fmt!("Unknown command: %?", line)),
                        }
                    }
                    writer = @NullWriter as @Writer;
                },
                Err(e) => fail!(fmt!("Failed to read the replay file: %s", e))
            }
        },
        _ => fail!("You must pass either pass zero or one arguments."),
    };
    rng = rand::IsaacRng::new_seeded(seed);

    let logger = CommandLogger{writer: writer};
    engine::main_loop(width, height, title, font_path,
                      initial_state(width, height, commands, rng, logger),
                      update);
}
