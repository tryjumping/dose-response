#![feature(if_let, macro_rules, globs, phase, link_args, unboxed_closures)]

extern crate collections;
extern crate libc;
extern crate time;


extern crate tcod;

use std::collections::RingBuf;
use std::os;

use tcod::{KeyState, Printable, Special};

use engine::{Engine, KeyCode};
use game_state::{GameState, Side};
use monster::Monster;
use point::Point;
use systems::input::Command;

mod color;
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
                    Special(KeyCode::Up) => commands.push_back(Command::N),
                    Special(KeyCode::Down) => commands.push_back(Command::S),
                    Special(KeyCode::Left) => match (ctrl(key), key.shift) {
                        (false, true) => commands.push_back(Command::NW),
                        (true, false) => commands.push_back(Command::SW),
                        _ => commands.push_back(Command::W),
                    },
                    Special(KeyCode::Right) => match (ctrl(key), key.shift) {
                        (false, true) => commands.push_back(Command::NE),
                        (true, false) => commands.push_back(Command::SE),
                        _ => commands.push_back(Command::E),
                    },
                    Printable('e') => {
                        commands.push_back(Command::Eat);
                    }
                    _ => (),
                }
            },
            None => break,
        }
    }
}


pub enum Action {
    Move((int, int)),
    Attack((int, int), monster::Damage),
    Eat,
}


fn process_player(state: &mut GameState) {
    if !state.level.player().alive() {
        return
    }
    if let Some(command) = state.commands.pop_front() {
        state.command_logger.log(command);
        let (x, y) = state.level.player().coordinates();
        let action = match command {
            Command::N => Action::Move((x,     y - 1)),
            Command::S => Action::Move((x,     y + 1)),
            Command::W => Action::Move((x - 1, y    )),
            Command::E => Action::Move((x + 1, y    )),

            Command::NW => Action::Move((x - 1, y - 1)),
            Command::NE => Action::Move((x + 1, y - 1)),
            Command::SW => Action::Move((x - 1, y + 1)),
            Command::SE => Action::Move((x + 1, y + 1)),

            Command::Eat => Action::Eat,
        };
        match action {
            Action::Move((x, y)) => {
                let (w, h) = state.level.size();
                let within_level = (x >= 0) && (y >= 0) && (x < w) && (y < h);
                if within_level {
                    if state.level.monster((x, y)).is_some() {
                        state.level.player_mut().spend_ap(1);
                        match state.level.kill_monster((x, y)).unwrap() {
                            Monster::Anxiety => {
                                println!("TODO: increase the anxiety kill counter / add one Will");
                            }
                            _ => {}
                        }
                    } else if state.level.walkable((x, y)) {
                        state.level.player_mut().spend_ap(1);
                        state.level.move_player((x, y));
                        loop {
                            match state.level.pickup_item((x, y)) {
                                Some(item) => {
                                    use item::Item::*;
                                    match item {
                                        Food => state.level.player_mut().inventory.push(item),
                                        Dose | StrongDose => {
                                            println!("TODO: use the dose");
                                        }
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                }
            }
            Action::Eat => {
                if let Some(food_idx) = state.level.player().inventory.iter().position(|&i| i == item::Item::Food) {
                    state.level.player_mut().inventory.remove(food_idx);
                    state.level.player_mut().spend_ap(1);
                    let food_explosion_radius = 2;
                    // TODO: move this to an "explode" procedure we can call elsewhere, too.
                    for expl_pos in point::points_within_radius(
                        state.level.player().coordinates(), food_explosion_radius) {
                        state.level.kill_monster(expl_pos);
                    }
                }
            }
            Action::Attack(_, _) => {
                unreachable!();
            }
        }
    }
}


fn process_monsters(state: &mut GameState) {
    if !state.level.player().alive() {
        return
    }
    let player_pos = state.level.player().coordinates();
    // TODO: we need to make sure these are always processed in the same order,
    // otherwise replay is bust!
    let mut monster_actions = vec![];
    for (&pos, monster) in state.level.monsters() {
        monster_actions.push((pos, monster.act(pos, player_pos, &state.level, &mut state.rng)));
    }
    for (pos, action) in monster_actions.into_iter() {
        match action {
            Action::Move(destination) => {
                if point::tile_distance(&pos, &destination) == 1 {
                    state.level.move_monster(pos, destination);
                } else {
                    let (w, h) = state.level.size();
                    // Walk one step:
                    let newpos_opt = {
                        let mut path = tcod::AStarPath::new_from_callback(
                            w, h,
                            |&mut: _from: (int, int), to: (int, int)| -> f32 {
                                if state.level.walkable(to) {
                                    1.0
                                } else {
                                    0.0
                                }
                            },
                            1.0);
                        path.find(pos.coordinates(), destination.coordinates());
                        assert!(path.len() != 1, "The path shouldn't be trivial. We already handled that.");
                        path.walk_one_step(true)
                    };
                    if let Some(newpos) = newpos_opt {
                        state.level.move_monster(pos, newpos);
                    }
                }
            }
            Action::Attack(target_pos, damage) => {
                assert!(target_pos == state.level.player().coordinates());
                state.level.player_mut().take_damage(damage);
            }
            Action::Eat => unreachable!(),
        }
    }
}


fn render_gui(display: &mut engine::Display, player: &player::Player) {
    let (_w, h) = display.size();
    let attribute_line = format!("SoM: {},  Will: {},  Food: {}",
                              player.state_of_mind,
                              player.will,
                              player.inventory.len());
    display.write_text(attribute_line.as_slice(), 0, h-1,
                       color::Color{r: 255, g: 255, b: 255},
                       color::Color{r: 0, g: 0, b: 0});

    let mut status_line = String::new();
    if player.alive() {
        if player.stun > 0 {
            status_line.push_str(format!("Stunned({})", player.stun).as_slice());
        }
        if player.panic > 0 {
            if status_line.len() > 0 {
                status_line.push_str(",  ");
            }
            status_line.push_str(format!("Panicking({})", player.panic).as_slice())
        }
    } else {
        status_line.push_str("Dead");
    }
    display.write_text(status_line.as_slice(), 0, h-2,
                       color::Color{r: 255, g: 255, b: 255},
                       color::Color{r: 0, g: 0, b: 0});
}


// TODO: use Duration instead of f32 for `dt`
fn update(mut state: GameState, _dt_s: f32, engine: &mut engine::Engine) -> Option<GameState> {
    if engine.key_pressed(Special(KeyCode::Escape)) {
        return None;
    }
    if engine.key_pressed(Special(KeyCode::F5)) {
        println!("Restarting game");
        engine.keys.clear();
        let (width, height) = state.display_size;
        let state = GameState::new_game(width, height);
        return Some(state);
    }

    if engine.key_pressed(Special(KeyCode::F6)) {
        state.cheating = !state.cheating;
        println!("Cheating set to: {}", state.cheating);
    }

    state.paused = if state.replay && engine.read_key(Special(KeyCode::Spacebar)) {
        if !state.paused {println!("Pausing the replay")};
        !state.paused
    } else {
        state.paused
    };

    // Move one step forward in the paused replay
    if state.paused && engine.read_key(Special(KeyCode::Right)) {
        unimplemented!();
    }

    process_keys(&mut engine.keys, &mut state.commands);
    match state.side {
        Side::Player => {
            process_player(&mut state);
            if !state.level.player_mut().has_ap(1) {
                state.side = Side::Computer;
            }
        }
        Side::Computer => {
            process_monsters(&mut state);
            state.side = Side::Player;
            state.level.player_mut().new_turn();
        }
    }

    state.level.render(&mut engine.display);
    render_gui(&mut engine.display, state.level.player());
    Some(state)
}



fn main() {
    let (width, height) = (80, 50);
    let title = "Dose Response";
    let font_path = Path::new("./fonts/dejavu16x16_gs_tc.png");

    let game_state = match os::args().len() {
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
