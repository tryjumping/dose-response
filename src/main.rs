#![deny(overflowing_literals, unsafe_code)]
#![windows_subsystem = "windows"]

extern crate bincode;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
extern crate rand;
#[cfg(feature = "cli")]
extern crate simplelog;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[cfg(feature = "cli")]
extern crate clap;

#[cfg(feature = "replay")]
extern crate chrono;

#[macro_use]
#[cfg(feature = "opengl")]
extern crate glium;

#[cfg(feature = "sdl")]
extern crate sdl2;

#[cfg(feature = "sdl")]
extern crate gl;

#[cfg(any(feature = "opengl", feature = "sdl"))]
extern crate image;

extern crate num_rational;

#[cfg(feature = "remote")]
extern crate zmq;

// NOTE: the external functions must be available in crate root:
#[cfg(feature = "web")]
pub use engine::wasm::{initialise, key_pressed, update};

mod ai;
mod animation;
mod blocker;
mod color;
mod engine;
mod formula;
mod game;
mod generators;
mod graphics;
mod item;
mod keys;
mod level;
mod monster;
mod palette;
mod pathfinding;
mod player;
mod point;
mod ranged_int;
mod rect;
mod render;
mod state;
mod stats;
mod timer;
mod ui;
mod util;
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

const GAME_TITLE: &str = "Dose Response";

#[allow(unused_variables)]
fn run_opengl(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    state: state::State,
    update: engine::UpdateFn,
) {
    info!("Using the glium+opengl backend");

    #[cfg(feature = "opengl")]
    engine::glium::main_loop(
        display_size,
        default_background,
        window_title,
        state,
        update,
    );

    #[cfg(not(feature = "opengl"))]
    error!("The \"opengl\" feature was not compiled in.");
}


#[allow(unused_variables)]
fn run_sdl(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    state: state::State,
    update: engine::UpdateFn,
) {
    info!("Using the sdl backend");

    #[cfg(feature = "sdl")]
    engine::sdl::main_loop(
        display_size,
        default_background,
        window_title,
        state,
        update,
    );

    #[cfg(not(feature = "sdl"))]
    error!("The \"sdl\" feature was not compiled in.");
}


#[allow(unused_variables)]
fn run_remote(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    state: state::State,
    update: engine::UpdateFn,
) {
    #[cfg(feature = "remote")]
    engine::remote::main_loop(
        display_size,
        default_background,
        window_title,
        state,
        update,
    );

    #[cfg(not(feature = "remote"))]
    error!("The \"remote\" feature was not compiled in.");
}

#[cfg(feature = "cli")]
fn process_cli_and_run_game() {
    use std::fs::File;
    use clap::{App, Arg, ArgGroup};
    use simplelog::{CombinedLogger, Config, LevelFilter, SimpleLogger, SharedLogger, WriteLogger};

    let mut loggers = vec![
        SimpleLogger::new(LevelFilter::Info, Config::default()) as Box<SharedLogger>,
    ];
    if let Ok(logfile) = File::create("dose-response.log") {
        loggers.push(WriteLogger::new(LevelFilter::Info, Config::default(), logfile));
    }
    // NOTE: ignore the loggers if we can't initialise them. The game
    // should still be able to function.
    let _ = CombinedLogger::init(loggers);

    let matches = App::new(GAME_TITLE)
        .author("Tomas Sedovic <tomas@sedovic.cz>")
        .about("Roguelike game about addiction")
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
        )
        .arg(
            Arg::with_name("exit-after")
                .help("Exit after the game or replay has finished")
                .long("exit-after"),
        )
        .arg(
            Arg::with_name("invincible")
                .help("Makes the player character invincible. They do not die.")
                .long("invincible"),
        )
        .arg(
            Arg::with_name("opengl")
                .long("opengl")
                .help("Use the Glium (OpenGL) rendering backend"),
        )
        .arg(
            Arg::with_name("sdl")
                .long("sdl")
                .help("Use the SDL2 rendering backend"),
        )
        .arg(Arg::with_name("remote").long("remote").help(
            "Don't create a game window. The input and output is \
             controled via ZeroMQ.",
        ))
        .group(
            ArgGroup::with_name("graphics")
                .args(&["opengl", "sdl", "remote"]),
        )
        .get_matches();

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
            DISPLAYED_MAP_SIZE,
            PANEL_WIDTH,
            DISPLAY_SIZE,
            &replay_path,
            matches.is_present("invincible"),
            matches.is_present("replay-full-speed"),
            matches.is_present("exit-after"),
        )
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
        state::State::new_game(
            WORLD_SIZE,
            DISPLAYED_MAP_SIZE,
            PANEL_WIDTH,
            DISPLAY_SIZE,
            matches.is_present("exit-after"),
            replay_file,
            matches.is_present("invincible"),
        )
    };

    if matches.is_present("remote") {
        run_remote(
            DISPLAY_SIZE,
            color::background,
            GAME_TITLE,
            state,
            game::update,
        );
    } else if matches.is_present("sdl") {
        run_sdl(
            DISPLAY_SIZE,
            color::background,
            GAME_TITLE,
            state,
            game::update,
        );
    } else {
        run_opengl(
            DISPLAY_SIZE,
            color::background,
            GAME_TITLE,
            state,
            game::update,
        );
    }
}

// NOTE: this function is intentionally empty and should stay here.
// Under wasm we don't want to run the game immediately because the
// game loop must be controlled from the browser not Rust. So we've
// provided external endpoints the browser will call in. But `main`
// still gets executed when the wasm binary is loaded.
#[cfg(not(feature = "cli"))]
fn process_cli_and_run_game() {}

fn main() {
    process_cli_and_run_game();
}
