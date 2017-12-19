#![deny(overflowing_literals, unsafe_code)]
#![feature(conservative_impl_trait)]


#[macro_use]
extern crate bitflags;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[cfg(feature = "cli")]
extern crate clap;

#[cfg(feature = "replay")]
extern crate chrono;

#[cfg(feature = "web")]

#[macro_use]
#[cfg(feature = "opengl")]
extern crate glium;

#[cfg(feature = "piston")]
extern crate piston_window;

#[cfg(any(feature = "piston", feature = "opengl"))]
extern crate image;

#[cfg(feature = "libtcod")]
pub extern crate tcod;

#[cfg(feature = "terminal")]
extern crate rustbox;

#[cfg(feature = "remote")]
extern crate zmq;

use state::State;
use std::path::Path;

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
mod pathfinding;
mod player;
mod point;
mod ranged_int;
mod rect;
mod render;
mod state;
mod stats;
mod timer;
mod util;
mod world;


#[cfg(feature = "libtcod")]
fn run_libtcod(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    font_path: &Path,
    state: State,
) {
    println!("Using the libtcod backend.");
    let mut engine =
        engine::tcod::Engine::new(display_size, default_background, window_title, &font_path);
    engine.main_loop(state, update);
}

#[cfg(not(feature = "libtcod"))]
fn run_libtcod(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _font_path: &Path,
    _state: State,
) {
    println!("The \"libtcod\" feature was not compiled in.");
}

#[cfg(feature = "piston")]
fn run_piston(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    font_path: &Path,
    state: State,
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
fn run_piston(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _font_path: &Path,
    _state: State,
    _update: engine::UpdateFn<State>,
) {
    println!("The \"piston\" feature was not compiled in.");
}

#[cfg(feature = "terminal")]
fn run_terminal() {
    println!("Using the rustbox backend.\n  "
             "TODO: this is not implemented yet.");
}

#[cfg(not(feature = "terminal"))]
fn run_terminal() {
    println!("The \"terminal\" feature was not compiled in.");
}

#[cfg(feature = "opengl")]
fn run_opengl(
    display_size: point::Point,
    default_background: color::Color,
    window_title: &str,
    state: State,
    update: engine::UpdateFn<State>,
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
fn run_opengl(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _state: State,
    _update: engine::UpdateFn<State>,
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
fn run_remote(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _state: State,
    _update: engine::UpdateFn<State>,
) {
    println!("The \"remote\" feature was not compiled in.");
}


#[cfg(feature = "cli")]
fn process_cli_and_run_game(
    display_size: point::Point,
    world_size: point::Point,
    map_size: i32,
    panel_width: i32,
    default_background: color::Color,
    title: &str,
    update: engine::UpdateFn<State>,
) {
    use clap::{App, Arg, ArgGroup};

    let matches = App::new(title)
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
        let replay_path = Path::new(replay);
        State::replay_game(
            world_size,
            map_size,
            panel_width,
            display_size,
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
        State::new_game(
            world_size,
            map_size,
            panel_width,
            display_size,
            matches.is_present("exit-after"),
            replay_file,
            matches.is_present("invincible"),
        )
    };


    if matches.is_present("libtcod") {
        run_libtcod(
            display_size,
            default_background,
            title,
            &Path::new(""),
            state,
        );
    } else if matches.is_present("piston") {
        run_piston(
            display_size,
            default_background,
            title,
            &Path::new(""),
            state,
            update,
        );
    } else if matches.is_present("terminal") {
        run_terminal();
    } else if matches.is_present("remote") {
        run_remote(display_size, default_background, title, state, update);
    } else {
        run_opengl(display_size, default_background, title, state, update);
    }
}


#[cfg(not(feature = "cli"))]
fn process_cli_and_run_game(
    _display_size: point::Point,
    _world_size: point::Point,
    _map_size: i32,
    _panel_width: i32,
    _default_background: color::Color,
    _title: &str,
    _update: engine::UpdateFn<State>,
) {
    // TODO: run the game here
}



#[no_mangle]
pub extern "C" fn initialise() -> *mut State {
    let mut state = {
        // NOTE: at our current font, the height of 43 is the maximum
        // value for 1336x768 monitors.
        let map_size = 43;
        let panel_width = 20;
        let display_size: point::Point = (map_size + panel_width, map_size).into();
        // NOTE: 2 ^ 30
        let world_size: point::Point = (1_073_741_824, 1_073_741_824).into();
        let _title = "Dose Response";

        Box::new(State::new_game(
            world_size,
            map_size,
            panel_width,
            display_size,
            false,  // exit-after
            None,  // replay file
            false,  // invincible
        ))
    };

    // NOTE(shadower): if you uncomment tihs, we won't be able to access the memory from Rust
    update_state(&mut state);
    Box::into_raw(state)
}

extern "C" {
    fn draw(nums: *const u8, len: usize, counter: i32);
}


fn update_state(state: &mut State) {
    state.turn += 1;

    {
        // // TODO update a frame here
        // match STATE.try_lock() {
        //     Ok(mut static_state) => {
        //         // let state = static_state.take();
        //         // if let Some(state) = state {
        //         //     let dt = std::time::Duration::new(0, 0);
        //         //     let display_size = point::Point::new(0, 0);
        //         //     let fps = 60;
        //         //     let keys: Vec<keys::Key> = vec![];
        //         //     let mouse: engine::Mouse = Default::default();
        //         //     let settings = engine::Settings{ fullscreen: false };
        //         //     let mut drawcalls: Vec<engine::Draw> = vec![];

        //         //     let state_mem_ptr = state.mem.as_ptr();


        //         //     let result = game::update(
        //         //         state,
        //         //         dt,
        //         //         display_size,
        //         //         fps,
        //         //         &keys,
        //         //         mouse,
        //         //         settings,
        //         //         &mut drawcalls,
        //         //     );

        //         //     if let Some((settings, state)) = result {
        //         //         static_state.get_or_insert(state);
        //         //     }
        //         // }
        //         drop(static_state);

        //     }
        //     Err(state) => {
        //         unreachable!()
        //     }
        // }
    }
}


#[no_mangle]
pub extern "C" fn update(state_ptr: *mut State) {
    #[allow(unsafe_code)]
    let mut state: Box<State> = unsafe { Box::from_raw(state_ptr) };

    update_state(&mut state);
    let counter = state.turn;

    //  TODO: put some actual data here
    let mut drawcalls = vec![42; 10];
    drawcalls.push(counter as u8);

    #[allow(unsafe_code)]
    unsafe {
        draw(drawcalls.as_ptr(), drawcalls.len(), counter);
    }

    std::mem::forget(state);
}


fn main() {
    // NOTE: at our current font, the height of 43 is the maximum
    // value for 1336x768 monitors.
    // let map_size = 43;
    // let panel_width = 20;
    // let display_size = (map_size + panel_width, map_size).into();
    // // NOTE: 2 ^ 30
    // let world_size = (1_073_741_824, 1_073_741_824).into();
    // let title = "Dose Response";

    // process_cli_and_run_game(display_size, world_size, map_size, panel_width,
    //                          color::background, title, game::update);
}
