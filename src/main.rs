#![crate_id = "dose-response#0.1.0"]
#![feature(macro_rules, struct_variant, globs)]

extern crate collections;
extern crate libc;
extern crate rand;
extern crate time;


extern crate emhyr;
extern crate tcod;


use std::io;
use std::io::File;
use std::io::util::NullWriter;
use std::os;
use std::rc::Rc;
use std::cell::{Cell, RefCell};

use collections::{Deque, RingBuf};
use rand::{Rng, IsaacRng, SeedableRng};
use tcod::{KeyState, Printable, Special};

use components::{Computer, Position, Side};
use ecm::{ComponentManager, ECM, Entity, System, World};
use engine::{Display, key};
use systems::input::commands;
use systems::input::commands::Command;

mod components;
mod ecm;
mod engine;
mod entity_util;
mod point;
mod systems;
mod world_gen;
mod world;


pub struct GameState {
    world: World<ECM>,
    side: Rc<RefCell<Side>>,
    world_size: (int, int),
    turn: Rc<RefCell<int>>,
    rng: Rc<RefCell<IsaacRng>>,
    commands: Rc<RefCell<RingBuf<Command>>>,
    command_logger: Rc<RefCell<CommandLogger>>,
    player: Entity,
    cheating: Rc<RefCell<bool>>,
    replay: bool,
    paused: bool,
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

pub fn null_input_system(_e: Entity, _ecm: &mut ECM) {}

fn update(mut state: GameState, dt_s: f32, engine: &engine::Engine) -> Option<GameState> {
    let keys = engine.keys();
    if key_pressed(&*keys.borrow(), Special(key::Escape)) { return None }
    if key_pressed(&*keys.borrow(), Special(key::F5)) {
        println!("Restarting game");
        keys.borrow_mut().clear();
        let (width, height) = state.world_size;
        let mut state = new_game_state(width, height);
        let player = state.world.ecm.borrow_mut().new_entity();
        world::create_player(&mut *state.world.ecm.borrow_mut(), player);
        let player_pos = Position{x: width / 2, y: height / 2};
        state.world.ecm.borrow_mut().set(player, player_pos);
        world::populate_world(&mut *state.world.ecm.borrow_mut(),
                              state.world_size,
                              player_pos,
                              &mut *state.rng.borrow_mut(),
                              world_gen::forrest);
        return Some(state);
    }

    if key_pressed(&*keys.borrow(), Special(key::F6)) {
        let cheating = !*state.cheating.borrow();
        *state.cheating.borrow_mut() = cheating;
        println!("Cheating set to: {}", cheating);
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

    process_input(&mut *keys.borrow_mut(), &mut *state.commands.borrow_mut());
    state.world.update((dt_s * 1000.0) as uint);
    Some(state)
}


// TODO: no longer needed?
fn write_line(writer: &mut Writer, line: &str) {
    let line_with_nl = String::from_str(line).append("\n");
    writer.write(line_with_nl.as_bytes());
    writer.flush();
}

fn rc_mut<T>(val: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(val))
}


struct CommandLogger {
    writer: Box<Writer>,
}

impl CommandLogger {
    fn log(&mut self, command: Command) {
        write_line(self.writer, command.to_str().as_slice());
    }
}

fn new_game_state(width: int, height: int) -> GameState {
    let mut rng = IsaacRng::new_unseeded();
    let commands = RingBuf::new();
    let seed = rng.gen_range(0u32, 10000);
    rng.reseed([seed]);
    let cur_time = time::now();
    let timestamp = time::strftime("%FT%T.", &cur_time).append((cur_time.tm_nsec / 1000000).to_str().as_slice());
    let replay_dir = &Path::new("./replays/");
    if !replay_dir.exists() {
        io::fs::mkdir_recursive(replay_dir, io::FilePermission::from_bits(0b111101101).unwrap());
    }
    let replay_path = &replay_dir.join(String::from_str("replay-").append(timestamp.as_slice()));
    let mut writer = match File::create(replay_path) {
        Ok(f) => box f,
        Err(msg) => fail!("Failed to create the replay file. {}", msg)
    };
    write_line(&mut *writer as &mut Writer, seed.to_str().as_slice());
    let logger = CommandLogger{writer: writer};
    let mut ecm = ECM::new();
    let player = ecm.new_entity();
    GameState {
        world: World::new(ecm),
        commands: rc_mut(commands),
        command_logger: rc_mut(logger),
        rng: rc_mut(rng),
        side: rc_mut(Computer),
        turn: rc_mut(0),
        player: player,
        cheating: rc_mut(false),
        replay: false,
        paused: false,
        world_size: (width, height),
    }
}

fn replay_game_state(width: int, height: int) -> GameState {
    let mut commands = RingBuf::new();
    let replay_path = &Path::new(os::args().get(1).as_slice());
    let mut seed: u32;
    match File::open(replay_path) {
        Ok(mut file) => {
            let bin_data = file.read_to_end().unwrap();
            let contents = std::str::from_utf8(bin_data.slice(0, bin_data.len()));
            let mut lines = contents.unwrap().lines();
            match lines.next() {
                Some(seed_str) => match from_str(seed_str) {
                    Some(parsed_seed) => seed = parsed_seed,
                    None => fail!("The seed must be a number.")
                },
                None => fail!("The replay file is empty."),
            }
            for line in lines {
                match from_str(line) {
                    Some(command) => commands.push_back(command),
                    None => fail!("Unknown command: {}", line),
                }
            }
        },
        Err(msg) => fail!("Failed to read the replay file: {}. Reason: {}",
                          replay_path.display(), msg)
    }
    println!("Replaying game log: '{}'", replay_path.display());
    let rng: IsaacRng = SeedableRng::from_seed(&[seed]);
    let logger = CommandLogger{writer: box NullWriter};
    let mut ecm = ECM::new();
    let player = ecm.new_entity();
    GameState {
        world: World::new(ecm),
        commands: rc_mut(commands),
        rng: rc_mut(rng),
        command_logger: rc_mut(logger),
        side: rc_mut(Computer),
        turn: rc_mut(0),
        player: player,
        cheating: rc_mut(false),
        replay: true,
        paused: false,
        world_size: (width, height),
    }
}


fn main() {
    use emhyr::World;
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
        _ => fail!("You must pass either pass zero or one arguments."),
    };

    let mut engine = engine::Engine::new(width, height, title, font_path.clone());

    let player = game_state.player;
    world::create_player(&mut *game_state.world.ecm.borrow_mut(), player);
    game_state.world.ecm.borrow_mut().set(player, Position{x: width / 2, y: height / 2});
    let player_pos: Position = game_state.world.ecm.borrow().get(player);
    world::populate_world(&mut *game_state.world.ecm.borrow_mut(),
                          game_state.world_size,
                          player_pos,
                          &mut *game_state.rng.borrow_mut(),
                          world_gen::forrest);

    // Appease the borrow checker: we can't do world.ecm inside of
    // world.add_system() because that's a double borrow:
    let ecm = game_state.world.ecm();
    let player_rc = rc_mut(player);
    let world_size_rc = rc_mut(game_state.world_size);

    game_state.world.add_system(box systems::turn_tick_counter::TurnTickCounterSystem::new(
        ecm.clone(),
        game_state.side.clone()));
    game_state.world.add_system(box systems::stun_effect_duration::StunEffectDurationSystem::new(
        ecm.clone(),
        game_state.turn.clone()));;
    game_state.world.add_system(box systems::panic_effect_duration::PanicEffectDurationSystem::new(
        ecm.clone(),
        game_state.turn.clone()));
    game_state.world.add_system(box systems::addiction::AddictionSystem::new(
        ecm.clone(),
        game_state.turn.clone()));
    game_state.world.add_system(box systems::command_logger::CommandLoggerSystem::new(
        ecm.clone(),
        game_state.commands.clone(),
        game_state.command_logger.clone()));
    game_state.world.add_system(box systems::input::InputSystem::new(
        ecm.clone(),
        game_state.commands.clone()));
    // TODO: systems::leave_area::system,
    game_state.world.add_system(box systems::ai::AISystem::new(
        ecm.clone(),
        player_rc.clone(),
        game_state.side.clone(),
        world_size_rc.clone(),
        game_state.rng.clone()));
    game_state.world.add_system(box systems::dose::DoseSystem::new(
        ecm.clone(),
        world_size_rc.clone()));
    game_state.world.add_system(box systems::panic::PanicSystem::new(
        ecm.clone(),
        world_size_rc.clone(),
        game_state.rng.clone()));
    game_state.world.add_system(box systems::stun::StunSystem::new(
        ecm.clone()));
    game_state.world.add_system(box systems::movement::MovementSystem::new(
        ecm.clone(),
        world_size_rc.clone()));
    game_state.world.add_system(box systems::eating::EatingSystem::new(
        ecm.clone()));
    game_state.world.add_system(box systems::interaction::InteractionSystem::new(
        ecm.clone()));
    game_state.world.add_system(box systems::bump::BumpSystem::new(
        ecm.clone()));
    game_state.world.add_system(box systems::combat::CombatSystem::new(
        ecm.clone(),
        player_rc.clone(),
        game_state.turn.clone()));
    game_state.world.add_system(box systems::will::WillSystem::new(
        ecm.clone()));
    game_state.world.add_system(box systems::exploration::ExplorationSystem::new(
        ecm.clone(),
        player_rc.clone()));
    game_state.world.add_system(box systems::fade_out::FadeOutSystem::new(
        ecm.clone()));
    // todo: systems::color_fade::system(id, &mut state.ecm, &mut state.resources, dt_s)
    game_state.world.add_system(box systems::tile::TileSystem::new(
        ecm.clone(),
        engine.display(),
        player_rc.clone(),
        game_state.cheating.clone()));
    game_state.world.add_system(box systems::gui::GUISystem::new(
        ecm.clone(),
        engine.display(),
        player_rc.clone(),
        game_state.turn.clone()));
    game_state.world.add_system(box systems::turn::TurnSystem::new(
        ecm.clone(),
        game_state.side.clone(),
        game_state.turn.clone()));
    game_state.world.add_system(box systems::addiction_graphics::AddictionGraphicsSystem::new(
        ecm.clone(),
        engine.display(),
        player_rc.clone()));

    engine.main_loop(game_state, update);
}
