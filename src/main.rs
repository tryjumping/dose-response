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
#[cfg(not(feature = "web"))]
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
#[cfg(not(feature = "web"))]
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
#[cfg(not(feature = "web"))]
fn run_piston(
    _display_size: point::Point,
    _default_background: color::Color,
    _window_title: &str,
    _font_path: &Path,
    _state: State,
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
    state: State,
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
    _state: State,
    _update: engine::UpdateFn,
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
    update: engine::UpdateFn,
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
    _update: engine::UpdateFn,
) {
    // TODO: run the game here
}



#[cfg(feature = "web")]
#[no_mangle]
pub extern "C" fn initialise() -> *mut State {
    let state = {
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

    Box::into_raw(state)
}

#[cfg(feature = "web")]
extern {
    fn draw(nums: *const u8, len: usize);
    fn random() -> f32;
}


#[cfg(feature = "web")]
#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn update(state_ptr: *mut State, dt_ms: u32) {
    let mut state: Box<State> = unsafe { Box::from_raw(state_ptr) };

    let dt = std::time::Duration::from_millis(dt_ms as u64);
    let display_size = point::Point::new(0, 0);
    let fps = 60;
    let keys: Vec<keys::Key> = vec![];
    let mouse: engine::Mouse = Default::default();
    let mut settings = engine::Settings{ fullscreen: false };
    let mut drawcalls: Vec<engine::Draw> = vec![];

    let result = game::update(
        &mut state,
        dt,
        display_size,
        fps,
        &keys,
        mouse,
        &mut settings,
        &mut drawcalls,
    );

    match result {
        game::RunningState::Running => {}
        game::RunningState::NewGame(new_state) => {
            *state = new_state;
        }
        game::RunningState::Stopped => {},
    }

    engine::sort_drawcalls(&mut drawcalls, 0..);

    // Each "drawcall" will be 6 u8 values: x, y, char, r, g, b
    let mut js_drawcalls = Vec::with_capacity(drawcalls.len() * 6);
    for dc in &drawcalls {
        match dc {
            &engine::Draw::Char(pos, glyph, color) => {
                assert!(pos.x >= 0 && pos.x < 255);
                assert!(pos.y >= 0 && pos.y < 255);
                assert!(glyph.is_ascii());
                js_drawcalls.push(pos.x as u8);
                js_drawcalls.push(pos.y as u8);
                js_drawcalls.push(glyph as u8);
                js_drawcalls.push(color.r);
                js_drawcalls.push(color.g);
                js_drawcalls.push(color.b);
            }

            &engine::Draw::Text(start_pos, ref text, color) => {
                for (i, glyph) in text.char_indices() {
                    let pos = start_pos + (i as i32, 0);
                    assert!(pos.x >= 0 && pos.x < 255);
                    assert!(pos.y >= 0 && pos.y < 255);
                    assert!(glyph.is_ascii());
                    js_drawcalls.push(pos.x as u8);
                    js_drawcalls.push(pos.y as u8);
                    js_drawcalls.push(glyph as u8);
                    js_drawcalls.push(color.r);
                    js_drawcalls.push(color.g);
                    js_drawcalls.push(color.b);
                }
            }

            &engine::Draw::Rectangle(top_left, dimensions, color) => {
                if dimensions.x >= 1 && dimensions.y >= 1 {
                    let rect = rect::Rectangle::from_point_and_size(top_left, dimensions);
                    for pos in rect.points() {
                        assert!(pos.x >= 0 && pos.x < 255);
                        assert!(pos.y >= 0 && pos.y < 255);
                        js_drawcalls.push(pos.x as u8);
                        js_drawcalls.push(pos.y as u8);
                        js_drawcalls.push(0);
                        js_drawcalls.push(color.r);
                        js_drawcalls.push(color.g);
                        js_drawcalls.push(color.b);
                    }
                }
            }

            &engine::Draw::Background(pos, color) => {
                assert!(pos.x >= 0 && pos.x < 255);
                assert!(pos.y >= 0 && pos.y < 255);
                js_drawcalls.push(pos.x as u8);
                js_drawcalls.push(pos.y as u8);
                js_drawcalls.push(0);
                js_drawcalls.push(color.r);
                js_drawcalls.push(color.g);
                js_drawcalls.push(color.b);
            }

            &engine::Draw::Fade(fade, color) => {
                assert!(fade >= 0.0);
                assert!(fade <= 1.0);
                // NOTE: (255, 255) position means fade
                js_drawcalls.push(255);
                js_drawcalls.push(255);
                // NOTE: fade value/alpha is stored in the glyph
                js_drawcalls.push(((1.0 - fade) * 255.0) as u8);
                js_drawcalls.push(color.r);
                js_drawcalls.push(color.g);
                js_drawcalls.push(color.b);
            }

        }
    }

    unsafe {
        draw(js_drawcalls.as_ptr(), js_drawcalls.len());
    }

    std::mem::forget(state);
}

#[allow(unsafe_code)]
#[no_mangle]
pub extern "C" fn key_pressed(
    state_ptr: *mut State,
    external_code: u32,
    ctrl: bool, alt: bool, shift: bool
)
{
    let mut state: Box<State> = unsafe { Box::from_raw(state_ptr) };

    let code = from_js_keycode(external_code);
    if let Some(code) = code {
        state.keys.push(keys::Key { code, alt, ctrl, shift});
    }

    std::mem::forget(state);
}

fn from_js_keycode(js_keycode: u32) -> Option<keys::KeyCode> {
    use keys::KeyCode::*;
    match js_keycode {
        // Numbers in ASCII
        48 => Some(D0),
        49 => Some(D1),
        50 => Some(D2),
        51 => Some(D3),
        52 => Some(D4),
        53 => Some(D5),
        54 => Some(D6),
        55 => Some(D7),
        56 => Some(D8),
        57 => Some(D9),

        // Uppercase letters in ASCII
        65 => Some(A),
        66 => Some(B),
        67 => Some(C),
        68 => Some(D),
        69 => Some(E),
        70 => Some(F),
        71 => Some(G),
        72 => Some(H),
        73 => Some(I),
        74 => Some(J),
        75 => Some(K),
        76 => Some(L),
        77 => Some(M),
        78 => Some(N),
        79 => Some(O),
        80 => Some(P),
        81 => Some(Q),
        82 => Some(R),
        83 => Some(S),
        84 => Some(T),
        85 => Some(U),
        86 => Some(V),
        87 => Some(W),
        88 => Some(X),
        89 => Some(Y),
        90 => Some(Z),

        // Lowercase letters in ASCII
        97 => Some(A),
        98 => Some(B),
        99 => Some(C),
        100 => Some(D),
        101 => Some(E),
        102 => Some(F),
        103 => Some(G),
        104 => Some(H),
        105 => Some(I),
        106 => Some(J),
        107 => Some(K),
        108 => Some(L),
        109 => Some(M),
        110 => Some(N),
        111 => Some(O),
        112 => Some(P),
        113 => Some(Q),
        114 => Some(R),
        115 => Some(S),
        116 => Some(T),
        117 => Some(U),
        118 => Some(V),
        119 => Some(W),
        120 => Some(X),
        121 => Some(Y),
        122 => Some(Z),

        0xFFB0 => Some(NumPad0),
        0xFFB1 => Some(NumPad1),
        0xFFB2 => Some(NumPad2),
        0xFFB3 => Some(NumPad3),
        0xFFB4 => Some(NumPad4),
        0xFFB5 => Some(NumPad5),
        0xFFB6 => Some(NumPad6),
        0xFFB7 => Some(NumPad7),
        0xFFB8 => Some(NumPad8),
        0xFFB9 => Some(NumPad9),

        0xFFBE => Some(F1),
        0xFFBF => Some(F2),
        0xFFC0 => Some(F3),
        0xFFC1 => Some(F4),
        0xFFC2 => Some(F5),
        0xFFC3 => Some(F6),
        0xFFC4 => Some(F7),
        0xFFC5 => Some(F8),
        0xFFC6 => Some(F9),
        0xFFC7 => Some(F10),
        0xFFC8 => Some(F11),
        0xFFC9 => Some(F12),

        0xFF51 => Some(Left),
        0xFF53 => Some(Right),
        0xFF52 => Some(Up),
        0xFF54 => Some(Down),

        13 => Some(Enter),
        32 => Some(Space),
        27 => Some(Esc),

        _ => None,
    }
}



fn main() {
    // NOTE: at our current font, the height of 43 is the maximum
    // value for 1336x768 monitors.
    let map_size = 43;
    let panel_width = 20;
    let display_size = (map_size + panel_width, map_size).into();
    // NOTE: 2 ^ 30
    let world_size = (1_073_741_824, 1_073_741_824).into();
    let title = "Dose Response";

    process_cli_and_run_game(display_size, world_size, map_size, panel_width,
                             color::background, title, game::update);
}
