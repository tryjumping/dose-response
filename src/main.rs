#![crate_id = "dose-response#0.1.0"]

// #[feature(macro_rules, struct_variant, globs)];

// extern mod extra;

// use std::io;
// use std::io::File;
// use std::io::util::NullWriter;

// use std::rand::{Rng, IsaacRng, SeedableRng};
// use std::os;

// use components::{ComponentManager, ID, Computer, Side, Position};
// use engine::{Display, MainLoopState, Key, key};
// use extra::ringbuf::RingBuf;
// use extra::container::Deque;
// use extra::time;
// use systems::input::commands;
// use systems::input::commands::Command;

// pub mod components;
// mod engine;
// pub mod systems;
// pub mod tcod;
// pub mod world_gen;
// pub mod world;
// pub mod util;


// pub struct GameState {
//     entities: ComponentManager,
//     resources: Resources,
// }

// pub struct Resources {
//     side: Side,
//     world_size: (int, int),
//     turn: int,
//     rng: IsaacRng,
//     commands: RingBuf<Command>,
//     command_logger: CommandLogger,
//     player_id: ID,
//     cheating: bool,
//     replay: bool,
//     paused: bool,
// }

// fn key_pressed(keys: &RingBuf<Key>, key_code: engine::key::KeyCode) -> bool {
//     for &pressed_key in keys.iter() {
//         if pressed_key.code == key_code {
//             return true;
//         }
//     }
//     false
// }

// /// Consumes the first occurence of the given key in the buffer.
// ///
// /// Returns `true` if the key has been in the buffer.
// fn read_key(keys: &mut RingBuf<Key>, key: engine::key::KeyCode) -> bool {
//     let mut len = keys.len();
//     let mut processed = 0;
//     let mut found = false;
//     while processed < len {
//         match keys.pop_front() {
//             Some(pressed_key) if !found && pressed_key.code == key => {
//                 len -= 1;
//                 found = true;
//             }
//             Some(pressed_key) => {
//                 keys.push_back(pressed_key);
//             }
//             None => return false
//         }
//         processed += 1;
//     }
//     return found;
// }

// fn process_input(keys: &mut RingBuf<Key>, commands: &mut RingBuf<Command>) {
//     // TODO: switch to DList and consume it with `mut_iter`.
//     loop {
//         match keys.pop_front() {
//             Some(key) => {
//                 match key.code {
//                     key::Up => commands.push_back(commands::N),
//                     key::Down => commands.push_back(commands::S),
//                     key::Left => match (key.ctrl(), key.shift()) {
//                         (false, true) => commands.push_back(commands::NW),
//                         (true, false) => commands.push_back(commands::SW),
//                         _ => commands.push_back(commands::W),
//                     },
//                     key::Right => match (key.ctrl(), key.shift()) {
//                         (false, true) => commands.push_back(commands::NE),
//                         (true, false) => commands.push_back(commands::SE),
//                         _ => commands.push_back(commands::E),
//                     },
//                     key::Char => {
//                         match key.char {
//                             'e' => commands.push_back(commands::Eat),
//                             _ => (),
//                         }
//                     }
//                     _ => (),
//                 }
//             },
//             None => break,
//         }
//     }
// }

// pub fn null_input_system(_e: ID, _ecm: &mut ComponentManager, _res: &mut Resources) {}

// fn update(state: &mut GameState,
//           display: &mut Display,
//           keys: &mut RingBuf<Key>,
//           dt_s: f32) -> MainLoopState<GameState> {
//     if key_pressed(keys, key::Escape) { return engine::Exit }
//     if key_pressed(keys, key::F5) {
//         println!("Restarting game");
//         keys.clear();
//         let (width, height) = state.resources.world_size;
//         let mut state = new_game_state(width, height);
//         let player = world::player_entity(&mut state.entities);
//         let player_pos = Position{x: width / 2, y: height / 2};
//         state.entities.set_position(player, player_pos);
//         assert!(state.entities.has_entity(state.resources.player_id));
//         world::populate_world(&mut state.entities,
//                               state.resources.world_size,
//                               player_pos,
//                               &mut state.resources.rng,
//                               world_gen::forrest);
//         return engine::NewState(state);
//     }
//     if key_pressed(keys, key::F6) {
//         state.resources.cheating = !state.resources.cheating;
//         println!("Cheating set to: {}", state.resources.cheating);
//     }


//     state.resources.paused = if state.resources.replay && read_key(keys, key::Spacebar) {
//         if !state.resources.paused {println!("Pausing the replay")};
//         !state.resources.paused
//     } else {
//         state.resources.paused
//     };
//     let mut input_system = if state.resources.paused {
//         null_input_system
//     } else {
//         systems::input::system
//     };
//     // Move one step forward in the paused replay
//     if state.resources.paused && read_key(keys, key::Right) {
//         input_system = systems::input::system;
//     }
//     let systems = [
//         systems::turn_tick_counter::system,
//         systems::effect_duration::system,
//         systems::addiction::system,
//         input_system,
//         systems::leave_area::system,
//         systems::player_dead::system,
//         systems::ai::system,
//         systems::dose::system,
//         systems::panic::system,
//         systems::stun::system,
//         systems::movement::system,
//         systems::eating::system,
//         systems::interaction::system,
//         systems::bump::system,
//         systems::combat::system,
//         systems::will::system,
//         systems::exploration::system,
//         systems::fade_out::system,
//     ];

//     process_input(keys, &mut state.resources.commands);
//     for id in state.entities.iter() {
//         for &sys in systems.iter() {
//             if state.entities.has_entity(id) {
//                 sys(id, &mut state.entities, &mut state.resources);
//             }
//         }
//         if state.entities.has_entity(id) {
//             systems::color_fade::system(id, &mut state.entities, &mut state.resources, dt_s);
//             systems::tile::system(id,
//                                   &mut state.entities,
//                                   &mut state.resources,
//                                   display);
//         }
//     }
//     systems::gui::system(&state.entities,
//                          &mut state.resources,
//                          display);
//     systems::turn::system(&mut state.entities,
//                           &mut state.resources);
//     systems::addiction_graphics::system(&mut state.entities,
//                                         &mut state.resources,
//                                         display);
//     engine::Running
// }


// fn write_line(writer: &mut Writer, line: &str) {
//     let line_with_nl = line + "\n";
//     writer.write(line_with_nl.as_bytes());
//     writer.flush();
// }


// struct CommandLogger {
//     priv writer: ~Writer,
// }

// impl CommandLogger {
//     fn log(&mut self, command: Command) {
//         write_line(self.writer, command.to_str());
//     }
// }

// fn new_game_state(width: int, height: int) -> GameState {
//     let mut rng = IsaacRng::new();
//     let commands = RingBuf::new();
//     let seed = rng.gen_range(0u32, 10000);
//     rng.reseed([seed]);
//     let cur_time = time::now();
//     let timestamp = time::strftime("%FT%T.", &cur_time) +
//         (cur_time.tm_nsec / 1000000).to_str();
//     let replay_dir = &Path::new("./replays/");
//     if !replay_dir.exists() {
//         io::fs::mkdir_recursive(replay_dir, 0b111101101);
//     }
//     let replay_path = &replay_dir.join("replay-" + timestamp);
//     let mut writer = match File::create(replay_path) {
//         Some(f) => ~f as ~Writer,
//         None => fail!("Failed to create the replay file.")
//     };
//     write_line(writer, seed.to_str());
//     let logger = CommandLogger{writer: writer};
//     let ecm = ComponentManager::new();
//     GameState {
//         entities: ecm,
//         resources: Resources{
//             commands: commands,
//             command_logger: logger,
//             rng: rng,
//             side: Computer,
//             turn: 0,
//             player_id: ID(0),
//             cheating: false,
//             replay: false,
//             paused: false,
//             world_size: (width, height),
//         },
//     }
// }

// fn replay_game_state(width: int, height: int) -> GameState {
//     let mut commands = RingBuf::new();
//     let replay_path = &Path::new(os::args()[1]);
//     let mut seed: u32;
//     match File::open(replay_path) {
//         Some(mut file) => {
//             let bin_data = file.read_to_end();
//             let contents = std::str::from_utf8(bin_data);
//             let mut lines = contents.lines();
//             match lines.next() {
//                 Some(seed_str) => match from_str(seed_str) {
//                     Some(parsed_seed) => seed = parsed_seed,
//                     None => fail!("The seed must be a number.")
//                 },
//                 None => fail!("The replay file is empty."),
//             }
//             for line in lines {
//                 match from_str(line) {
//                     Some(command) => commands.push_back(command),
//                     None => fail!("Unknown command: {}", line),
//                 }
//             }
//         },
//         None => fail!("Failed to read the replay file: {}", replay_path.display())
//     }
//     println!("Replaying game log: '{}'", replay_path.display());
//     let rng: IsaacRng = SeedableRng::from_seed(&[seed]);
//     let logger = CommandLogger{writer: ~NullWriter as ~Writer};
//     let ecm = ComponentManager::new();
//     GameState {
//         entities: ecm,
//         resources: Resources {
//             commands: commands,
//             rng: rng,
//             command_logger: logger,
//             side: Computer,
//             turn: 0,
//             player_id: ID(0),
//             cheating: false,
//             replay: true,
//             paused: false,
//             world_size: (width, height),
//         },
//     }
// }


// fn main() {
//     let (width, height) = (80, 50);
//     let title = "Dose Response";
//     let font_path = Path::new("./fonts/dejavu16x16_gs_tc.png");

//     let mut game_state = match os::args().len() {
//         1 => {  // Run the game with a new seed, create the replay log
//             new_game_state(width, height)
//         },
//         2 => {  // Replay the game from the entered log
//             replay_game_state(width, height)
//         },
//         _ => fail!("You must pass either pass zero or one arguments."),
//     };

//     let player = world::player_entity(&mut game_state.entities);
//     game_state.entities.set_position(player, Position{x: width / 2, y: height / 2});
//     let player_pos = game_state.entities.get_position(player);
//     assert_eq!(player, ID(0));
//     game_state.resources.player_id = player;
//     world::populate_world(&mut game_state.entities,
//                           game_state.resources.world_size,
//                           player_pos,
//                           &mut game_state.resources.rng,
//                           world_gen::forrest);

//     engine::main_loop(width, height, title, font_path,
//                       game_state,
//                       update);
// }

extern crate collections;

extern crate emhyr;
extern crate tcod;

use emhyr::{ComponentManager, ECM};

use engine::{Display};

mod engine;


fn main() {
    let mut ecm = ECM::new();
    let player = ecm.new_entity();
    let mut console = tcod::Console::init_root(80, 25, "Whatever", false);
    console.set_custom_font(Path::new("./fonts/dejavu16x16_gs_tc.png"));
    println!("Hello, world!");
    println!("This is the player entity: {:?}", player);
}
