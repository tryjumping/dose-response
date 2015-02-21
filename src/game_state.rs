use std::collections::VecDeque;
use std::env;
use std::time::Duration;
use std::old_io;
use std::old_io::File;
use std::old_io::fs::PathExtensions;
use std::old_io::util::NullWriter;
use std::str;
use std::string::ToString;

use time;
use rand::{self, IsaacRng, SeedableRng};

use generators;
use level::Level;
use monster::Monster;
use player::Player;
use world;


#[derive(Copy, PartialEq, Clone, Debug)]
pub enum Side {
    Player,
    Computer,
}


#[derive(Copy, Debug)]
pub enum Command {
    N, E, S, W, NE, NW, SE, SW,
    Eat,
}

impl ToString for Command {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}


fn command_from_str(name: &str) -> Command {
    use self::Command::*;
    match name {
        "N" => N,
        "E" => E,
        "S" => S,
        "W" => W,
        "NE" => NE,
        "NW" => NW,
        "SE" => SE,
        "SW" => SW,
        "Eat" => Eat,
        _ => panic!("Unknown command: '{}'", name)
    }
}



pub struct GameState {
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub explosion_animation: super::ExplosionAnimation,
    pub level: Level,
    pub display_size: (i32, i32),
    pub rng: IsaacRng,
    pub commands: VecDeque<Command>,
    pub command_logger: CommandLogger,
    pub side: Side,
    pub turn: i32,
    pub cheating: bool,
    pub replay: bool,
    pub clock: Duration,
    pub paused: bool,
    pub screen_fading: Option<super::ScreenFadeAnimation>,
    pub see_entire_screen: bool,
}

impl GameState {
    fn new(width: i32, height: i32,
           commands: VecDeque<Command>,
           log_writer: Box<Writer+'static>,
           seed: u32,
           cheating: bool,
           replay: bool) -> GameState {
        let seed_arr: &[_] = &[seed];
        GameState {
            player: Player::new((40, 25)),
            monsters: vec![],
            explosion_animation: None,
            level: Level::new(width, height - 2),
            display_size: (width, height),
            rng: SeedableRng::from_seed(seed_arr),
            commands: commands,
            command_logger: CommandLogger{writer: log_writer},
            side: Side::Player,
            turn: 0,
            cheating: cheating,
            replay: replay,
            clock: Duration::zero(),
            paused: false,
            screen_fading: None,
            see_entire_screen: false,
        }
    }

    pub fn new_game(width: i32, height: i32) -> GameState {
        let commands = VecDeque::new();
        let seed = rand::random::<u32>();
        let cur_time = time::now();
        let timestamp = format!("{}.{:03}",
                                time::strftime("%FT%T", &cur_time).unwrap(),
                                (cur_time.tm_nsec / 1000000));
        let replay_dir = &Path::new("./replays/");
        if !replay_dir.exists() {
            old_io::fs::mkdir_recursive(replay_dir,
                                    old_io::FilePermission::from_bits(0b111101101).unwrap()).unwrap();
        }
        let replay_path = &replay_dir.join(format!("replay-{}", timestamp));
        let mut writer = match File::create(replay_path) {
            Ok(f) => Box::new(f),
            Err(msg) => panic!("Failed to create the replay file. {}", msg)
        };
        println!("Recording the gameplay to '{}'", replay_path.display());
        writer.write_line(seed.to_string().as_slice()).unwrap();
        let mut state = GameState::new(width, height, commands, writer, seed, false, false);
        initialise_world(&mut state);
        state
    }

    pub fn replay_game(width: i32, height: i32) -> GameState {
        let mut commands = VecDeque::new();
        let replay_path = &Path::new(env::args().nth(1).unwrap());
        let mut seed: u32;
        match File::open(replay_path) {
            Ok(mut file) => {
                let bin_data = file.read_to_end().unwrap();
                let contents = str::from_utf8(&bin_data);
                let mut lines = contents.unwrap().lines();
                match lines.next() {
                    Some(seed_str) => match seed_str.parse() {
                        Ok(parsed_seed) => seed = parsed_seed,
                        Err(_) => panic!("The seed must be a number.")
                    },
                    None => panic!("The replay file is empty."),
                }
                for line in lines {
                    commands.push_back(command_from_str(line));
                }
            },
            Err(msg) => panic!("Failed to read the replay file: {}. Reason: {}",
                               replay_path.display(), msg)
        }
        println!("Replaying game log: '{}'", replay_path.display());
        let mut state = GameState::new(width, height, commands, Box::new(NullWriter), seed, true, true);
        initialise_world(&mut state);
        state
    }
}

fn initialise_world(game_state: &mut GameState) {
    let (width, height) = game_state.level.size();
    let generated_world = generators::forrest::generate(&mut game_state.rng,
                                                        width, height,
                                                        game_state.player.pos);
    world::populate_world(&mut game_state.level,
                          &mut game_state.monsters,
                          generated_world);
    // Sort monsters by their APs, set their IDs to equal their indexes in state.monsters:
    game_state.monsters.sort_by(|a, b| b.max_ap.cmp(&a.max_ap));
    for (index, m) in game_state.monsters.iter_mut().enumerate() {
        unsafe { m.set_id(index) }
        game_state.level.set_monster(m.position, m.id(), m);
    }
}


pub struct CommandLogger {
    writer: Box<Writer+'static>,
}

impl CommandLogger {
    pub fn log(&mut self, command: Command) {
        self.writer.write_line(command.to_string().as_slice()).unwrap();
    }
}
