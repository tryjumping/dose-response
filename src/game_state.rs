use std::collections::VecDeque;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, BufRead, Write};
use std::path::Path;

use time;
use time::Duration;
use rand::{self, IsaacRng, SeedableRng};

use generators;
use level::Level;
use monster::Monster;
use player::Player;
use point::Point;
use world;


#[derive(Copy, PartialEq, Clone, Debug)]
pub enum Side {
    Player,
    Computer,
}


// TODO: rename this to Input or something like that. This represents the raw
// commands from the player or AI abstracted from keyboard, joystick or
// whatever. But they shouldn't carry any context or data.
#[derive(Copy, Clone, Debug)]
pub enum Command {
    N, E, S, W, NE, NW, SE, SW,
    Eat,
}

impl Command {
    fn to_str(&self) -> &'static str {
    use self::Command::*;
        match *self {
            N => "N",
            E => "E",
            S => "S",
            W => "W",
            NE => "NE",
            NW => "NW",
            SE => "SE",
            SW => "SW",
            Eat => "Eat",
        }
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


fn path_exists(path: &Path) -> bool {
    ::std::fs::metadata(path).is_ok()
}


pub struct GameState {
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub explosion_animation: super::ExplosionAnimation,
    pub level: Level,

    /// The actual size of the game world in tiles. Could be infinite
    /// but we're limiting it for performance reasons for now.
    pub world_size: Point,

    /// The size of the game map inside the game window. We're keeping
    /// this square so this value repesents both width and heigh.
    /// It's a window into the game world that is actually rendered.
    pub map_size: i32,

    /// The width of the in-game status panel.
    pub panel_width: i32,

    /// The size of the game window in tiles. The area stuff is
    /// rendered to. NOTE: currently, the width is equal to map_size +
    /// panel_width, height is map_size.
    pub display_size: Point,
    pub screen_position_in_world: Point,
    pub rng: IsaacRng,
    pub commands: VecDeque<Command>,
    pub command_logger: Box<Write>,
    pub side: Side,
    pub turn: i32,
    pub cheating: bool,
    pub replay: bool,
    pub clock: Duration,
    pub pos_timer: ::Timer,
    pub paused: bool,
    pub old_screen_pos: Point,
    pub new_screen_pos: Point,
    pub screen_fading: Option<super::ScreenFadeAnimation>,
    pub see_entire_screen: bool,
}

impl GameState {
    fn new<W: Write+'static>(world_size: Point,
                             map_size: i32,
                             panel_width: i32,
                             display_size: Point,
                             commands: VecDeque<Command>,
                             log_writer: W,
                             seed: u32,
                             cheating: bool,
                             replay: bool)
                             -> GameState {
        let seed_arr: &[_] = &[seed];
        let world_centre = world_size / 2;
        assert_eq!(display_size, (map_size + panel_width, map_size));
        GameState {
            player: Player::new(world_centre),
            monsters: vec![],
            explosion_animation: None,
            level: Level::new(world_size.x, world_size.y),
            world_size: world_size,
            map_size: map_size,
            panel_width: panel_width,
            display_size: display_size,
            screen_position_in_world: world_centre,
            rng: SeedableRng::from_seed(seed_arr),
            commands: commands,
            command_logger: Box::new(log_writer),
            side: Side::Player,
            turn: 0,
            cheating: cheating,
            replay: replay,
            clock: Duration::zero(),
            pos_timer: ::Timer::new(Duration::milliseconds(0)),
            old_screen_pos: (0, 0).into(),
            new_screen_pos: (0, 0).into(),
            paused: false,
            screen_fading: None,
            see_entire_screen: false,
        }
    }

    pub fn new_game(world_size: Point, map_size: i32, panel_width: i32, display_size: Point) -> GameState {
        let commands = VecDeque::new();
        let seed = rand::random::<u32>();
        let cur_time = time::now();
        let timestamp = format!("{}.{:03}",
                                time::strftime("%FT%T", &cur_time).unwrap(),
                                (cur_time.tm_nsec / 1000000));
        let replay_dir = &Path::new("replays");
        assert!(replay_dir.is_relative());
        if !path_exists(replay_dir) {
            fs::create_dir_all(replay_dir).unwrap();
        }
        let replay_path = &replay_dir.join(format!("replay-{}", timestamp));
        let mut writer = match File::create(replay_path) {
            Ok(f) => f,
            Err(msg) => panic!("Failed to create the replay file at '{:?}'.\nReason: '{}'.",
                               replay_path.display(), msg),
        };
        // println!("Recording the gameplay to '{}'", replay_path.display());
        log_seed(&mut writer, seed);
        let mut state = GameState::new(world_size, map_size, panel_width, display_size, commands, writer, seed, false, false);
        initialise_world(&mut state);
        state
    }

    pub fn replay_game(world_size: Point, map_size: i32, panel_width: i32, display_size: Point) -> GameState {
        let mut commands = VecDeque::new();
        let path_str = env::args().nth(1).unwrap();
        let replay_path = &Path::new(&path_str);
        let seed: u32;
        match File::open(replay_path) {
            Ok(file) => {
                let mut lines = BufReader::new(file).lines();
                match lines.next() {
                    Some(seed_str) => match seed_str.unwrap().parse() {
                        Ok(parsed_seed) => seed = parsed_seed,
                        Err(_) => panic!("The seed must be a number.")
                    },
                    None => panic!("The replay file is empty."),
                }
                for line in lines {
                    match line {
                        Ok(line) => commands.push_back(command_from_str(&line)),
                        Err(err) => panic!("Error reading a line from the replay file: {:?}.", err),
                    }
                }
            },
            Err(msg) => panic!("Failed to read the replay file: {}. Reason: {}",
                               replay_path.display(), msg)
        }
        // println!("Replaying game log: '{}'", replay_path.display());
        let mut state = GameState::new(world_size, map_size, panel_width, display_size, commands, Box::new(io::sink()), seed, true, true);
        initialise_world(&mut state);
        state
    }
}

fn initialise_world(game_state: &mut GameState) {
    let dimensions = game_state.level.size();
    let generated_world = generators::forrest::generate(&mut game_state.rng,
                                                        dimensions,
                                                        game_state.player.pos);
    world::populate_world(&mut game_state.level,
                          &mut game_state.monsters,
                          generated_world);
    // Sort monsters by their APs, set their IDs to equal their indexes in state.monsters:
    game_state.monsters.sort_by(|a, b| b.max_ap.cmp(&a.max_ap));
    for (index, m) in game_state.monsters.iter_mut().enumerate() {
        unsafe {
            m.set_id(index);
        }
        game_state.level.set_monster(m.position, m.id(), m);
    }
}


pub fn log_seed<W: Write>(writer: &mut W, seed: u32) {
    writeln!(writer, "{}", seed).unwrap();
}

pub fn log_command<W: Write>(writer: &mut W, command: Command) {
    writeln!(writer, "{}", command.to_str()).unwrap();
}
