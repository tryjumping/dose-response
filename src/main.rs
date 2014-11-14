#![feature(macro_rules, struct_variant, globs, phase, link_args, unboxed_closures)]

extern crate collections;
extern crate libc;
extern crate time;


#[phase(plugin, link)] extern crate emhyr;
extern crate tcod;

use std::collections::{RingBuf, HashMap};
use std::collections::hash_map::{Occupied, Vacant};
use std::time::Duration;
use std::io;
use std::io::File;
use std::io::util::NullWriter;
use std::io::fs:: PathExtensions;
use std::os;
use std::vec::MoveItems;

use std::rand::{IsaacRng, SeedableRng};
use std::rand;
use tcod::{KeyState, Printable, Special};

use components::{Computer, Position, Side};
use emhyr::{Change, Entity, World};
use engine::{Engine, key};
use level::Level;
use systems::input::commands;
use systems::input::commands::Command;

mod components;
mod engine;
mod entity_util;
mod level;
mod point;
mod systems;
mod world_gen;
mod world;

pub struct GameState {
    world: World,
    level: Level,
    world_size: (int, int),
    rng: IsaacRng,
    commands: RingBuf<Command>,
    command_logger: CommandLogger,
    side: Side,
    turn: int,
    player: Entity,
    cheating: bool,
    replay: bool,
    paused: bool,
}

impl GameState {
    fn new(width: int, height: int,
           commands: RingBuf<Command>,
           log_writer: Box<Writer+'static>,
           seed: u32,
           cheating: bool,
           replay: bool) -> GameState {
        let seed_arr: &[_] = &[seed];
        let mut world = World::new();
        let player = world.new_entity();
        GameState {
            world: world,
            level: Level::new(width, height - 2),
            world_size: (width, height),
            rng: SeedableRng::from_seed(seed_arr),
            commands: commands,
            command_logger: CommandLogger{writer: log_writer},
            side: Computer,
            turn: 0,
            player: player,
            cheating: cheating,
            replay: replay,
            paused: false,
        }
    }
}

fn key_pressed(keys: &RingBuf<KeyState>, key_code: tcod::Key) -> bool {
    for &pressed_key in keys.iter() {
        if pressed_key.key == key_code {
            return true;
        }
    }
    false
}

/// Consumes the first occurence of the given key in the buffer.
///
/// Returns `true` if the key has been in the buffer.
fn read_key(keys: &mut RingBuf<KeyState>, key: tcod::Key) -> bool {
    let mut len = keys.len();
    let mut processed = 0;
    let mut found = false;
    while processed < len {
        match keys.pop_front() {
            Some(pressed_key) if !found && pressed_key.key == key => {
                len -= 1;
                found = true;
            }
            Some(pressed_key) => {
                keys.push_back(pressed_key);
            }
            None => return false
        }
        processed += 1;
    }
    return found;
}


fn ctrl(key: tcod::KeyState) -> bool {
    key.left_ctrl || key.right_ctrl
}

fn process_input(keys: &mut RingBuf<tcod::KeyState>, commands: &mut RingBuf<Command>) {
    // TODO: switch to DList and consume it with `mut_iter`.
    loop {
        match keys.pop_front() {
            Some(key) => {
                match key.key {
                    Special(key::Up) => commands.push_back(commands::N),
                    Special(key::Down) => commands.push_back(commands::S),
                    Special(key::Left) => match (ctrl(key), key.shift) {
                        (false, true) => commands.push_back(commands::NW),
                        (true, false) => commands.push_back(commands::SW),
                        _ => commands.push_back(commands::W),
                    },
                    Special(key::Right) => match (ctrl(key), key.shift) {
                        (false, true) => commands.push_back(commands::NE),
                        (true, false) => commands.push_back(commands::SE),
                        _ => commands.push_back(commands::E),
                    },
                    Printable('e') => {
                        commands.push_back(commands::Eat);
                    }
                    _ => (),
                }
            },
            None => break,
        }
    }
}

fn update(mut state: GameState, dt_s: f32, engine: &engine::Engine) -> Option<GameState> {
    let keys = engine.keys();
    if key_pressed(&*keys.borrow(), Special(key::Escape)) {
        use std::cmp::{Less, Equal, Greater};
        let mut stats = state.world.generate_stats();
        stats.sort_by(|&(_, time_1, _), &(_, time_2, _)|
                      if time_1 < time_2 { Greater }
                      else if time_1 > time_2 { Less }
                      else if time_1 == time_2 { Equal }
                      else { println!("{}, {}", time_1, time_2); unreachable!() });
        println!("Mean update time in miliseconds per system:");
        for &(system_name, average_time_ns, system_only_ns) in stats.iter() {
            println!("{:4.3f}\t{:4.3f}\t{}",
                     average_time_ns / 1000000.0,
                     system_only_ns / 1000000.0,
                     system_name);
        }
        let total_time_ns = stats.iter()
            .map(|&(_, time_ns, _)| time_ns)
            .fold(0.0, |a, b| a + b);
        println!("\nAggregate mean time per tick: {}ms", total_time_ns / 1000000.0);
        return None;
    }
    if key_pressed(&*keys.borrow(), Special(key::F5)) {
        println!("Restarting game");
        keys.borrow_mut().clear();
        let (width, height) = state.world_size;
        let mut state = new_game_state(width, height);
        initialise_world(&mut state, engine);
        return Some(state);
    }

    if key_pressed(&*keys.borrow(), Special(key::F6)) {
        state.cheating = !state.cheating;
        println!("Cheating set to: {}", state.cheating);
    }

    state.paused = if state.replay && read_key(&mut *keys.borrow_mut(), Special(key::Spacebar)) {
        if !state.paused {println!("Pausing the replay")};
        !state.paused
    } else {
        state.paused
    };

    // TODO: move this to an input system or something?
    // TODO: this is the pause/step-on-replay code
    // let mut input_system = if state.paused {
    //     null_input_system
    // } else {
    //     systems::input::system
    // };

    // // Move one step forward in the paused replay
    // if state.paused && read_key(&mut *keys.borrow_mut(), Special(key::Right)) {
    //     input_system = systems::input::system;
    // }

    process_input(&mut *keys.borrow_mut(), &mut state.commands);
    state.level.render(&mut *engine.display().borrow_mut());
    Some(state)
}



struct CommandLogger {
    writer: Box<Writer+'static>,
}

impl CommandLogger {
    fn log(&mut self, command: Command) {
        self.writer.write_line(command.to_string().as_slice()).unwrap();
    }
}

fn new_game_state(width: int, height: int) -> GameState {
    let commands = RingBuf::new();
    let seed = rand::random::<u32>();
    let cur_time = time::now();
    let timestamp = format!("{}{}", time::strftime("%FT%T.", &cur_time),
                            (cur_time.tm_nsec / 1000000).to_string());
    let replay_dir = &Path::new("./replays/");
    if !replay_dir.exists() {
        io::fs::mkdir_recursive(replay_dir,
                                io::FilePermission::from_bits(0b111101101).unwrap()).unwrap();
    }
    let replay_path = &replay_dir.join(format!("replay-{}", timestamp));
    let mut writer = match File::create(replay_path) {
        Ok(f) => box f,
        Err(msg) => panic!("Failed to create the replay file. {}", msg)
    };
    println!("Recording the gameplay to '{}'", replay_path.display());
    writer.write_line(seed.to_string().as_slice()).unwrap();
    GameState::new(width, height, commands, writer, seed, false, false)
}

fn replay_game_state(width: int, height: int) -> GameState {
    let mut commands = RingBuf::new();
    let replay_path = &Path::new(os::args()[1].as_slice());
    let mut seed: u32;
    match File::open(replay_path) {
        Ok(mut file) => {
            let bin_data = file.read_to_end().unwrap();
            let contents = std::str::from_utf8(bin_data.slice(0, bin_data.len()));
            let mut lines = contents.unwrap().lines();
            match lines.next() {
                Some(seed_str) => match from_str(seed_str) {
                    Some(parsed_seed) => seed = parsed_seed,
                    None => panic!("The seed must be a number.")
                },
                None => panic!("The replay file is empty."),
            }
            for line in lines {
                match from_str(line) {
                    Some(command) => commands.push_back(command),
                    None => panic!("Unknown command: {}", line),
                }
            }
        },
        Err(msg) => panic!("Failed to read the replay file: {}. Reason: {}",
                          replay_path.display(), msg)
    }
    println!("Replaying game log: '{}'", replay_path.display());
    GameState::new(width, height, commands, box NullWriter, seed, true, true)
}

fn initialise_world(game_state: &mut GameState, engine: &Engine) {
    let (width, height) = game_state.world_size;
    let player = game_state.player;
    world::create_player(&mut game_state.world.cs, player);
    game_state.world.cs.set(Position{x: width / 2, y: height / 2}, player);
    let player_pos: Position = game_state.world.cs.get(player);
    world::populate_world(&mut game_state.world,
                          game_state.world_size,
                          player_pos,
                          &mut game_state.rng,
                          world_gen::forrest);
}

fn main() {
    let (width, height) = (80, 50);
    let title = "Dose Response";
    let font_path = Path::new("./fonts/dejavu16x16_gs_tc.png");

    let mut game_state = match os::args().len() {
        1 => {  // Run the game with a new seed, create the replay log
            new_game_state(width, height)
        },
        2 => {  // Replay the game from the entered log
            replay_game_state(width, height)
        },
        _ => panic!("You must pass either pass zero or one arguments."),
    };

    let mut engine = Engine::new(width, height, title, font_path.clone());
    initialise_world(&mut game_state, &engine);
    engine.main_loop(game_state, update);
}
