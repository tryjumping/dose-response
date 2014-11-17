#![feature(if_let, macro_rules, globs, phase, link_args, unboxed_closures)]

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

mod color;
mod components;
mod engine;
//mod entity_util;
mod game_state;
mod item;
mod level;
mod monster;
mod player;
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


enum Action {
    Move(int, int),
    Eat,
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
        let action = match command {
            commands::N => Move(x,     y - 1),
            commands::S => Move(x,     y + 1),
            commands::W => Move(x - 1, y    ),
            commands::E => Move(x + 1, y    ),

            commands::NW => Move(x - 1, y - 1),
            commands::NE => Move(x + 1, y - 1),
            commands::SW => Move(x - 1, y + 1),
            commands::SE => Move(x + 1, y + 1),

            commands::Eat => Eat,
        };
        match action {
            Move(x, y) => {
                let (w, h) = state.level.size();
                let within_level = (x >= 0) && (y >= 0) && (x < w) && (y < h);
                let walkable = match state.level.cell((x, y)).tile {
                    level::Empty => true,
                    _ => false,
                };
                if within_level {
                    if state.level.cell((x, y)).monster.is_some() {
                        match state.level.kill_monster((x, y)).unwrap() {
                            monster::Anxiety => {
                                println!("TODO: increase the anxiety kill counter / add one Will");
                            }
                            _ => {}
                        }
                    } else if walkable {
                        state.level.move_player((x, y));
                        loop {
                            match state.level.pickup_item((x, y)) {
                                Some(item) => {
                                    println!("Picked up item {}", item);
                                }
                                None => break,
                            }
                        }
                    }
                }
            }
            Eat => {
                unimplemented!();
            }
        }
    }


    // TODO: Process the monsters:
    // for each monster:
    // let action  = monster.run_ai()
    // match action {
    //     Move(x, y) => ...,
    // }


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
