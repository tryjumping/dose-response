#[feature(macro_rules, struct_variant, globs)];

extern mod extra;

use std::io;
use std::io::File;

use std::rand::{Rng, IsaacRng, SeedableRng};
use std::os;
use std::to_bytes::{ToBytes};

use components::{ComponentManager, ID, Computer, Side, Position};
use engine::{Display, MainLoopState, Key};
use extra::ringbuf::RingBuf;
use extra::container::Deque;
use extra::time;
use systems::input::commands;
use systems::input::commands::Command;

pub mod components;
mod engine;
pub mod systems;
pub mod tcod;
pub mod world_gen;
pub mod world;


pub struct GameState {
    entities: ComponentManager,
    resources: Resources,
}

pub struct Resources {
    side: Side,
    world_size: (int, int),
    turn: int,
    rng: IsaacRng,
    commands: RingBuf<Command>,
    command_logger: CommandLogger,
    player_id: ID,
    cheating: bool,
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

fn f6_pressed(keys: &RingBuf<Key>) -> bool {
    for &key in keys.iter() {
        if key.code == 55 { return true; }
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
          keys: &mut RingBuf<Key>,
          dt_s: f32) -> MainLoopState<GameState> {
    if escape_pressed(keys) { return engine::Exit }
    if f5_pressed(keys) {
        println!("Restarting game");
        keys.clear();
        let (width, height) = state.resources.world_size;
        let mut state = new_game_state(width, height);
        let player = world::player_entity(&mut state.entities);
        let player_pos = Position{x: width / 2, y: height / 2};
        state.entities.set_position(player, player_pos);
        assert!(state.entities.has_entity(state.resources.player_id));
        world::populate_world(&mut state.entities,
                              state.resources.world_size,
                              player_pos,
                              &mut state.resources.rng,
                              world_gen::forrest);
        return engine::NewState(state);
    }
    if f6_pressed(keys) {
        state.resources.cheating = !state.resources.cheating;
        println!("Cheating set to: {}", state.resources.cheating);
    }

    let systems = [
        systems::turn_tick_counter::system,
        systems::effect_duration::system,
        systems::addiction::system,
        systems::input::system,
        systems::leave_area::system,
        systems::player_dead::system,
        systems::ai::system,
        systems::dose::system,
        systems::panic::system,
        systems::stun::system,
        systems::movement::system,
        systems::interaction::system,
        systems::bump::system,
        systems::combat::system,
        systems::will::system,
        systems::exploration::system,
        systems::fade_out::system,
    ];

    process_input(keys, &mut state.resources.commands);
    for id in state.entities.iter() {
        for &sys in systems.iter() {
            if state.entities.has_entity(id) {
                sys(id, &mut state.entities, &mut state.resources);
            }
        }
        if state.entities.has_entity(id) {
            systems::color_fade::system(id, &mut state.entities, &mut state.resources, dt_s);
            systems::tile::system(id,
                                  &mut state.entities,
                                  &mut state.resources,
                                  display);
        }
    }
    systems::gui::system(&state.entities,
                         &mut state.resources,
                         display);
    systems::turn::system(&mut state.entities,
                          &mut state.resources);
    systems::addiction_graphics::system(&mut state.entities,
                                        &mut state.resources,
                                        display);
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
    fn write(&mut self, _buf: &[u8]) {}
}

struct CommandLogger {
    priv writer: ~io::Writer,
}

impl CommandLogger {
    fn log(&self, command: Command) {
        self.writer.write((command.to_str() + "\n").as_bytes());
        self.writer.flush();
    }
}

fn new_game_state(width: int, height: int) -> GameState {
    let mut rng = IsaacRng::new();
    let seed: ~[u8];
    let commands = RingBuf::new();
    let seed = rng.gen_range(0u32, 10000);
    rng.reseed([seed]);
    let cur_time = time::now();
    let timestamp = time::strftime("%FT%T.", &cur_time) +
        (cur_time.tm_nsec / 1000000).to_str();
    let replay_dir = &Path::init("./replays/");
    if !replay_dir.exists() {
        io::fs::mkdir_recursive(replay_dir, 0b111101101);
    }
    let replay_path = &replay_dir.join("replay-" + timestamp);
    let writer = match File::create(replay_path) {
        Some(f) => f,
        None => fail!("Failed to create the replay file.")
    };
    writer.write((seed.to_str() + "\n").as_bytes());
    let logger = CommandLogger{writer: ~writer as ~Writer};
    let ecm = ComponentManager::new();
    GameState {
        entities: ecm,
        resources: Resources{
            commands: commands,
            command_logger: logger,
            rng: rng,
            side: Computer,
            turn: 0,
            player_id: ID(0),
            cheating: false,
            world_size: (width, height),
        },
    }
}

fn replay_game_state(width: int, height: int) -> GameState {
    let mut commands = RingBuf::new();
    let replay_path = &Path::init(os::args()[1]);
    let mut seed: ~[u8];
    match File::open(replay_path) {
        Some(file) => {
            let contents = std::str::from_utf8(file.read_to_end());
            let mut lines_it = contents.lines();
            match lines_it.next() {
                Some(seed_str) => seed = seed_from_str(seed_str),
                None => fail!("The replay file is empty"),
            }
            for line in lines_it {
                match from_str(line) {
                    Some(command) => commands.push_back(command),
                    None => fail!("Unknown command: {}", line),
                }
            }
        },
        None => fail!("Failed to read the replay file: {}", replay_path.display())
    }
    println!("Replaying game log: '{}'", replay_path.display());
    let rng: IsaacRng = SeedableRng::from_seed(seed);
    let logger = CommandLogger{writer: ~NullWriter as ~Writer};
    let ecm = ComponentManager::new();
    GameState {
        entities: ecm,
        resources: Resources {
            commands: commands,
            rng: rng,
            command_logger: logger,
            side: Computer,
            turn: 0,
            player_id: ID(0),
            cheating: false,
            world_size: (width, height),
        },
    }
}


fn main() {
    let (width, height) = (80, 50);
    let title = "Dose Response";
    let font_path = Path::init("./fonts/dejavu16x16_gs_tc.png");

    let mut game_state = match os::args().len() {
        1 => {  // Run the game with a new seed, create the replay log
            new_game_state(width, height)
        },
        2 => {  // Replay the game from the entered log
            replay_game_state(width, height)
        },
        _ => fail!("You must pass either pass zero or one arguments."),
    };

    let player = world::player_entity(&mut game_state.entities);
    game_state.entities.set_position(player, Position{x: width / 2, y: height / 2});
    let player_pos = game_state.entities.get_position(player);
    assert_eq!(player, ID(0));
    game_state.resources.player_id = player;
    world::populate_world(&mut game_state.entities,
                          game_state.resources.world_size,
                          player_pos,
                          &mut game_state.resources.rng,
                          world_gen::forrest);

    engine::main_loop(width, height, title, font_path,
                      game_state,
                      update);
}
