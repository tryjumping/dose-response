extern mod extra;

use std::io;

use std::rand;
use std::rand::Rng;
use std::os;
use std::to_bytes::{ToBytes};
use entity_manager::EntityManager;

use components::{GameObject, Computer};
use engine::{Display, MainLoopState, Key};
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
pub mod world_gen;
mod world;


struct GameState {
    entities: EntityManager<GameObject>,
    commands: ~RingBuf<Command>,
    rng: rand::IsaacRng,
    logger: CommandLogger,
    map: map::Map,
    current_side: components::Side,
    current_turn: int,
    player_id: entity_manager::ID,
}

fn escape_pressed(keys: &RingBuf<Key>) -> bool {
    for &key in keys.iter() {
        if key.char as int == 27 { return true; }
    }
    false
}

fn f5_pressed(keys: &RingBuf<Key>) -> bool {
    for &key in keys.iter() {
        if key.code == 54 { return true; }
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
          keys: &mut RingBuf<Key>) -> MainLoopState<GameState> {
    if escape_pressed(keys) { return engine::Exit }
    if f5_pressed(keys) {
        println!("Restarting game");
        keys.clear();
        let mut state = new_game_state(state.map.width, state.map.height);
        let player = world::player_entity();
        state.entities.add(player);
        assert!(state.entities.get_ref(state.player_id).is_some());
        world::populate_world(&mut state.entities,
                              &mut state.map,
                              &mut state.rng,
                              world_gen::forrest);
        return engine::NewState(state);
    }

    process_input(keys, state.commands);
    for id in state.entities.id_iter() {
        if state.entities.get_ref(id).is_none() {
            loop
        }
        let ecm = &mut state.entities;
        systems::turn_tick_counter_system(id, ecm, state.current_side);
        systems::effect_duration::run(id, ecm, state.current_turn);
        systems::addiction::run(id, ecm, &mut state.map, state.current_turn);
        systems::input_system(id, ecm, state.commands, state.logger, state.current_side);
        systems::ai::process(id, ecm, &mut state.rng, &state.map, state.current_side, state.player_id);
        systems::dose::run(id, ecm, &state.map);
        systems::path_system(id, ecm, &mut state.map);
        systems::movement::run(id, ecm, &mut state.rng, &mut state.map);
        systems::interaction::run(id, ecm, &mut state.map);
        systems::bump_system(id, ecm);
        systems::combat::run(id, ecm, &mut state.map, state.current_turn);
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

fn new_game_state(width: uint, height: uint) -> GameState {
    let mut rng = rand::IsaacRng::new();
    let seed: ~[u8];
    let writer: @io::Writer;
    let commands = ~RingBuf::new();
    let seed_int = rng.gen_integer_range(0, 10000);
    seed = seed_int.to_bytes(true);
    rng = rand::IsaacRng::new_seeded(seed);
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
    let logger = CommandLogger{writer: writer};
    let ecm = EntityManager::new();
    let map = map::Map::new(width, height);
    GameState {
        entities: ecm,
        commands: commands,
        rng: rng,
        logger: logger,
        map: map,
        current_side: Computer,
        current_turn: 0,
        player_id: entity_manager::ID(0),
    }
}

fn replay_game_state(width: uint, height: uint) -> GameState {
    let mut commands = ~RingBuf::new();
    let replay_path = &Path(os::args()[1]);
    let mut seed: ~[u8];
    let writer: @Writer;
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
    let rng = rand::IsaacRng::new_seeded(seed);
    let logger = CommandLogger{writer: writer};
    let ecm = EntityManager::new();
    let map = map::Map::new(width, height);
    GameState {
        entities: ecm,
        commands: commands,
        rng: rng,
        logger: logger,
        map: map,
        current_side: Computer,
        current_turn: 0,
        player_id: entity_manager::ID(0),
    }
}


fn main() {
    let (width, height) = (80, 50);
    let title = "Dose Response";
    let font_path = Path("./fonts/dejavu16x16_gs_tc.png");

    let mut game_state = match os::args().len() {
        1 => {  // Run the game with a new seed, create the replay log
            new_game_state(width, height)
        },
        2 => {  // Replay the game from the entered log
            replay_game_state(width, height)
        },
        _ => fail!("You must pass either pass zero or one arguments."),
    };

    let player = world::player_entity();
    game_state.entities.add(player);
    assert!(game_state.entities.get_ref(game_state.player_id).is_some());
    world::populate_world(&mut game_state.entities,
                          &mut game_state.map,
                          &mut game_state.rng,
                          world_gen::forrest);

    engine::main_loop(width, height, title, font_path,
                      game_state,
                      update);
}
