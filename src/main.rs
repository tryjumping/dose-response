#![deny(overflowing_literals, unsafe_code)]
#![feature(conservative_impl_trait)]


#[macro_use]
extern crate bitflags;
extern crate bincode;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[cfg(feature = "web")]
extern crate rmp_serde as rmps;

#[cfg(feature = "cli")]
extern crate clap;

#[cfg(feature = "replay")]
extern crate chrono;

#[macro_use]
#[cfg(feature = "opengl")]
extern crate glium;

#[cfg(feature = "piston")]
extern crate piston_window;

#[cfg(any(feature = "piston", feature = "opengl"))]
extern crate image;

extern crate num_rational;

#[cfg(feature = "libtcod")]
pub extern crate tcod;

#[cfg(feature = "terminal")]
extern crate rustbox;

#[cfg(feature = "remote")]
extern crate zmq;


// NOTE: the external functions must be available in crate root:
#[cfg(feature = "web")]
pub use engine::wasm::{key_pressed, initialise, update};


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
mod main_menu_window;
mod monster;
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
mod window_stack;
mod world;


// These are all in tiles and relate to how much we show on the screen.
//
// NOTE: at our current font size, the height of 43 tiles is the
// maximum value for 1336x768 monitors.
const DISPLAYED_MAP_SIZE: i32 = 43;
const PANEL_WIDTH: i32 = 20;
const DISPLAY_SIZE: point::Point = point::Point {
    x: DISPLAYED_MAP_SIZE + PANEL_WIDTH,
    y: DISPLAYED_MAP_SIZE,
};
const WORLD_SIZE: point::Point = point::Point {
    x: 1_073_741_824,
    y: 1_073_741_824,
};

const GAME_TITLE: &str = "Dose Response";


#[cfg(feature = "libtcod")]
fn run_libtcod(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    font_path: &std::path::Path,
    state: state::State,
) {
    println!("Using the libtcod backend.");
    let mut engine =
        engine::tcod::Engine::new(display_size, default_background, window_title, &font_path);
    engine.main_loop(state, update);
}

#[cfg(not(feature = "libtcod"))]
#[cfg(not(feature = "web"))]
fn run_libtcod(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _font_path: &std::path::Path,
    _state: state::State,
) {
    println!("The \"libtcod\" feature was not compiled in.");
}

#[cfg(feature = "piston")]
fn run_piston(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    font_path: &std::path::Path,
    state: state::State,
    update: engine::UpdateFn<State>,
) {
    println!("Using the piston backend.");
    engine::piston::main_loop(
        display_size,
        default_background,
        window_title,
        &font_path,
        state,
        update,
    );
}

#[cfg(not(feature = "piston"))]
#[cfg(not(feature = "web"))]
fn run_piston(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _font_path: &std::path::Path,
    _state: state::State,
    _update: engine::UpdateFn,
) {
    println!("The \"piston\" feature was not compiled in.");
}

#[cfg(feature = "terminal")]
fn run_terminal() {
    println!("Using the rustbox backend.\n  "
             "TODO: this is not implemented yet.");
}

#[cfg(not(feature = "terminal"))]
#[cfg(not(feature = "web"))]
fn run_terminal() {
    println!("The \"terminal\" feature was not compiled in.");
}

#[cfg(feature = "opengl")]
fn run_opengl(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    state: state::State,
    update: engine::UpdateFn,
) {
    println!("Using the default backend: opengl");
    engine::glium::main_loop(
        display_size,
        default_background,
        window_title,
        state,
        update,
    );
}

#[cfg(not(feature = "opengl"))]
#[cfg(not(feature = "web"))]
fn run_opengl(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _state: State,
    _update: engine::UpdateFn,
) {
    println!("The \"opengl\" feature was not compiled in.");
}

#[cfg(feature = "remote")]
fn run_remote(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    state: State,
    update: engine::UpdateFn<State>,
) {
    engine::remote::main_loop(
        display_size,
        default_background,
        window_title,
        state,
        update,
    );
}

#[cfg(not(feature = "remote"))]
#[cfg(not(feature = "web"))]
fn run_remote(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _state: state::State,
    _update: engine::UpdateFn,
) {
    println!("The \"remote\" feature was not compiled in.");
}


#[cfg(feature = "cli")]
fn process_cli_and_run_game() {
    use clap::{App, Arg, ArgGroup};

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
        .arg(Arg::with_name("libtcod").long("libtcod").help(
            "Use the libtcod rendering backend",
        ))
        .arg(Arg::with_name("piston").long("piston").help(
            "Use the Piston rendering backend",
        ))
        .arg(Arg::with_name("opengl").long("opengl").help(
            "Use the Glium (OpenGL) rendering backend",
        ))
        .arg(Arg::with_name("terminal").long("terminal").help(
            "Use the Rustbox (terminal-only) rendering backend",
        ))
        .arg(Arg::with_name("remote").long("remote").help(
            "Don't create a game window. The input and output is \
                    controled via ZeroMQ.",
        ))
        .group(ArgGroup::with_name("graphics").args(
            &[
                "libtcod",
                "piston",
                "opengl",
                "terminal",
                "remote",
            ],
        ))
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


    if matches.is_present("libtcod") {
        run_libtcod(
            DISPLAY_SIZE,
            color::background,
            GAME_TITLE,
            &std::path::Path::new(""),
            state,
        );
    } else if matches.is_present("piston") {
        run_piston(
            DISPLAY_SIZE,
            color::background,
            GAME_TITLE,
            &std::path::Path::new(""),
            state,
            game::update,
        );
    } else if matches.is_present("terminal") {
        run_terminal();
    } else if matches.is_present("remote") {
        run_remote(DISPLAY_SIZE, color::background, GAME_TITLE, state, game::update);
    } else {
        run_opengl(DISPLAY_SIZE, color::background, GAME_TITLE, state, game::update);
    }
}


// NOTE: this function is intentionally empty and should stay here.
// Under wasm we don't want to run the game immediately because the
// game loop must be controlled from the browser not Rust. So we've
// provided external endpoints the browser will call in. But `main`
// still gets executed when the wasm binary is loaded.
#[cfg(not(feature = "cli"))]
fn process_cli_and_run_game() {
}


fn main() {
    process_cli_and_run_game();
}
