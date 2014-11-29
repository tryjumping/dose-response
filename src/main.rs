#![feature(if_let, macro_rules, globs, phase, link_args, unboxed_closures, tuple_indexing)]

extern crate libc;
extern crate time;
extern crate tcod;


use std::collections::RingBuf;
use std::os;
use std::rand::Rng;
use std::time::Duration;

use tcod::{KeyState, Printable, Special};

use engine::{Engine, KeyCode};
use game_state::{Command, GameState, Side};


mod color;
mod engine;
mod game_state;
mod generators;
mod graphics;
mod item;
mod level;
mod monster;
mod player;
mod point;
mod world;



fn process_keys(keys: &mut RingBuf<tcod::KeyState>, commands: &mut RingBuf<Command>) {
    fn ctrl(key: tcod::KeyState) -> bool {
        key.left_ctrl || key.right_ctrl
    }

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


#[deriving(Show)]
pub enum Action {
    Move((int, int)),
    Attack((int, int), player::Modifier),
    Eat,
}


fn kill_monster(monster: &mut monster::Monster, level: &mut level::Level) {
    monster.dead = true;
    level.remove_monster(monster.id(), monster);
}

fn explode(center: point::Point,
           radius: int,
           level: &mut level::Level,
           monsters: &mut Vec<monster::Monster>) {
    for pos in point::points_within_radius(center, radius) {
        if let Some(monster_id) = level.monster_on_pos(pos) {
            kill_monster(&mut monsters[monster_id], level);
        }
    }
}

fn process_player(player: &mut player::Player,
                  commands: &mut RingBuf<Command>,
                  level: &mut level::Level,
                  monsters: &mut Vec<monster::Monster>,
                  command_logger: &mut game_state::CommandLogger) {
    if !player.alive() {
        return
    }
    if let Some(command) = commands.pop_front() {
        command_logger.log(command);
        let (x, y) = player.pos;
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
                let (w, h) = level.size();
                let within_level = (x >= 0) && (y >= 0) && (x < w) && (y < h);
                if within_level {
                    if let Some(monster_id) = level.monster_on_pos((x, y)) {
                        player.spend_ap(1);
                        let monster = &mut monsters[monster_id];
                        assert_eq!(monster.id(), monster_id);
                        kill_monster(monster, level);
                        match monster.kind {
                            monster::Kind::Anxiety => {
                                println!("TODO: increase the anxiety kill counter / add one Will");
                            }
                            _ => {}
                        }
                    } else if level.walkable((x, y)) {
                        player.spend_ap(1);
                        player.move_to((x, y));
                        loop {
                            match level.pickup_item((x, y)) {
                                Some(item) => {
                                    use item::Kind::*;
                                    use player::Modifier::*;
                                    match item.kind {
                                        Food => player.inventory.push(item),
                                        Dose => {
                                            if let Intoxication{state_of_mind, ..} = item.modifier {
                                                let radius = match state_of_mind <= 100 {
                                                    true => 4,
                                                    false => 6,
                                                };
                                                player.take_effect(item.modifier);
                                                explode(player.pos, radius, level, monsters);
                                            } else {
                                                unreachable!();
                                            }
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
                if let Some(food_idx) = player.inventory.iter().position(|&i| i.kind == item::Kind::Food) {
                    player.spend_ap(1);
                    let food = player.inventory.remove(food_idx).unwrap();
                    player.take_effect(food.modifier);
                    let food_explosion_radius = 2;
                    explode(player.pos, food_explosion_radius, level, monsters);
                }
            }
            Action::Attack(_, _) => {
                unreachable!();
            }
        }
    }
}


fn process_monsters<R: Rng>(monsters: &mut Vec<monster::Monster>,
                            level: &mut level::Level,
                            player: &mut player::Player,
                            rng: &mut R) {
    if !player.alive() {
        return
    }

    for monster in monsters.iter_mut().filter(|m| !m.dead && m.has_ap(1)) {
        let action = monster.act(player.pos, level, rng);
        match action {
            Action::Move(destination) => {
                let pos = monster.position;
                let newpos_opt = if point::tile_distance(pos, destination) <= 1 {
                    Some(destination)
                } else {
                    let (w, h) = level.size();
                    {   // Find path && walk one step:
                        let mut path = tcod::AStarPath::new_from_callback(
                            w, h,
                            |&mut: _from: (int, int), to: (int, int)| -> f32 {
                                if level.walkable(to) {
                                    1.0
                                } else {
                                    0.0
                                }
                            },
                            1.0);
                        path.find(pos, destination);
                        assert!(path.len() != 1, "The path shouldn't be trivial. We already handled that.");
                        path.walk_one_step(false)
                    }
                };
                monster.spend_ap(1);
                match newpos_opt {
                    Some(step) => {
                        if level.monster_on_pos(step).is_none() {
                            level.move_monster(monster, step);
                        } else if step == monster.position {
                            println!("{} cannot move so it waits.", monster);
                        } else {
                            unreachable!();
                        }
                    }
                    None => {
                        println!("{} can't find a path so it waits.", monster);
                    }
                }
            }

            Action::Attack(target_pos, damage) => {
                assert!(target_pos == player.pos);
                monster.spend_ap(1);
                player.take_effect(damage);
                if monster.die_after_attack {
                    kill_monster(monster, level);
                }
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


fn update(mut state: GameState, dt: Duration, engine: &mut engine::Engine) -> Option<GameState> {
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
    state.clock = state.clock + dt;


    if engine.key_pressed(Special(KeyCode::F6)) {
        state.cheating = !state.cheating;
        println!("Cheating set to: {}", state.cheating);
    }

    state.paused = if state.replay && engine.read_key(Special(KeyCode::Spacebar)) {
        !state.paused
    } else {
        state.paused
    };

    let running = !state.paused && !state.replay;
    let paused_one_step = state.paused && engine.read_key(Special(KeyCode::Right));
    let timed_step = if state.replay && !state.paused && state.clock.num_milliseconds() >= 50 {
        state.clock = Duration::zero();
        true
    } else {
        false
    };
    if running || paused_one_step || timed_step {
        process_keys(&mut engine.keys, &mut state.commands);

        // Process player
        match state.side {
            Side::Player => {
                process_player(&mut state.player,
                               &mut state.commands,
                               &mut state.level,
                               &mut state.monsters,
                               &mut state.command_logger);
                if !state.player.has_ap(1) {
                    state.side = Side::Computer;
                    for monster in state.monsters.iter_mut() {
                        monster.new_turn();
                    }
                }
            }
            Side::Computer => {}
        }

        assert!(state.monsters.iter().enumerate().all(|(index, monster)| index == monster.id()),
                "Monster.id must always be equal to its index in state.monsters.");
        // Process monsters
        match state.side {
            Side::Player => {}
            Side::Computer => {
                process_monsters(&mut state.monsters, &mut state.level, &mut state.player, &mut state.rng);
                if state.monsters.iter().filter(|m| !m.dead).all(|m| !m.has_ap(1)) {
                    state.side = Side::Player;
                    state.player.new_turn();
                }
            }
        }
    }


    state.level.render(&mut engine.display);
    // TODO: assert no monster is on the same coords as the player
    // assert!(pos != self.player().coordinates(), "Monster can't be on the same cell as player.");
    for monster in state.monsters.iter().filter(|m| !m.dead) {
        graphics::draw(&mut engine.display, monster.position, monster);
    }
    graphics::draw(&mut engine.display, state.player.pos, &state.player);
    render_gui(&mut engine.display, &state.player);
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
