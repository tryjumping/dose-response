extern mod extra;

use std::io;

use std::rand::{Rng, IsaacRng};
use std::os;
use std::to_bytes::{ToBytes};
use entity_manager::EntityManager;

use components::{GameObject, Computer, Side};
use engine::{Display, MainLoopState, Key};
use extra::ringbuf::RingBuf;
use extra::container::Deque;
use extra::time;
use map::Map;
use systems::input::commands;
use systems::input::commands::Command;

pub mod components;
mod engine;
pub mod entity_manager;
pub mod map;
pub mod systems;
pub mod tcod;
pub mod world_gen;
pub mod world;


pub struct GameState {
    entities: EntityManager<GameObject>,
    resources: Resources,
    player_id: entity_manager::ID,
}

pub struct Resources {
    map: Map,
    side: Side,
    turn: int,
    rng: IsaacRng,
    commands: RingBuf<Command>,
    command_logger: CommandLogger,
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
                    14 => commands.push_back(commands::N),
                    // Down
                    17 => commands.push_back(commands::S),
                    // Left
                    15 => match (key.ctrl(), key.shift()) {
                        (false, true) => commands.push_back(commands::NW),
                        (true, false) => commands.push_back(commands::SW),
                        _ => commands.push_back(commands::W),
                    },
                    // Right
                    16 => match (key.ctrl(), key.shift()) {
                        (false, true) => commands.push_back(commands::NE),
                        (true, false) => commands.push_back(commands::SE),
                        _ => commands.push_back(commands::E),
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
        let mut state = new_game_state(state.resources.map.width, state.resources.map.height);
        let player = world::player_entity();
        state.entities.add(player);
        assert!(state.entities.get_ref(state.player_id).is_some());
        world::populate_world(&mut state.entities,
                              &mut state.resources.map,
                              &mut state.resources.rng,
                              world_gen::forrest);
        return engine::NewState(state);
    }

    process_input(keys, &mut state.resources.commands);
    for id in state.entities.id_iter() {
        let mut abort_early = false;
        if state.entities.get_ref(id).is_none() {
            loop
        }
        systems::turn_tick_counter::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::effect_duration::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::addiction::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::input::system(id, &mut state.entities, state.player_id, &mut state.resources);

        // TODO: add `clear` to ecm, handle abort early differently
        systems::leave_area::system(state.player_id, state, &mut abort_early);
        if abort_early {
            return engine::Running;
        }
        systems::ai::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::panic::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::stun::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::dose::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::path::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::movement::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::interaction::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::bump::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::combat::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::will::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::idle_ai::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::player_dead::system(id, &mut state.entities, state.player_id, &mut state.resources);
        systems::tile::system(id, &state.entities, display);
    }
    systems::gui::system(&state.entities,
                         state.player_id,
                         display,
                         &mut state.resources);
    systems::turn::system(&mut state.entities,
                          &mut state.resources);
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
    let mut rng = IsaacRng::new();
    let seed: ~[u8];
    let writer: @io::Writer;
    let commands = RingBuf::new();
    let seed_int = rng.gen_integer_range(0, 10000);
    seed = seed_int.to_bytes(true);
    rng = IsaacRng::new_seeded(seed);
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
        resources: Resources{
            commands: commands,
            command_logger: logger,
            map: map,
            rng: rng,
            side: Computer,
            turn: 0,
        },
        player_id: entity_manager::ID(0),
    }
}

fn replay_game_state(width: uint, height: uint) -> GameState {
    let mut commands = RingBuf::new();
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
    let rng = IsaacRng::new_seeded(seed);
    let logger = CommandLogger{writer: writer};
    let ecm = EntityManager::new();
    let map = map::Map::new(width, height);
    GameState {
        entities: ecm,
        resources: Resources {
            commands: commands,
            rng: rng,
            command_logger: logger,
            map: map,
            side: Computer,
            turn: 0,
        },
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
                          &mut game_state.resources.map,
                          &mut game_state.resources.rng,
                          world_gen::forrest);

    engine::main_loop(width, height, title, font_path,
                      game_state,
                      update);
}
