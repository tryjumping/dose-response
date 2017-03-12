use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, BufReader, BufRead, Write};
use std::path::Path;

use stats::Stats;
use time;
use time::Duration;
use rand::{self, IsaacRng, SeedableRng};

use animation::{AreaOfEffect, ScreenFade};
use keys::Keys;
use player::Player;
use point::Point;
use timer::Timer;
use world::World;


// TODO: Rename this to `GameState` and the existing `GameState` to
// `Game`? It's no longer just who's side it is but also: did the
// player won? Lost?
#[derive(Copy, PartialEq, Clone, Debug)]
pub enum Side {
    Player,
    Victory,
}


// TODO: rename this to Input or something like that. This represents the raw
// commands from the player or AI abstracted from keyboard, joystick or
// whatever. But they shouldn't carry any context or data.
#[derive(Copy, Clone, Debug)]
pub enum Command {
    N, E, S, W,
    NE, NW, SE, SW,
    UseFood,
    UseDose,
    UseCardinalDose,
    UseDiagonalDose,
    UseStrongDose,
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
            UseFood => "UseFood",
            UseDose => "UseDose",
            UseCardinalDose => "UseCardinalDose",
            UseDiagonalDose => "UseDiagonalDose",
            UseStrongDose => "UseStrongDose",
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
        "UseFood" => UseFood,
        "UseDose" => UseDose,
        "UseStrongDose" => UseStrongDose,
        _ => panic!("Unknown command: '{}'", name)
    }
}


// TODO: remove when this exists in the stable standard library (it prolly does now)
fn path_exists(path: &Path) -> bool {
    ::std::fs::metadata(path).is_ok()
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub turn: i32,
    pub chunk_count: usize,
    pub player_pos: Point,
}

pub struct GameState {
    pub player: Player,
    pub explosion_animation: Option<Box<AreaOfEffect>>,

    /// The actual size of the game world in tiles. Could be infinite
    /// but we're limiting it for performance reasons for now.
    pub world_size: Point,
    pub chunk_size: i32,
    pub world: World,

    /// The size of the game map inside the game window. We're keeping
    /// this square so this value repesents both width and heigh.
    /// It's a window into the game world that is actually rendered.
    pub map_size: Point,

    /// The width of the in-game status panel.
    pub panel_width: i32,

    /// The size of the game window in tiles. The area stuff is
    /// rendered to. NOTE: currently, the width is equal to map_size +
    /// panel_width, height is map_size.
    pub display_size: Point,
    pub screen_position_in_world: Point,
    pub seed: u32,
    pub rng: IsaacRng,
    pub keys: Keys,
    pub commands: VecDeque<Command>,
    pub verifications: VecDeque<Verification>,
    pub command_logger: Box<Write>,
    pub side: Side,
    pub turn: i32,
    pub cheating: bool,
    pub replay: bool,
    pub replay_full_speed: bool,
    pub replay_exit_after: bool,
    pub clock: Duration,
    pub replay_step: Duration,
    pub stats: Stats,
    pub pos_timer: Timer,
    pub paused: bool,
    pub old_screen_pos: Point,
    pub new_screen_pos: Point,
    pub screen_fading: Option<ScreenFade>,

    /// Whether the game is over (one way or another) and we should
    /// show the endgame screen -- uncovered map, the score, etc.
    pub endgame_screen: bool,
}

impl GameState {
    fn new<W: Write+'static>(world_size: Point,
                             map_size: i32,
                             panel_width: i32,
                             display_size: Point,
                             commands: VecDeque<Command>,
                             verifications: VecDeque<Verification>,
                             log_writer: W,
                             seed: u32,
                             cheating: bool,
                             replay: bool,
                             replay_full_speed: bool,
                             replay_exit_after: bool)
                             -> GameState {
        let seed_arr: &[_] = &[seed];
        let world_centre = (0, 0).into();
        assert_eq!(world_size.x, world_size.y);
        assert_eq!(display_size, (map_size + panel_width, map_size));
        let player_position = world_centre;
        GameState {
            player: Player::new(player_position),
            explosion_animation: None,
            chunk_size: 32,
            world_size: world_size,
            world: World::new(seed, world_size.x, 32, player_position),
            map_size: (map_size, map_size).into(),
            panel_width: panel_width,
            display_size: display_size,
            screen_position_in_world: world_centre,
            seed: seed,
            rng: SeedableRng::from_seed(seed_arr),
            keys: Keys::new(),
            commands: commands,
            verifications: verifications,
            command_logger: Box::new(log_writer),
            side: Side::Player,
            turn: 0,
            cheating: cheating,
            replay: replay,
            replay_full_speed: replay_full_speed,
            replay_exit_after: replay_exit_after,
            clock: Duration::zero(),
            replay_step: Duration::zero(),
            stats: Stats::new(6000),  // about a minute and a half at 60 FPS
            pos_timer: Timer::new(Duration::milliseconds(0)),
            old_screen_pos: (0, 0).into(),
            new_screen_pos: (0, 0).into(),
            paused: false,
            screen_fading: None,
            endgame_screen: false,
        }
    }

    pub fn new_game(world_size: Point, map_size: i32, panel_width: i32, display_size: Point) -> GameState {
        let commands = VecDeque::new();
        let verifications = VecDeque::new();
        let seed = rand::random::<u32>();
        let cur_time = time::now();
        // Timestamp in format: 2016-11-20T20-04-39.123
        // We can't use the colons in the timestamp -- Windows don't allow them in a path.
        let timestamp = format!("{}.{:03}",
                                time::strftime("%FT%H-%M-%S", &cur_time).unwrap(),
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
        GameState::new(world_size, map_size, panel_width, display_size, commands,
                       verifications, writer,
                       seed, false, false, false, false)
    }

    pub fn replay_game(world_size: Point, map_size: i32, panel_width: i32, display_size: Point, replay_path: &Path, replay_full_speed: bool, replay_exit_after: bool) -> GameState {
        use serde_json;
        let mut commands = VecDeque::new();
        let mut verifications = VecDeque::new();
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

                loop {
                    match lines.next() {
                        Some(Ok(line)) => commands.push_back(command_from_str(&line)),
                        Some(Err(err)) => panic!("Error reading a line from the replay file: {:?}.", err),
                        None => break,
                    }

                    match lines.next() {
                        Some(Ok(line)) => {
                            let verification = serde_json::from_str(&line).expect(
                                &format!("Could not deserialise the verification: '{}'", line));
                            verifications.push_back(verification);
                        },
                        Some(Err(err)) => panic!("Error reading a verification from the replay log: {:?}.", err),
                        None => break,
                    }
                }

            },
            Err(msg) => panic!("Failed to read the replay file: {}. Reason: {}",
                               replay_path.display(), msg)
        }
        // println!("Replaying game log: '{}'", replay_path.display());
        GameState::new(world_size, map_size, panel_width, display_size, commands,
                       verifications,
                       Box::new(io::sink()), seed, true, true, replay_full_speed, replay_exit_after)
    }
}


pub fn log_seed<W: Write>(writer: &mut W, seed: u32) {
    writeln!(writer, "{}", seed).unwrap();
}

pub fn log_command<W: Write>(writer: &mut W, command: Command) {
    writeln!(writer, "{}", command.to_str()).unwrap();
}

pub fn log_verification<W: Write>(writer: &mut W, verification: Verification) {
    use serde_json;
    let json = serde_json::to_string(&verification).expect(
        &format!("Could not serialise {:?} to json.", verification));
    writeln!(writer, "{}", json).expect(
        &format!("Could not write the verification: '{}' to the replay log.", json));
}
