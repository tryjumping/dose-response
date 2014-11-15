#![feature(if_let, macro_rules, struct_variant, globs, phase, link_args, unboxed_closures)]

extern crate collections;
extern crate libc;
extern crate time;


extern crate tcod;

use std::collections::RingBuf;
use std::os;

use tcod::{KeyState, Printable, Special};

use engine::{Engine, key};
use game_state::GameState;
use point::Point;
use systems::input::commands;
use systems::input::commands::Command;

mod components;
mod engine;
//mod entity_util;
mod game_state;
mod level;
mod point;
mod systems;
mod world_gen;
mod world;


fn ctrl(key: tcod::KeyState) -> bool {
    key.left_ctrl || key.right_ctrl
}

fn process_keys(keys: &mut RingBuf<tcod::KeyState>, commands: &mut RingBuf<Command>) {
    // TODO: switch to DList and consume it with `mut_iter`.
    loop {
        match keys.pop_front() {
            Some(key) => {
                match key.key {
                    Special(key::Up) => commands.push_back(commands::N),
                    Special(key::Down) => commands.push_back(commands::S),
                    Special(key::Left) => match (ctrl(key), key.shift) {
                        (false, true) => commands.push_back(commands::NW),
                        (true, false) => commands.push_back(commands::SW),
                        _ => commands.push_back(commands::W),
                    },
                    Special(key::Right) => match (ctrl(key), key.shift) {
                        (false, true) => commands.push_back(commands::NE),
                        (true, false) => commands.push_back(commands::SE),
                        _ => commands.push_back(commands::E),
                    },
                    Printable('e') => {
                        commands.push_back(commands::Eat);
                    }
                    _ => (),
                }
            },
            None => break,
        }
    }
}

fn update(mut state: GameState, dt_s: f32, engine: &mut engine::Engine) -> Option<GameState> {
    if engine.key_pressed(Special(key::Escape)) {
        return None;
    }
    if engine.key_pressed(Special(key::F5)) {
        println!("Restarting game");
        engine.keys.clear();
        let (width, height) = state.display_size;
        let mut state = GameState::new_game(width, height);
        return Some(state);
    }

    if engine.key_pressed(Special(key::F6)) {
        state.cheating = !state.cheating;
        println!("Cheating set to: {}", state.cheating);
    }

    state.paused = if state.replay && engine.read_key(Special(key::Spacebar)) {
        if !state.paused {println!("Pausing the replay")};
        !state.paused
    } else {
        state.paused
    };

    // Move one step forward in the paused replay
    if state.paused && engine.read_key(Special(key::Right)) {
        unimplemented!();
    }

    process_keys(&mut engine.keys, &mut state.commands);

    // Process the player input
    if let Some(command) = state.commands.pop_front() {
        let (x, y) = state.level.player().coordinates();
        match command {
            commands::N => state.level.move_player((x,     y - 1)),
            commands::S => state.level.move_player((x,     y + 1)),
            commands::W => state.level.move_player((x - 1, y    )),
            commands::E => state.level.move_player((x + 1, y    )),

            commands::NW => state.level.move_player((x - 1, y - 1)),
            commands::NE => state.level.move_player((x + 1, y - 1)),
            commands::SW => state.level.move_player((x - 1, y + 1)),
            commands::SE => state.level.move_player((x + 1, y + 1)),

            commands::Eat => {
                unimplemented!();
            }
        }
    }

    state.level.render(&mut engine.display);
    Some(state)
}



fn main() {
    let (width, height) = (80, 50);
    let title = "Dose Response";
    let font_path = Path::new("./fonts/dejavu16x16_gs_tc.png");

    let mut game_state = match os::args().len() {
        1 => {  // Run the game with a new seed, create the replay log
            GameState::new_game(width, height)
        },
        2 => {  // Replay the game from the entered log
            GameState::replay_game(width, height)
        },
        _ => panic!("You must pass either pass zero or one arguments."),
    };

    let mut engine = Engine::new(width, height, title, font_path.clone());
    engine.main_loop(game_state, update);
}
