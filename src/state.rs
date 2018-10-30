use crate::animation::{AreaOfEffect, ScreenFade};
use crate::engine::Mouse;
use crate::keys::Keys;
use crate::monster;
use crate::player::Player;
use crate::point::Point;
use crate::util;
use rand::IsaacRng;

use crate::stats::Stats;
use crate::timer::Timer;
use crate::windows;
use crate::world::{MonsterId, World};
use std::collections::VecDeque;
use std::error::Error;
#[cfg(feature = "replay")]
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

// TODO: Rename this to `GameState` and the existing `GameState` to
// `Game`? It's no longer just who's side it is but also: did the
// player won? Lost?
#[derive(Copy, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Side {
    Player,
    Victory,
}

// TODO: rename this to Input or something like that. This represents the raw
// commands from the player or AI abstracted from keyboard, joystick or
// whatever. But they shouldn't carry any context or data.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    N,
    E,
    S,
    W,
    NE,
    NW,
    SE,
    SW,
    UseFood,
    UseDose,
    UseCardinalDose,
    UseDiagonalDose,
    UseStrongDose,
}

#[cfg(feature = "replay")]
pub fn generate_replay_path() -> Option<PathBuf> {
    use chrono::prelude::*;
    let local_time = Local::now();

    // Timestamp in format: 2016-11-20T20-04-39.123. We can't use the
    // colons in the timestamp -- Windows don't allow them in a path.
    let timestamp = local_time.format("%FT%H-%M-%S%.3f");
    let replay_dir = &Path::new("replays");
    assert!(replay_dir.is_relative());
    if !replay_dir.exists() {
        fs::create_dir_all(replay_dir).unwrap();
    }
    let replay_path = &replay_dir.join(format!("replay-{}", timestamp));
    Some(replay_path.into())
}

#[cfg(not(feature = "replay"))]
pub fn generate_replay_path() -> Option<PathBuf> {
    None
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub turn: i32,
    pub chunk_count: usize,
    pub player_pos: Point,
    pub monsters: Vec<(Point, Point, monster::Kind)>,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub player: Player,
    #[serde(skip_serializing, skip_deserializing)]
    pub explosion_animation: Option<Box<dyn AreaOfEffect>>,

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
    pub mouse: Mouse,
    pub commands: VecDeque<Command>,
    #[serde(skip_serializing, skip_deserializing)]
    pub verifications: VecDeque<Verification>,
    #[serde(
        skip_serializing,
        skip_deserializing,
        default = "empty_command_logger"
    )]
    pub command_logger: Box<dyn Write>,
    pub side: Side,
    pub turn: i32,
    pub cheating: bool,
    pub replay: bool,
    pub replay_full_speed: bool,
    pub exit_after: bool,
    pub clock: Duration,
    pub replay_step: Duration,
    #[serde(skip_serializing, skip_deserializing)]
    pub stats: Stats,
    pub pos_timer: Timer,
    pub paused: bool,
    pub old_screen_pos: Point,
    pub new_screen_pos: Point,
    pub screen_fading: Option<ScreenFade>,
    pub offset_px: Point,

    /// Whether the game is over (one way or another) and we should
    /// show the endgame screen -- uncovered map, the score, etc.
    pub game_ended: bool,
    pub victory_npc_id: Option<MonsterId>,

    pub window_stack: windows::Windows<Window>,

    pub show_keboard_movement_hints: bool,
    pub current_help_window: windows::help::Page,

    /// Whether we should push the Endscreen window and uncover the
    /// map during the transition from screen fade out to fade in
    /// phase. This is purely a visual effect and the values here are
    /// a bit of a hack. If there's more instances of us wanting to do
    /// this, we hould just have a list of screen fade transition
    /// effects here instead.
    pub show_endscreen_and_uncover_map_during_fadein: bool,
    pub uncovered_map: bool,
}

impl State {
    fn new<W: Write + 'static>(
        world_size: Point,
        map_size: i32,
        panel_width: i32,
        display_size: Point,
        commands: VecDeque<Command>,
        verifications: VecDeque<Verification>,
        log_writer: W,
        seed: u32,
        cheating: bool,
        invincible: bool,
        replay: bool,
        replay_full_speed: bool,
        exit_after: bool,
    ) -> State {
        let world_centre = (0, 0).into();
        assert_eq!(world_size.x, world_size.y);
        assert_eq!(display_size, (map_size + panel_width, map_size));
        let player_position = world_centre;
        let player = Player::new(player_position, invincible);
        let mut rng = IsaacRng::new_from_u64(u64::from(seed));
        let world = World::new(&mut rng, seed, world_size.x, 32, player.info());

        State {
            player,
            explosion_animation: None,
            chunk_size: 32,
            world_size,
            world,
            map_size: (map_size, map_size).into(),
            panel_width,
            display_size,
            screen_position_in_world: world_centre,
            seed,
            rng,
            keys: Keys::new(),
            mouse: Default::default(),
            commands,
            verifications,
            command_logger: Box::new(log_writer),
            side: Side::Player,
            turn: 0,
            cheating,
            replay,
            replay_full_speed,
            exit_after,
            clock: Duration::new(0, 0),
            replay_step: Duration::new(0, 0),
            stats: Default::default(),
            pos_timer: Timer::new(Duration::from_millis(0)),
            old_screen_pos: (0, 0).into(),
            new_screen_pos: (0, 0).into(),
            offset_px: Point::zero(),
            paused: false,
            screen_fading: None,
            game_ended: false,
            victory_npc_id: None,
            window_stack: windows::Windows::new(Window::Game),
            show_keboard_movement_hints: true,
            current_help_window: windows::help::Page::DoseResponse,
            show_endscreen_and_uncover_map_during_fadein: false,
            uncovered_map: false,
        }
    }

    pub fn new_game(
        world_size: Point,
        map_size: i32,
        panel_width: i32,
        display_size: Point,
        exit_after: bool,
        replay_path: Option<PathBuf>,
        invincible: bool,
    ) -> State {
        let commands = VecDeque::new();
        let verifications = VecDeque::new();
        let seed = util::random_seed();
        let mut writer: Box<dyn Write> = if let Some(replay_path) = replay_path {
            match File::create(&replay_path) {
                Ok(f) => {
                    info!("Recording the gameplay to '{}'", replay_path.display());
                    Box::new(f)
                }
                Err(msg) => panic!(
                    "Failed to create the replay file at '{:?}'.
Reason: '{}'.",
                    replay_path.display(),
                    msg
                ),
            }
        } else {
            Box::new(io::sink())
        };

        log_seed(&mut writer, seed);
        let cheating = false;
        let replay = false;
        let replay_full_speed = false;
        State::new(
            world_size,
            map_size,
            panel_width,
            display_size,
            commands,
            verifications,
            writer,
            seed,
            cheating,
            invincible,
            replay,
            replay_full_speed,
            exit_after,
        )
    }

    #[cfg(not(feature = "replay"))]
    #[allow(dead_code)]
    pub fn replay_game(
        world_size: Point,
        map_size: i32,
        panel_width: i32,
        display_size: Point,
        _replay_path: &Path,
        invincible: bool,
        _replay_full_speed: bool,
        exit_after: bool,
    ) -> State {
        Self::new_game(
            world_size,
            map_size,
            panel_width,
            display_size,
            exit_after,
            None,
            invincible,
        )
    }

    #[cfg(feature = "replay")]
    pub fn replay_game(
        world_size: Point,
        map_size: i32,
        panel_width: i32,
        display_size: Point,
        replay_path: &Path,
        invincible: bool,
        replay_full_speed: bool,
        exit_after: bool,
    ) -> State {
        use serde_json;
        use std::io::{BufRead, BufReader};
        let mut commands = VecDeque::new();
        let mut verifications = VecDeque::new();
        let seed: u32;
        match File::open(replay_path) {
            Ok(file) => {
                let mut lines = BufReader::new(file).lines();
                match lines.next() {
                    Some(seed_str) => match seed_str.unwrap().parse() {
                        Ok(parsed_seed) => seed = parsed_seed,
                        Err(_) => panic!("The seed must be a number."),
                    },
                    None => panic!("The replay file is empty."),
                }

                loop {
                    match lines.next() {
                        Some(Ok(line)) => {
                            let command = serde_json::from_str(&line);
                            if let Ok(command) = command {
                                commands.push_back(command);
                            } else {
                                let verification = serde_json::from_str(&line).expect(&format!(
                                    "Couldn't load the \
                                     command or \
                                     verification: '{}'.",
                                    line
                                ));
                                verifications.push_back(verification);
                            }
                        }
                        Some(Err(err)) => panic!(
                            "Error reading a line from the replay \
                             file: {:?}.",
                            err
                        ),
                        None => break,
                    }
                }
            }
            Err(msg) => panic!(
                "Failed to read the replay file: {}. Reason: {}",
                replay_path.display(),
                msg
            ),
        }
        info!("Replaying game log: '{}'", replay_path.display());
        let cheating = true;
        let invincible = invincible;
        let replay = true;
        State::new(
            world_size,
            map_size,
            panel_width,
            display_size,
            commands,
            verifications,
            Box::new(io::sink()),
            seed,
            cheating,
            invincible,
            replay,
            replay_full_speed,
            exit_after,
        )
    }

    pub fn verification(&self) -> Verification {
        // TODO: we can sort the chunks and compare directly at some point.
        let chunks = self.world.positions_of_all_chunks();
        let mut monsters = vec![];
        for &chunk_pos in &chunks {
            for monster in self.world.chunk(chunk_pos).unwrap().monsters() {
                if !monster.dead {
                    monsters.push((monster.position, chunk_pos, monster.kind));
                }
            }
        }
        monsters
            .sort_by_key(|&(monster_pos, _chunk_pos, kind)| (monster_pos.x, monster_pos.y, kind));

        Verification {
            turn: self.turn,
            chunk_count: chunks.len(),
            player_pos: self.player.pos,
            monsters,
        }
    }

    pub fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        use bincode::serialize;

        // TODO: select the filename dynamicaly!
        let filename = "SAVEDGAME.sav";
        let buffer = serialize(self)?;

        // TODO: this can be compressed nicely!

        let mut file = File::create(filename)?;
        file.write_all(&buffer)?;
        file.flush()?;

        Ok(())
    }

    pub fn load_from_file() -> Result<State, Box<dyn Error>> {
        use bincode::deserialize;
        use std::io::Read;

        let filename = "SAVEDGAME.sav";
        let state = {
            let mut buffer = Vec::with_capacity(1_000_000);
            let mut file = File::open(filename)?;
            file.read_to_end(&mut buffer)?;
            deserialize(&buffer[..])?
        };

        if let Err(error) = ::std::fs::remove_file(filename) {
            error!(
                "Failed to delete the successfully loaded savegame. Error: {:?}",
                error
            );
        }

        Ok(state)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Window {
    MainMenu,
    Game,
    Help,
    Endgame,
    Message(String),
}

fn empty_command_logger() -> Box<dyn Write> {
    Box::new(io::sink())
}

pub fn log_seed<W: Write>(writer: &mut W, seed: u32) {
    writeln!(writer, "{}", seed).unwrap();
}

pub fn log_command<W: Write>(writer: &mut W, command: Command) {
    use serde_json;
    let json_command = serde_json::to_string(&command).expect(&format!(
        "Could not \
         serialise {:?} to \
         json.",
        command
    ));
    writeln!(writer, "{}", json_command).unwrap();
}

pub fn log_verification<W: Write>(writer: &mut W, verification: &Verification) {
    use serde_json;
    let json = serde_json::to_string(&verification).expect(&format!(
        "Could not \
         serialise \
         {:?} to json.",
        verification
    ));
    writeln!(writer, "{}", json).expect(&format!(
        "Could not write the \
         verification: '{}' to the \
         replay log.",
        json
    ));
}
