#![forbid(overflowing_literals)]
#![deny(
    unsafe_code,
    rust_2018_idioms,
    rust_2018_compatibility,
    unused_extern_crates,
    nonstandard_style,
    future_incompatible,
    clippy::explicit_iter_loop,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::cloned_instead_of_copied,
    clippy::unnested_or_patterns,
    clippy::if_not_else,
    clippy::map_unwrap_or,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix,
    clippy::doc_markdown,
    // Prevent panics
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::ok_expect,
    clippy::panic,
    clippy::match_on_vec_items,
    clippy::manual_strip,
    clippy::await_holding_refcell_ref
)]
#![warn(missing_copy_implementations)]
#![allow(
    clippy::explicit_iter_loop,
    clippy::identity_op,
    clippy::wildcard_imports,
    clippy::match_bool,
    clippy::single_match_else,
    clippy::manual_slice_size_calculation,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::default_trait_access,
    clippy::ptr_as_ptr,
    clippy::float_cmp,
    clippy::from_iter_instead_of_collect,
    clippy::collapsible_else_if,
    clippy::bool_assert_comparison
)]

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
const DISPLAYED_MAP_SIZE: i32 = 30;

const PANEL_WIDTH: i32 = 17;
const DISPLAY_SIZE: point::Point = point::Point {
    x: DISPLAYED_MAP_SIZE + PANEL_WIDTH,
    y: DISPLAYED_MAP_SIZE,
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

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    use crate::settings::Store;
    use clap::{App, Arg};
    use simplelog::{CombinedLogger, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};
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
        )?
    } else {
        if matches.is_present("replay-full-speed") {
            throw!(
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

    Ok(())
}
