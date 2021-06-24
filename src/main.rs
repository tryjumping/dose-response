#![deny(overflowing_literals, unsafe_code)]
#![allow(clippy::identity_op)]
#![windows_subsystem = "windows"]

mod ai;
mod animation;
mod audio;
mod blocker;
mod color;
mod engine;
mod graphic;
#[macro_use]
mod error;
mod formula;
mod game;
mod generators;
mod graphics;
mod item;
mod keys;
mod level;
mod metadata;
mod monster;
mod palette;
mod pathfinding;
mod player;
mod point;
mod random;
mod ranged_int;
mod rect;
mod render;
mod settings;
mod state;
mod stats;
mod timer;
mod ui;
mod util;
mod window;
mod windows;
mod world;

// These are all in tiles and relate to how much we show on the screen.
//
// NOTE: 53 x 30 tiles is the same aspect ratio as a widescreen
// monitor. So that's the ideal to strive for. But if we want to keep
// the display map square, the sidebar gets too wide.
//
// So instead, we've narrowed the sidebar to 17 tiles (just enough to
// make every withdrawal step show up). That means we don't maintain
// the perfect aspect ratio, but it seems to be good enough.
const DISPLAYED_MAP_SIZE: i32 = 30;

const PANEL_WIDTH: i32 = 17;
const DISPLAY_SIZE: point::Point = point::Point {
    x: DISPLAYED_MAP_SIZE + PANEL_WIDTH,
    y: DISPLAYED_MAP_SIZE,
};

const WORLD_SIZE: point::Point = point::Point {
    x: 1_073_741_824,
    y: 1_073_741_824,
};

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
    engine::glutin::main_loop(
        default_background,
        window_title,
        settings_store,
        Box::new(state),
    );

    #[cfg(not(feature = "glutin-backend"))]
    log::error!("The \"glutin-backend\" feature was not compiled in.");
}

#[allow(unused_variables, dead_code, clippy::needless_pass_by_value)]
fn run_remote(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    settings_store: settings::FileSystemStore,
    state: state::State,
) {
    #[cfg(feature = "remote")]
    engine::remote::main_loop(
        display_size,
        default_background,
        window_title,
        settings_store,
        Box::new(state),
        update,
    );

    #[cfg(not(feature = "remote"))]
    log::error!("The \"remote\" feature was not compiled in.");
}

fn main() {
    use crate::settings::Store;
    use clap::{App, Arg};
    use simplelog::{CombinedLogger, Config, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};
    use std::fs::File;

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
                .help("Print debug-level info. This can be really verbose."),
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

    if let Ok(logfile) = File::create("dose-response.log") {
        loggers.push(WriteLogger::new(log_level, Config::default(), logfile));
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

    log::info!(
        "Available text sizes: {:?}",
        crate::engine::AVAILABLE_TEXT_SIZES
    );

    log::info!(
        "Available graphics backends: {:?}",
        crate::engine::AVAILABLE_BACKENDS
    );

    let settings_store = settings::FileSystemStore::new();
    let settings = settings_store.load();
    let backend = settings.backend.clone();
    let challenge = settings.challenge();
    let palette = settings.palette();

    let state = if let Some(replay) = matches.value_of("replay") {
        if matches.is_present("replay-file") {
            panic!(
                "The `replay-file` option can only be used during regular \
                 game, not replay."
            );
        }
        let replay_path = std::path::Path::new(replay);
        state::State::replay_game(
            WORLD_SIZE,
            point::Point::from_i32(DISPLAYED_MAP_SIZE),
            PANEL_WIDTH,
            replay_path,
            matches.is_present("cheating"),
            matches.is_present("invincible"),
            matches.is_present("replay-full-speed"),
            matches.is_present("exit-after"),
            challenge,
            palette,
        )
        .expect("Could not load the replay file")
    } else {
        if matches.is_present("replay-full-speed") {
            panic!(
                "The `full-replay-speed` option can only be used if the \
                 replay log is passed."
            );
        }
        let replay_file = match matches.value_of("replay-file") {
            Some(file) => Some(file.into()),
            None => state::generate_replay_path(),
        };
        let mut state = state::State::new_game(
            WORLD_SIZE,
            point::Point::from_i32(DISPLAYED_MAP_SIZE),
            PANEL_WIDTH,
            matches.is_present("exit-after"),
            replay_file,
            challenge,
            palette,
        );
        state.player.invincible = matches.is_present("invincible");
        state.window_stack.push(window::Window::MainMenu);
        state
    };

    let display_size = DISPLAY_SIZE;
    let background = palette.unexplored_background;
    let game_title = metadata::TITLE;

    match backend.as_str() {
        "remote" => run_remote(display_size, background, game_title, settings_store, state),
        "glutin" => run_glutin(background, game_title, settings_store, state),
        _ => {
            log::error!("Unknown backend: {}", backend);
        }
    }
}
