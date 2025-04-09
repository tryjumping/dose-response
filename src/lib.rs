macro_rules! throw {
    ($message:expr) => {
        return core::result::Result::Err(std::boxed::Box::new(crate::error::Error::new($message)))
    };
}

pub mod ai;
pub mod animation;
pub mod audio;
pub mod blocker;
pub mod color;
pub mod engine;
pub mod error;
pub mod formula;
pub mod game;
pub mod gamepad;
pub mod generators;
pub mod graphic;
pub mod graphics;
pub mod item;
pub mod keys;
pub mod level;
pub mod metadata;
pub mod monster;
pub mod palette;
pub mod pathfinding;
pub mod player;
pub mod point;
pub mod random;
pub mod ranged_int;
pub mod rect;
pub mod render;
pub mod settings;
pub mod state;
pub mod stats;
pub mod timer;
pub mod ui;
pub mod util;
pub mod window;
pub mod windows;
pub mod world;

pub const WORLD_SIZE: point::Point = point::Point {
    x: 1_073_741_824,
    y: 1_073_741_824,
};

use simplelog::Config;

// These are all in tiles and relate to how much we show on the screen.
//
// NOTE: 53 x 30 tiles is the same aspect ratio as a widescreen
// monitor. So that's the ideal to strive for. But if we want to keep
// the display map square, the sidebar gets too wide.
//
// So instead, we've narrowed the sidebar to 17 tiles (just enough to
// make every withdrawal step show up). That means we don't maintain
// the perfect aspect ratio, but it seems to be good enough.
pub const DISPLAYED_MAP_SIZE: i32 = 30;

pub const PANEL_WIDTH: i32 = 17;

#[allow(unused_variables, dead_code, clippy::needless_pass_by_value)]
fn run_glutin(
    default_background: color::Color,
    window_title: &str,
    settings_store: settings::FileSystemStore,
    state: state::State,
) {
    log::info!("Using the glutin backend");

    // // TODO: figure out how to record screenshots with glutin!
    // let (fixed_fps, replay_dir) = if record_replay {
    //     (Some(60), Some("/home/thomas/tmp/dose-response-recording"))
    // } else {
    //     (None, None)
    // };

    #[cfg(feature = "glutin-backend")]
    let result = engine::glutin::main_loop(
        default_background,
        window_title,
        settings_store,
        Box::new(state),
    );
    if let Err(err) = result {
        log::error!("Error occured in the glutin main_loop: {}", err);
    };

    #[cfg(not(feature = "glutin-backend"))]
    log::error!("The \"glutin-backend\" feature was not compiled in.");
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    use crate::settings::Store;
    use clap::{App, Arg};
    use simplelog::{CombinedLogger, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};
    use std::fs::File;

    // Print out all environment variables:
    log::info!("Environment variables:");
    for (key, value) in std::env::vars() {
        log::info!("{key}: {value}");
    }

    let mut app = App::new(metadata::TITLE)
        .version(metadata::VERSION)
        .author(metadata::AUTHORS)
        .about(metadata::DESCRIPTION)
        .arg(
            Arg::with_name("exit-after")
                .help("Exit after the game or replay has finished")
                .long("exit-after"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Don't write any messages to stdout."),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("Debug mode. Output detailed messages and replay logs. This can be really verbose and take up massive amounts of space."),
        );

    if cfg!(feature = "cheating") {
        app = app
            .arg(
                Arg::with_name("cheating")
                    .help("Opens the cheat mode on start. Uncovers the map.")
                    .long("cheating"),
            )
            .arg(
                Arg::with_name("invincible")
                    .help("Makes the player character invincible. They do not die.")
                    .long("invincible"),
            );
    }

    if cfg!(feature = "replay") {
        app = app
            // NOTE: this is a positional argument, because it doesn't
            // have `.short` or `.long` set. It looks similar to a
            // "keyword" arguments that take values (such as
            // --replay-file) but it's different.
            .arg(
                Arg::with_name("replay")
                    .value_name("FILE")
                    .help(
                        "Replay this file instead of starting and playing a new \
                         game",
                    )
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("replay-full-speed")
                    .help(
                        "Don't slow the replay down (useful for getting accurate \
                         measurements)",
                    )
                    .long("replay-full-speed"),
            )
	    .arg(
		Arg::with_name("headless")
		    .help("Run the replay in a headless mode. No window will be open but the game will play through the full replay log. This can be useful for automated testing.")
		    .long("headless"))
            .arg(
                Arg::with_name("replay-file")
                    .help("Path where to store the replay log.")
                    .long("replay-file")
                    .value_name("FILE")
                    .takes_value(true),
            );
    }

    if cfg!(feature = "recording") {
        app = app.arg(
            Arg::with_name("record-frames")
                .long("record-frames")
                .help("Whether to record the frames and save them on disk."),
        );
    }

    let matches = app.get_matches();

    let mut loggers = vec![];

    let log_level = if matches.is_present("debug") {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    if !matches.is_present("quiet") {
        loggers.push(SimpleLogger::new(log_level, Config::default()) as Box<dyn SharedLogger>);
    }

    let file_log_config = simplelog::ConfigBuilder::new()
        // // NOTE: This disables logging the datetime in messages. Useful for diffing playthroughts. Uncomment to disable datetime logging.
        // 	.set_time_format_custom(time::macros::format_description!(""))
        .build();

    if let Ok(logfile) = File::create("dose-response.log") {
        loggers.push(WriteLogger::new(
            LevelFilter::Trace,
            file_log_config,
            logfile,
        ));
    }

    // NOTE: ignore the loggers if we can't initialise them. The game
    // should still be able to function.
    let _ = CombinedLogger::init(loggers);

    log_panics::init();

    log::info!("{} version: {}", metadata::TITLE, metadata::VERSION);
    log::info!("By: {}", metadata::AUTHORS);
    log::info!("{}", metadata::HOMEPAGE);

    let hash = metadata::GIT_HASH.trim();
    if !hash.is_empty() {
        log::info!("Git commit: {}", hash);
    }
    let target_triple = metadata::TARGET_TRIPLE.trim();
    if !target_triple.is_empty() {
        log::info!("Target triple: {}", target_triple);
    }

    log::info!("Build profile: {}", metadata::PROFILE);
    log::info!("Optimisation level: {}", metadata::OPT_LEVEL);
    log::info!("Build features: {}", metadata::FEATURES);
    log::info!("Build configs: {}", metadata::CONFIGS);

    log::info!(
        "Available text sizes: {:?}",
        crate::engine::AVAILABLE_TEXT_SIZES
    );

    log::info!(
        "Available graphics backends: {:?}",
        crate::engine::AVAILABLE_BACKENDS
    );

    let mut settings_store = settings::FileSystemStore::new();
    let mut settings = settings_store.load();
    let backend = settings.backend.clone();
    let challenge = settings.challenge();
    let palette = settings.palette();

    let state = if let Some(replay) = matches.value_of("replay") {
        if matches.is_present("replay-file") {
            throw!(
                "The `replay-file` option can only be used during regular \
                 game, not replay."
            );
        }

        // Always exit after the replay finishes when in the headless
        // mode. Otherwise the game will loop on the end-game screen
        // and since there's no way to interact with the headless
        // mode, it'll never exit.
        let exit_after = if matches.is_present("headless") {
            true
        } else {
            matches.is_present("exit-after")
        };

        let replay_path = std::path::Path::new(replay);
        state::State::replay_game(
            WORLD_SIZE,
            point::Point::from_i32(DISPLAYED_MAP_SIZE),
            PANEL_WIDTH,
            replay_path,
            matches.is_present("cheating"),
            matches.is_present("invincible"),
            matches.is_present("replay-full-speed"),
            exit_after,
            matches.is_present("debug"),
            challenge,
            palette,
        )?
    } else {
        if matches.is_present("replay-full-speed") {
            throw!(
                "The `full-replay-speed` option can only be used if the \
                 replay log is passed."
            );
        }
        let replay_file = match matches.value_of("replay-file") {
            Some(file) => {
                let replay_path: std::path::PathBuf = file.into();
                if replay_path.exists() {
                    throw!(
                        "The replay file provided by the `--replay-file` option exists already. Not going to overwrite it, aborting."
                    );
                }
                Some(replay_path)
            }
            None => state::generate_replay_path(),
        };
        let mut state = state::State::new_game(
            WORLD_SIZE,
            point::Point::from_i32(DISPLAYED_MAP_SIZE),
            PANEL_WIDTH,
            matches.is_present("exit-after"),
            matches.is_present("debug"),
            replay_file,
            challenge,
            palette,
        );
        state.player.invincible = matches.is_present("invincible");

        state.window_stack = windows::Windows::new(window::Window::Game);
        if settings.first_ever_startup {
            // Start the game with the game on top, don't push in any other window.
            // Just like in Braid, basically.
            state.generate_world();
            state.game_session = state::GameSession::InProgress;
            //
            // Mark any future runs as not the very first one:
            settings.first_ever_startup = false;
            settings_store.save(&settings);
        } else {
            // Open
            state.window_stack.push(window::Window::MainMenu);
        }

        state
    };

    let background = palette.unexplored_background;
    let game_title = metadata::TITLE;

    if matches.is_present("headless") && matches.is_present("replay") {
        log::info!("Run in the headless mode");

        let result = engine::headless::main_loop(settings_store, Box::new(state));

        return result;
    }

    match backend.as_str() {
        "glutin" => run_glutin(background, game_title, settings_store, state),
        _ => {
            log::error!("Unknown backend: {}", backend);
        }
    }

    Ok(())
}
