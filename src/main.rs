#![feature(macro_rules, struct_variant, globs, phase, link_args)]

extern crate collections;
extern crate libc;
extern crate time;


#[phase(plugin, link)] extern crate emhyr;
extern crate tcod;

use std::collections::{Deque, RingBuf, HashMap};
use std::time::Duration;
use std::io;
use std::io::{File, IoResult};
use std::io::util::NullWriter;
use std::os;
use std::rc::Rc;
use std::vec::MoveItems;
use std::cell::RefCell;

use std::rand::{IsaacRng, SeedableRng};
use std::rand;
use tcod::{KeyState, Printable, Special};

use components::{Computer, Position, Side};
use emhyr::{Entity, World};
use engine::{Engine, key};
use systems::input::commands;
use systems::input::commands::Command;

mod components;
mod engine;
mod entity_util;
mod point;
mod systems;
mod world_gen;
mod world;

// Set the binary's RPATH to the `deps` directory:
#[link_args ="-Wl,-rpath=$ORIGIN/deps"] extern {}

pub struct GameState<'a> {
    world: World<'a>,
    world_size: (int, int),
    rng: Rc<RefCell<IsaacRng>>,
    commands: Rc<RefCell<RingBuf<Command>>>,
    command_logger: Rc<RefCell<CommandLogger>>,
    position_cache: Rc<RefCell<PositionCache>>,
    side: Rc<RefCell<Side>>,
    turn: Rc<RefCell<int>>,
    player: Entity,
    cheating: Rc<RefCell<bool>>,
    replay: bool,
    paused: bool,
}

impl<'a> GameState<'a> {
    fn new(width: int, height: int,
           commands: RingBuf<Command>,
           log_writer: Box<Writer+'static>,
           seed: u32,
           cheating: bool,
           replay: bool) -> GameState<'a> {
        let seed_arr: &[_] = &[seed];
        let mut world = World::new();
        let player = world.new_entity();
        GameState {
            world: world,
            world_size: (width, height),
            rng: rc_mut(SeedableRng::from_seed(seed_arr)),
            commands: rc_mut(commands),
            command_logger: rc_mut(CommandLogger{writer: log_writer}),
            position_cache: rc_mut(PositionCache::new()),
            side: rc_mut(Computer),
            turn: rc_mut(0),
            player: player,
            cheating: rc_mut(cheating),
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
                keys.push(pressed_key);
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
                    Special(key::Up) => commands.push(commands::N),
                    Special(key::Down) => commands.push(commands::S),
                    Special(key::Left) => match (ctrl(key), key.shift) {
                        (false, true) => commands.push(commands::NW),
                        (true, false) => commands.push(commands::SW),
                        _ => commands.push(commands::W),
                    },
                    Special(key::Right) => match (ctrl(key), key.shift) {
                        (false, true) => commands.push(commands::NE),
                        (true, false) => commands.push(commands::SE),
                        _ => commands.push(commands::E),
                    },
                    Printable('e') => {
                        commands.push(commands::Eat);
                    }
                    _ => (),
                }
            },
            None => break,
        }
    }
}

fn update<'a>(mut state: GameState<'a>, dt_s: f32, engine: &engine::Engine) -> Option<GameState<'a>> {
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
    // TODO this crashes the compiler: WTF?
    state.world.update(Duration::milliseconds((dt_s * 1000.0) as i64));
    Some(state)
}


// // TODO: no longer needed?
fn write_line(writer: &mut Writer, line: &str) -> IoResult<()> {
    let line_with_nl = String::from_str(line).append("\n");
    try!(writer.write(line_with_nl.as_bytes()));
    try!(writer.flush());
    return Ok(());
}

fn rc_mut<T>(val: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(val))
}


struct CommandLogger {
    writer: Box<Writer+'static>,
}

impl CommandLogger {
    fn log(&mut self, command: Command) {
        write_line(self.writer, command.to_string().as_slice()).unwrap();
    }
}

fn new_game_state<'a>(width: int, height: int) -> GameState<'a> {
    let commands = RingBuf::new();
    let seed = rand::random::<u32>();
    let cur_time = time::now();
    let timestamp = time::strftime("%FT%T.", &cur_time).append((cur_time.tm_nsec / 1000000).to_string().as_slice());
    let replay_dir = &Path::new("./replays/");
    if !replay_dir.exists() {
        io::fs::mkdir_recursive(replay_dir,
                                io::FilePermission::from_bits(0b111101101).unwrap()).unwrap();
    }
    let replay_path = &replay_dir.join(String::from_str("replay-").append(timestamp.as_slice()));
    let mut writer = match File::create(replay_path) {
        Ok(f) => box f,
        Err(msg) => fail!("Failed to create the replay file. {}", msg)
    };
    println!("Recording the gameplay to '{}'", replay_path.display());
    write_line(&mut *writer as &mut Writer, seed.to_string().as_slice()).unwrap();
    GameState::new(width, height, commands, writer, seed, false, false)
}

fn replay_game_state<'a>(width: int, height: int) -> GameState<'a> {
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
                    None => fail!("The seed must be a number.")
                },
                None => fail!("The replay file is empty."),
            }
            for line in lines {
                match from_str(line) {
                    Some(command) => commands.push(command),
                    None => fail!("Unknown command: {}", line),
                }
            }
        },
        Err(msg) => fail!("Failed to read the replay file: {}. Reason: {}",
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
                          &mut *game_state.rng.borrow_mut(),
                          world_gen::forrest);

    // Register the components World doesn't know about yet
    game_state.world.register_component::<components::Stunned>();
    game_state.world.register_component::<components::Panicking>();
    game_state.world.register_component::<components::Destination>();
    game_state.world.register_component::<components::UsingItem>();
    game_state.world.register_component::<components::Bump>();
    game_state.world.register_component::<components::AttackTarget>();
    game_state.world.register_component::<components::FadingOut>();

    let player_rc = rc_mut(player);
    let world_size_rc = rc_mut(game_state.world_size);

    game_state.world.add_system(box systems::turn_tick_counter::TurnTickCounterSystem::new(
        game_state.side.clone()));
    game_state.world.add_system(box systems::stun_effect_duration::StunEffectDurationSystem::new(
        game_state.turn.clone()));;
    game_state.world.add_system(box systems::panic_effect_duration::PanicEffectDurationSystem::new(
        game_state.turn.clone()));
    game_state.world.add_system(box systems::addiction::AddictionSystem::new(
        game_state.turn.clone()));
    game_state.world.add_system(box systems::command_logger::CommandLoggerSystem::new(
        game_state.commands.clone(),
        game_state.command_logger.clone()));
    game_state.world.add_system(box systems::input::InputSystem::new(
        player_rc.clone(),
        game_state.commands.clone(),
        game_state.side.clone()));
    // // TODO: systems::leave_area::system,
    game_state.world.add_system(box systems::ai::AISystem::new(
        player_rc.clone(),
        game_state.position_cache.clone(),
        game_state.side.clone(),
        world_size_rc.clone(),
        game_state.rng.clone()));
    game_state.world.add_system(box systems::dose::DoseSystem::new(
        game_state.position_cache.clone(),
        world_size_rc.clone()));
    game_state.world.add_system(box systems::panic::PanicSystem::new(
        game_state.position_cache.clone(),
        world_size_rc.clone(),
        game_state.rng.clone()));
    game_state.world.add_system(box systems::stun::StunSystem::new(
        player_rc.clone()));
    game_state.world.add_system(box systems::movement::MovementSystem::new(
        game_state.position_cache.clone(),
        world_size_rc.clone()));
    game_state.world.add_system(box systems::eating::EatingSystem::new(
        game_state.position_cache.clone()));
    game_state.world.add_system(box systems::interaction::InteractionSystem::new(
        player_rc.clone(),
        game_state.position_cache.clone()));
    game_state.world.add_system(box systems::bump::BumpSystem::new(
        player_rc.clone()));
    game_state.world.add_system(box systems::combat::CombatSystem::new(
        player_rc.clone(),
        game_state.turn.clone()));
    game_state.world.add_system(box systems::will::WillSystem::new(
        player_rc.clone()));
    game_state.world.add_system(box systems::exploration::ExplorationSystem::new(
        player_rc.clone(),
        game_state.position_cache.clone()));
    game_state.world.add_system(box systems::fade_out::FadeOutSystem::new(
        player_rc.clone()));
    game_state.world.add_system(box systems::color_animation::ColorAnimationSystem::new(
        player_rc.clone()));
    game_state.world.add_system(box systems::tile::TileSystem::new(
        engine.display(),
        player_rc.clone(),
        game_state.cheating.clone()));
    game_state.world.add_system(box systems::gui::GUISystem::new(
        engine.display(),
        player_rc.clone(),
        game_state.turn.clone()));
    game_state.world.add_system(box systems::turn::TurnSystem::new(
        game_state.side.clone(),
        game_state.turn.clone()));
    game_state.world.add_system(box systems::addiction_graphics::AddictionGraphicsSystem::new(
        engine.display(),
        player_rc.clone()));
}

pub struct PositionCache {
    map: HashMap<(int, int), Vec<Entity>>,
}

impl PositionCache {
    pub fn new() -> PositionCache {
        PositionCache{map: HashMap::new()}
    }

    pub fn set(&mut self, pos: (int, int), e: Entity) {
        let entities = self.map.find_or_insert_with(pos, |_| vec![]);
        entities.push(e);
    }

    pub fn unset(&mut self, pos: (int, int), e: Entity) {
        let ref mut entities = self.map.get_mut(&pos);
        match entities.iter().position(|&i| i == e) {
            Some(entity_index) => {
                entities.remove(entity_index);
            }
            None => fail!("{} at position {} missing from the cache.", e, pos),
        }
    }

    pub fn entities_on_pos<P: point::Point>(&self, pos: P) -> MoveItems<Entity> {
        match self.map.find(&pos.coordinates()) {
            Some(entities) => entities.clone().move_iter(),
            None => vec![].move_iter(),
        }
    }
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
        _ => fail!("You must pass either pass zero or one arguments."),
    };

    let pos_cache = game_state.position_cache.clone();
    game_state.world.on_component_change(|e, component, change| {
        use std::any::AnyRefExt;
        use emhyr::{ComponentSet, ComponentUnset};
        match component.downcast_ref::<Position>() {
            Some(pos) => {
                let coords = (pos.x, pos.y);
                let mut cache = pos_cache.borrow_mut();
                match change {
                    ComponentSet => {
                        // We don't need to remove any previous Position the
                        // entity may have had from the cache because Emhyr will
                        // have fired a ComponentRemoved event.
                        cache.set(coords, e);
                    }
                    ComponentUnset => {
                        // We know it's a Position and we know it must have been
                        // set before. Therefore we know it's in the cache:
                        cache.unset(coords, e);
                    }
                }
            }
            None => (),
        }
    });
    let mut engine = Engine::new(width, height, title, font_path.clone());
    initialise_world(&mut game_state, &engine);
    engine.main_loop(game_state, update);
}
