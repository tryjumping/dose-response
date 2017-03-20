#![deny(overflowing_literals, unsafe_code)]

#[macro_use]
extern crate bitflags;
extern crate clap;
extern crate rand;
extern crate time;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
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


use std::borrow::Cow;
use std::collections::VecDeque;
use std::cmp;
use std::io::Write;
use std::path::Path;

use rand::Rng;
use time::Duration;

use animation::AreaOfEffect;
use engine::{Draw, Settings};
use game_state::{Command, GameState, Side};
use keys::{Key, KeyCode};


mod animation;
mod color;
mod engine;
mod game_state;
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
mod stats;
mod timer;
mod world;



fn process_keys(keys: &mut keys::Keys, commands: &mut VecDeque<Command>) {
    use keys::KeyCode::*;
    while let Some(key) = keys.get() {
        match key {
            // Numpad (8246 for cardinal and 7193 for diagonal movement)
            Key { code: NumPad8, .. } => commands.push_back(Command::N),
            Key { code: NumPad2, .. } => commands.push_back(Command::S),
            Key { code: NumPad4, .. } => commands.push_back(Command::W),
            Key { code: NumPad6, .. } => commands.push_back(Command::E),
            Key { code: NumPad7, .. } => commands.push_back(Command::NW),
            Key { code: NumPad1, .. } => commands.push_back(Command::SW),
            Key { code: NumPad9, .. } => commands.push_back(Command::NE),
            Key { code: NumPad3, .. } => commands.push_back(Command::SE),

            // NotEye (arrow keys plus Ctrl and Shift modifiers for horizontal movement)
            Key { code: Up, ..}      => commands.push_back(Command::N),
            Key { code: Down, ..}    => commands.push_back(Command::S),
            Key { code: Left, ctrl: false, shift: true, .. }   => commands.push_back(Command::NW),
            Key { code: Left, ctrl: true, shift: false, .. }   => commands.push_back(Command::SW),
            Key { code: Left, .. }   => commands.push_back(Command::W),
            Key { code: Right, ctrl: false, shift: true, .. }  => commands.push_back(Command::NE),
            Key { code: Right, ctrl: true, shift: false, .. }  => commands.push_back(Command::SE),
            Key { code: Right, .. }  => commands.push_back(Command::E),

            // Vi keys (hjkl for cardinal and yunm for diagonal movement)
            Key { code: K, .. } => commands.push_back(Command::N),
            Key { code: J, .. }  => commands.push_back(Command::S),
            Key { code: H, .. }  => commands.push_back(Command::W),
            Key { code: L, .. }  => commands.push_back(Command::E),
            Key { code: Y, .. }  => commands.push_back(Command::NW),
            Key { code: N, .. }  => commands.push_back(Command::SW),
            Key { code: U, .. }  => commands.push_back(Command::NE),
            Key { code: M, .. }  => commands.push_back(Command::SE),

            // Non-movement commands
            Key { code: E, .. } | Key { code: D1, .. } => {
                commands.push_back(Command::UseFood);
            }
            Key { code: D2, ..} => {
                commands.push_back(Command::UseDose);
            }
            Key { code: D3, ..} => {
                commands.push_back(Command::UseStrongDose);
            }
            _ => match inventory_commands(key) {
                Some(command) => commands.push_back(command),
                None => (),
            },
        }
    }
}

fn inventory_commands(key: Key) -> Option<Command> {
    use keys::KeyCode::*;
    use item::Kind;

    for kind in Kind::iter() {
        let num_key = match inventory_key(kind) {
            1 => D1,
            2 => D2,
            3 => D3,
            4 => D4,
            5 => D5,
            6 => D6,
            7 => D7,
            8 => D8,
            9 => D9,
            _ => unreachable!("There should only even be 9 item kinds at most."),
        };

        if key.code == num_key {
            let command = match kind {
                Kind::Food => Command::UseFood,
                Kind::Dose => Command::UseDose,
                Kind::CardinalDose => Command::UseCardinalDose,
                Kind::DiagonalDose => Command::UseDiagonalDose,
                Kind::StrongDose => Command::UseStrongDose,
            };
            return Some(command);
        }
    }
    None
}


fn inventory_key(kind: item::Kind) -> u8 {
    use item::Kind::*;
    match kind {
        Food => 1,
        Dose => 2,
        CardinalDose => 3,
        DiagonalDose => 4,
        StrongDose => 5,
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Move(point::Point),
    Attack(point::Point, player::Modifier),
    Use(item::Kind),
}


fn kill_monster(monster_position: point::Point, world: &mut world::World) {
    if let Some(monster) = world.monster_on_pos(monster_position) {
        monster.dead = true;
    }
    world.remove_monster(monster_position);
}

fn use_dose(player: &mut player::Player,
            explosion_animation: &mut Option<Box<AreaOfEffect>>,
            item: item::Item) {
    use player::Modifier::*;
    use item::Kind::*;
    // TODO: do a different explosion animation for the cardinal dose
    if let Intoxication{state_of_mind, ..} = item.modifier {
        let radius = match state_of_mind <= 100 {
            true => 4,
            false => 6,
        };
        player.take_effect(item.modifier);
        let animation: Box<AreaOfEffect> = match item.kind {
            Dose | StrongDose =>
                Box::new(animation::SquareExplosion::new(player.pos, radius, 2, color::explosion)),
            CardinalDose =>
                Box::new(
                    animation::CardinalExplosion::new(
                        player.pos, radius, 2, color::explosion, color::shattering_explosion)),
            DiagonalDose =>
                Box::new(
                    animation::DiagonalExplosion::new(
                        player.pos, radius, 2, color::explosion, color::shattering_explosion)),
            Food => unreachable!(),

        };
        *explosion_animation = Some(animation);
    } else {
        unreachable!();
    }
}

fn exploration_radius(mental_state: player::Mind) -> i32 {
    use player::Mind::*;
    match mental_state {
        Withdrawal(value) => {
            if *value >= value.middle() {
                5
            } else {
                4
            }
        }
        Sober(_) => 6,
        High(value) => {
            if *value >= value.middle() {
                8
            } else {
                7
            }
        }
    }
}

fn player_resist_radius(dose_irresistible_value: i32, will: i32) -> i32 {
    cmp::max(dose_irresistible_value + 1 - will, 0)
}


fn process_player(state: &mut game_state::GameState) {
    let previous_action_points = state.player.ap();

    process_player_action(&mut state.player,
                          &mut state.commands,
                          &mut state.world,
                          &mut state.explosion_animation,
                          &mut state.rng,
                          &mut state.command_logger);

    let spent_ap_this_turn = previous_action_points > state.player.ap();

    // Increase the sobriety counter if the player behaved themself.
    if spent_ap_this_turn && !state.player.mind.is_high() && state.player.will.is_max() {
        state.player.sobriety_counter += 1;
    }

    // Set the longest high streak
    if spent_ap_this_turn {
        if state.player.mind.is_high() {
            state.player.current_high_streak += 1;
            if state.player.current_high_streak > state.player.longest_high_streak {
                state.player.longest_high_streak = state.player.current_high_streak;
            }
        } else {
            state.player.current_high_streak = 0;
        }
    }

    // NOTE: The player has stayed sober long enough. Victory! \o/
    if state.player.sobriety_counter.is_max() {
        state.side = Side::Victory;
    }

    state.world.explore(state.player.pos, exploration_radius(state.player.mind));
}

fn process_player_action<R, W>(player: &mut player::Player,
                               commands: &mut VecDeque<Command>,
                               world: &mut world::World,
                               explosion_animation: &mut Option<Box<AreaOfEffect>>,
                               rng: &mut R,
                               command_logger: &mut W)
    where R: Rng, W: Write {
    if !player.alive() || !player.has_ap(1) {
        return
    }

    if let Some(command) = commands.pop_front() {
        game_state::log_command(command_logger, command);
        let mut action = match command {
            Command::N => Action::Move(player.pos + ( 0, -1)),
            Command::S => Action::Move(player.pos + ( 0,  1)),
            Command::W => Action::Move(player.pos + (-1,  0)),
            Command::E => Action::Move(player.pos + ( 1,  0)),

            Command::NW => Action::Move(player.pos + (-1, -1)),
            Command::NE => Action::Move(player.pos + ( 1, -1)),
            Command::SW => Action::Move(player.pos + (-1,  1)),
            Command::SE => Action::Move(player.pos + ( 1,  1)),

            Command::UseFood => Action::Use(item::Kind::Food),
            Command::UseDose => Action::Use(item::Kind::Dose),
            Command::UseCardinalDose => Action::Use(item::Kind::CardinalDose),
            Command::UseDiagonalDose => Action::Use(item::Kind::DiagonalDose),
            Command::UseStrongDose => Action::Use(item::Kind::StrongDose),
        };

        if *player.stun > 0 {
            action = Action::Move(player.pos);
        } else if *player.panic > 0 {
            let new_pos = world.random_neighbour_position(
                rng, player.pos, level::Walkability::WalkthroughMonsters);
            action = Action::Move(new_pos);

        } else if let Some((dose_pos, dose)) = world.nearest_dose(player.pos, 5) {
            let resist_radius = player_resist_radius(dose.irresistible, *player.will) as usize;
            if player.pos.tile_distance(dose_pos) < resist_radius as i32 {
                // TODO: think about caching the discovered path or partial path-finding??
                let mut path = pathfinding::Path::find(player.pos, dose_pos, world,
                                                       level::Walkability::WalkthroughMonsters);

                let new_pos_opt = if path.len() <= resist_radius {
                    path.next()
                } else {
                    None
                };

                if let Some(new_pos) = new_pos_opt {
                    action = Action::Move(new_pos);
                } else {
                    //println!("Can't find path to irresistable dose at {:?} from player's position {:?}.", dose_pos, player.pos);
                }
            }
        }

        // NOTE: If we have doses in the inventory that we wouldn't be
        // able to pick up anymore, use them up one by one each turn:
        let carried_irresistible_dose = player.inventory.iter()
            .find(|i| i.is_dose() && player_resist_radius(i.irresistible, *player.will) > 0)
            .map(|i| i.kind);
        if let Some(kind) = carried_irresistible_dose {
            action = Action::Use(kind);
        }

        match action {
            Action::Move(dest) => {
                if world.within_bounds(dest) {
                    let dest_walkable = world.walkable(dest, level::Walkability::BlockingMonsters);
                    let bumping_into_monster = world.monster_on_pos(dest).is_some();
                    if bumping_into_monster {
                        player.spend_ap(1);
                        //println!("Player attacks {:?}", monster);
                        if let Some(monster) = world.monster_on_pos(dest) {
                            match monster.kind {
                                monster::Kind::Anxiety => {
                                    player.anxiety_counter += 1;
                                    if player.anxiety_counter.is_max() {
                                        player.will += 1;
                                        player.anxiety_counter.set_to_min();
                                    }
                                }
                                _ => {}
                            }
                        }
                        kill_monster(dest, world);

                    } else if dest_walkable {
                        player.spend_ap(1);
                        player.move_to(dest);
                        loop {
                            match world.pickup_item(dest) {
                                Some(item) => {
                                    use item::Kind::*;
                                    match item.kind {
                                        Food => player.inventory.push(item),
                                        Dose | StrongDose | CardinalDose | DiagonalDose => {
                                            if player_resist_radius(item.irresistible, *player.will) == 0 {
                                                player.inventory.push(item);
                                            } else {
                                                use_dose(player, explosion_animation, item);
                                            }
                                        }
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                } else {
                    // TODO: Walk to the neighbouring chunk!
                    unimplemented!()
                }
            }

            Action::Use(item::Kind::Food) => {
                if let Some(food_idx) = player.inventory.iter().position(|&i| i.kind == item::Kind::Food) {
                    player.spend_ap(1);
                    let food = player.inventory.remove(food_idx);
                    player.take_effect(food.modifier);
                    let food_explosion_radius = 2;
                    let animation = animation::SquareExplosion::new(player.pos, food_explosion_radius, 1, color::explosion);
                    *explosion_animation = Some(Box::new(animation));
                }
            }

            Action::Use(item::Kind::Dose) => {
                if let Some(dose_index) = player.inventory.iter().position(|&i| i.kind == item::Kind::Dose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::StrongDose) => {
                if let Some(dose_index) = player.inventory.iter().position(|&i| i.kind == item::Kind::StrongDose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::CardinalDose) => {
                if let Some(dose_index) = player.inventory.iter().position(|&i| i.kind == item::Kind::CardinalDose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::DiagonalDose) => {
                if let Some(dose_index) = player.inventory.iter().position(|&i| i.kind == item::Kind::DiagonalDose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Attack(_, _) => {
                unreachable!();
            }
        }
    }
}


fn process_monsters<R: Rng>(world: &mut world::World,
                            player: &mut player::Player,
                            screen_top_left_corner: point::Point,
                            map_dimensions: point::Point,
                            rng: &mut R) {
    if !player.alive() {
        return
    }
    // NOTE: one quarter of the map area should be a decent overestimate
    let monster_count_estimate = map_dimensions.x * map_dimensions.y / 4;
    assert!(monster_count_estimate > 0);
    let mut monster_positions_to_process = VecDeque::with_capacity(monster_count_estimate as usize);
    monster_positions_to_process.extend(
        world.monster_positions(
            screen_top_left_corner - (10, 10),
            map_dimensions + (10, 10)));

    for &pos in monster_positions_to_process.iter() {
        if let Some(monster) = world.monster_on_pos(pos) {
            monster.new_turn();
        }
    }

    while let Some(mut monster_position) = monster_positions_to_process.pop_front() {
        let monster_readonly = world.monster_on_pos(monster_position).expect("Monster should exist on this position").clone();
        let action = {
            let (ai, action) = monster_readonly.act(player.pos, world, rng);
            if let Some(monster) = world.monster_on_pos(monster_position) {
                monster.ai_state = ai;
                monster.spend_ap(1);
            }
            action
        };

        match action {
            Action::Move(destination) => {
                assert_eq!(monster_position, monster_readonly.position);

                let pos = monster_readonly.position;

                let path_changed = monster_readonly.path.last()
                    .map(|&cached_destination| cached_destination != destination)
                    .unwrap_or(true);

                // NOTE: we keep a cache of any previously calculated
                // path in `monster.path`. If the precalculated path
                // is blocked or there is none, calculate a new one
                // and cache it. Otherwise, just walk it.
                let (newpos, newpath) = if monster_readonly.path.is_empty() || path_changed || !world.walkable(monster_readonly.path[0], level::Walkability::BlockingMonsters) {
                    // Calculate a new path or recalculate the existing one.
                    let mut path = pathfinding::Path::find(
                        pos, destination, world, level::Walkability::BlockingMonsters);
                    let newpos = path.next().unwrap_or(pos);
                    // Cache the path-finding result
                    let newpath = path.collect();
                    (newpos, newpath)
                } else {
                    (monster_readonly.path[0], monster_readonly.path[1..].into())
                };

                world.move_monster(pos, newpos);
                if let Some(monster) = world.monster_on_pos(newpos) {
                    monster.path = newpath;
                    if monster.has_ap(1) {
                        monster.trail = Some(newpos);
                    }
                }
                monster_position = newpos;
            }

            Action::Attack(target_pos, damage) => {
                assert!(target_pos == player.pos);
                player.take_effect(damage);
                if monster_readonly.die_after_attack {
                    kill_monster(monster_readonly.position, world);
                }
            }

            Action::Use(_) => unreachable!(),
        }

        if world.monster_on_pos(monster_position).map_or(false, |m| m.has_ap(1)) {
            monster_positions_to_process.push_back(monster_position);
        }

    }
}


fn render_panel(x: i32, width: i32, display_size: point::Point, state: &GameState,
                dt: Duration, drawcalls: &mut Vec<Draw>, fps: i32) {
    let fg = color::gui_text;
    let bg = color::dim_background;

    {
        let height = display_size.y;
        drawcalls.push(
            Draw::Rectangle(point::Point{x: x, y: 0}, point::Point{x: width, y: height}, bg));
    }

    let player = &state.player;

    let (mind_str, mind_val_percent) = match player.mind {
        player::Mind::Withdrawal(val) => ("Withdrawal", val.percent()),
        player::Mind::Sober(val) => ("Sober", val.percent()),
        player::Mind::High(val) => ("High", val.percent()),
    };

    let mut lines: Vec<Cow<'static, str>> = vec![
        mind_str.into(),
        "".into(), // NOTE: placeholder for the Mind state percentage bar
        "".into(),
        format!("Will: {}", *player.will).into(),
    ];

    if player.inventory.len() > 0 {
        lines.push("".into());
        lines.push("Inventory:".into());

        let mut item_counts = std::collections::HashMap::new();
        for item in player.inventory.iter() {
            let count = item_counts.entry(item.kind).or_insert(0);
            *count += 1;
        }

        for kind in item::Kind::iter() {
            if let Some(count) = item_counts.get(&kind) {
                lines.push(format!("[{}] {:?}: {}", inventory_key(kind), kind, count).into());
            }
        }
    }

    lines.push("".into());

    if player.will.is_max() {
        lines.push(format!("Sobriety: {}", player.sobriety_counter.percent()).into());
    }

    if state.cheating {
        lines.push("CHEATING".into());
        lines.push("".into());
    }

    if state.side == Side::Victory {
        lines.push(format!("VICTORY!").into());
    }

    if player.alive() {
        if *player.stun > 0 {
            lines.push(format!("Stunned({})", *player.stun).into());
        }
        if *player.panic > 0 {
            lines.push(format!("Panicking({})", *player.panic).into());
        }
    } else {
        lines.push("Dead".into());
    }

    if state.cheating {
        lines.push("Time stats:".into());
        for frame_stat in state.stats.last_frames(25) {
            lines.push(format!("upd: {}, dc: {}",
                               frame_stat.update.num_milliseconds(),
                               frame_stat.drawcalls.num_milliseconds()).into());
        }
        lines.push(format!("longest upd: {}",
                           state.stats.longest_update().num_milliseconds()).into());
        lines.push(format!("longest dc: {}",
                           state.stats.longest_drawcalls().num_milliseconds()).into());
    }


    for (y, line) in lines.into_iter().enumerate() {
        drawcalls.push(Draw::Text(point::Point{x: x + 1, y: y as i32}, line.into(), fg));
    }

    let max_val = match player.mind {
        player::Mind::Withdrawal(val) => val.max(),
        player::Mind::Sober(val) => val.max(),
        player::Mind::High(val) => val.max(),
    };
    let mut bar_width = width - 2;
    if max_val < bar_width {
        bar_width = max_val;
    }

    graphics::progress_bar(drawcalls, mind_val_percent, (x + 1, 1).into(), bar_width,
                           color::gui_progress_bar_fg,
                           color::gui_progress_bar_bg);

    let bottom = display_size.y - 1;

    if state.cheating {
        drawcalls.push(Draw::Text(point::Point{x: x + 1, y: bottom - 1},
                                  format!("dt: {}ms", dt.num_milliseconds()).into(), fg));
        drawcalls.push(Draw::Text(point::Point{x: x + 1, y: bottom}, format!("FPS: {}", fps).into(), fg));
    }

}

fn render_controls_help(map_size: point::Point, drawcalls: &mut Vec<Draw>) {
    fn rect_dim(lines: &[&str]) -> (i32, i32) {
        (lines.iter().map(|l| l.len() as i32).max().unwrap(), lines.len() as i32)
    }

    fn draw_rect(lines: &[&'static str], start: point::Point, w: i32, h: i32,
                 drawcalls: &mut Vec<Draw>) {
        drawcalls.push(Draw::Rectangle(start,
                                       point::Point::new(w, h),
                                       color::dim_background));
        for (index, &line) in lines.iter().enumerate() {
            drawcalls.push(Draw::Text(start + point::Point::new(0, index as i32 ),
                                      line.into(),
                                      color::gui_text));
        }
    };

    let padding = 3;

    let lines = [
        "Up",
        "Num 8",
        "or: K",
    ];
    let (width, height) = rect_dim(&lines);
    let start = point::Point {
        x: (map_size.x - width) / 2,
        y: padding,
    };
    draw_rect(&lines, start, width, height, drawcalls);

    let lines = [
        "Down",
        "Num 2",
        "or: J",
    ];
    let (width, height) = rect_dim(&lines);
    let start = point::Point {
        x: (map_size.x - width) / 2,
        y: map_size.y - height - padding,
    };
    draw_rect(&lines, start, width, height, drawcalls);

    let lines = [
        "Left",
        "Num 4",
        "or: H",
    ];
    let (width, height) = rect_dim(&lines);
    let start = point::Point {
        x: padding,
        y: (map_size.y - height) / 2,
    };
    draw_rect(&lines, start, width, height, drawcalls);

    let lines = [
        "Right",
        "Num 6",
        "or: L",
    ];
    let (width, height) = rect_dim(&lines);
    let start = point::Point {
        x: map_size.x - width - padding,
        y: (map_size.y - height) / 2,
    };
    draw_rect(&lines, start, width, height, drawcalls);

    let lines = [
        "Shift+Right",
        "Num 7",
        "or: Y",
    ];
    let (width, height) = rect_dim(&lines);
    let start = point::Point {
        x: padding,
        y: padding,
    };
    draw_rect(&lines, start, width, height, drawcalls);

    let lines = [
        "Shift+Right",
        "Num 9",
        "or: U",
    ];
    let (width, height) = rect_dim(&lines);
    let start = point::Point {
        x: map_size.x - width - padding,
        y: padding,
    };
    draw_rect(&lines, start, width, height, drawcalls);

    let lines = [
        "Ctrl+Left",
        "Num 1",
        "or: N",
    ];
    let (width, height) = rect_dim(&lines);
    let start = point::Point {
        x: padding,
        y: map_size.y - height - padding,
    };
    draw_rect(&lines, start, width, height, drawcalls);

    let lines = [
        "Ctrl+Right",
        "Num 3",
        "or: M",
    ];
    let (width, height) = rect_dim(&lines);
    let start = point::Point {
        x: map_size.x - width - padding,
        y: map_size.y - height - padding,
    };
    draw_rect(&lines, start, width, height, drawcalls);

}

fn show_exit_stats(stats: &stats::Stats) {
    println!("Slowest update durations: {:?}\n\nSlowest drawcall durations: {:?}",
             stats.longest_update_durations().iter().map(|dur| dur.num_microseconds().unwrap_or(std::i64::MAX)).map(|us| us as f32 / 1000.0).collect::<Vec<_>>(),
             stats.longest_drawcall_durations().iter().map(|dur| dur.num_microseconds().unwrap_or(std::i64::MAX)).map(|us| us as f32 / 1000.0).collect::<Vec<_>>());
    println!("\nMean update duration: {} ms\nMean drawcall duration: {} ms",
             stats.mean_update(),
             stats.mean_drawcalls());
}

fn update(mut state: GameState,
          dt: Duration,
          display_size:
          point::Point,
          fps: i32,
          new_keys: &[Key],
          mut settings: Settings,
          drawcalls: &mut Vec<Draw>)
          -> Option<(Settings, GameState)>
{
    let update_stopwatch = timer::Stopwatch::start();
    state.clock = state.clock + dt;
    state.replay_step = state.replay_step + dt;

    state.keys.extend(new_keys.iter().cloned());

    // Quit the game when Q is pressed or on replay and requested
    if state.keys.matches_code(KeyCode::Q) ||
        (!state.player.alive() && state.exit_after) ||
        (state.replay && state.exit_after && (state.commands.is_empty() || (!state.player.alive() && state.screen_fading.is_none())))
    {
        show_exit_stats(&state.stats);
        return None;
    }

    // Restart the game on F5
    if state.keys.matches_code(KeyCode::F5) {
        let state = GameState::new_game(state.world_size, state.map_size.x, state.panel_width, state.display_size, false);
        return Some((settings, state));
    }

    // Full screen on Alt-Enter
    if state.keys.matches(|k| k.alt && k.code == KeyCode::Enter) {
        settings.fullscreen = !settings.fullscreen;
    }

    // Uncover map
    if state.keys.matches_code(KeyCode::F6) {
        state.cheating = !state.cheating;
    }

    state.paused = if state.replay && state.keys.matches_code(KeyCode::Space) {
        !state.paused
    } else {
        state.paused
    };

    let paused_one_step = state.paused && state.keys.matches_code(KeyCode::Right);
    let timed_step = if state.replay && !state.paused && (state.replay_step.num_milliseconds() >= 50 || state.replay_full_speed) {
        state.replay_step = Duration::zero();
        true
    } else {
        false
    };

    // Animation to re-center the screen around the player when they
    // get too close to an edge.
    state.pos_timer.update(dt);
    if !state.pos_timer.finished() {
        let percentage = state.pos_timer.percentage_elapsed();
        let x = (((state.new_screen_pos.x - state.old_screen_pos.x) as f32) * percentage) as i32;
        let y = (((state.new_screen_pos.y - state.old_screen_pos.y) as f32) * percentage) as i32;
        state.screen_position_in_world = state.old_screen_pos + (x, y);
    }


    let player_was_alive = state.player.alive();
    let running = !state.paused && !state.replay;
    let screen_left_top_corner = state.screen_position_in_world - (state.map_size / 2);
    let mut spent_turn = false;

    // NOTE: this isn't just cosmetic. If the screen re-center
    // animation happens on replay, the world chunks are generated at
    // the wrong time, which would be except that includes the
    // monsters and that leads to a discrepancy between the actual
    // gameplay and replay.
    //
    // Until we find a better fix, we'll just have to block command
    // processing whenever any animation is playing.
    let no_animations = state.explosion_animation.is_none() && state.pos_timer.finished();


    if running || paused_one_step || timed_step && state.side != Side::Victory && no_animations {
        process_keys(&mut state.keys, &mut state.commands);

        let command_count = state.commands.len();

        // NOTE: Process player
        process_player(&mut state);

        // NOTE: Process monsters
        if state.player.ap() <= 0 && state.explosion_animation.is_none() {
            process_monsters(&mut state.world, &mut state.player, screen_left_top_corner, state.map_size, &mut state.rng);
            state.player.new_turn();
        }

        spent_turn = command_count > state.commands.len();
    }

    // Run the dose explosion effect here:
    if let Some(ref anim) = state.explosion_animation {
        for (pos, _, effect) in anim.tiles() {
            if effect.contains(animation::KILL) {
                kill_monster(pos, &mut state.world);
            }
            if effect.contains(animation::SHATTER) {
                let cell =  state.world.cell_mut(pos);
                cell.tile.kind = level::TileKind::Empty;
                cell.items.clear();
            }
        }
    }

    if spent_turn {
        state.turn += 1;
        // TODO: we can sort the chunks and compare directly at some point.
        let chunks = state.world.chunks();
        let mut monsters = vec![];
        for &chunk_pos in &chunks {
            for monster in state.world.chunk(chunk_pos).monsters().iter() {
                if !monster.dead {
                    monsters.push((monster.position, chunk_pos, monster.kind));
                }
            }
        }
        monsters.sort_by_key(|&(monster_pos, _chunk_pos, kind)| (monster_pos.x, monster_pos.y, kind));

        let actual_state_verification = game_state::Verification {
            turn: state.turn,
            chunk_count: chunks.len(),
            player_pos: state.player.pos,
            monsters: monsters,
        };
        if state.replay {
            let expected = state.verifications.pop_front().expect(
                &format!("No verification present for turn {}.", state.turn));
            assert_eq!(expected, actual_state_verification);

            if player_was_alive && !state.player.alive() {
                if !state.commands.is_empty() {
                    panic!("Game quit too early -- there are still {} commands queued up.",
                           state.commands.len());
                }
            }

        } else {
            game_state::log_verification(&mut state.command_logger, actual_state_verification);
        }
    }

    let update_duration = update_stopwatch.finish();
    let drawcall_stopwatch = timer::Stopwatch::start();

    // NOTE: re-centre the display if the player reached the end of the screen
    if state.pos_timer.finished() {
        let display_pos = state.player.pos - screen_left_top_corner;
        let dur = Duration::milliseconds(400);
        let exploration_radius = exploration_radius(state.player.mind);
        // TODO: move the screen roughly the same distance along X and Y
        if display_pos.x < exploration_radius || display_pos.x >= state.map_size.x - exploration_radius {
            // change the screen centre to that of the player
            state.pos_timer = timer::Timer::new(dur);
            state.old_screen_pos = state.screen_position_in_world;
            state.new_screen_pos = (state.player.pos.x, state.old_screen_pos.y).into();
        } else if display_pos.y < exploration_radius || display_pos.y >= state.map_size.y - exploration_radius {
            // change the screen centre to that of the player
            state.pos_timer = timer::Timer::new(dur);
            state.old_screen_pos = state.screen_position_in_world;
            state.new_screen_pos = (state.old_screen_pos.x, state.player.pos.y).into();
        } else {
            // Do nothing
        }
    }

    // Rendering & related code here:
    if state.player.alive() {
        use player::Mind::*;
        // Fade when withdrawn:
        match state.player.mind {
            Withdrawal(value) => {
                // TODO: animate the fade from the previous value?
                let fade = value.percent() * 0.6 + 0.2;
                drawcalls.push(Draw::Fade(fade , color::Color{r: 0, g: 0, b: 0}));
            }
            Sober(_) | High(_) => {
                // NOTE: Not withdrawn, don't fade
            }
        }

    } else if player_was_alive {  // NOTE: Player just died
        state.screen_fading = Some(animation::ScreenFade::new(
            color::death_animation,
            Duration::milliseconds(500),
            Duration::milliseconds(200),
            Duration::milliseconds(300)));
    } else {
        // NOTE: player is already dead (didn't die this frame)
    }

    // NOTE: render the screen fading animation on death
    if let Some(mut anim) = state.screen_fading {
        if anim.timer.finished() {
            state.screen_fading = None;
            println!("Game real time: {:?}", state.clock);
        } else {
            use animation::ScreenFadePhase;
            let fade = match anim.phase {
                ScreenFadePhase::FadeOut => anim.timer.percentage_remaining(),
                ScreenFadePhase::Wait => 0.0,
                ScreenFadePhase::FadeIn => anim.timer.percentage_elapsed(),
                ScreenFadePhase::Done => {
                    // NOTE: this should have been handled by the if statement above.
                    unreachable!();
                }
            };
            drawcalls.push(Draw::Fade(fade, anim.color));
            let prev_phase = anim.phase;
            anim.update(dt);
            let new_phase = anim.phase;
            // TODO: this is a bit hacky, but we want to uncover the screen only
            // after we've faded out:
            if (prev_phase != new_phase) && prev_phase == ScreenFadePhase::FadeOut {
                state.endgame_screen = true;
            }
            state.screen_fading = Some(anim);
        }
    }

    let mut bonus = state.player.bonus;
    // TODO: setting this as a bonus is a hack. Pass it to all renderers
    // directly instead.
    if state.endgame_screen {
        bonus = player::Bonus::UncoverMap;
    }
    if state.cheating {
        bonus = player::Bonus::UncoverMap;
    }
    let radius = exploration_radius(state.player.mind);

    let map_size = state.map_size;
    let within_map_bounds = |pos| pos >= (0, 0) && pos < map_size;
    let player_pos = state.player.pos;
    let in_fov = |pos| player_pos.distance(pos) < (radius as f32);
    let screen_coords_from_world = |pos| pos - screen_left_top_corner;

    let total_time_ms = state.clock.num_milliseconds();
    let world_size = state.world_size;

    let player_will_is_max = state.player.will.is_max();
    let player_will = *state.player.will;
    // NOTE: this is here to appease the borrow checker. If we
    // borrowed the state here as immutable, we wouln't need it.
    let show_intoxication_effect = state.player.alive() && state.player.mind.is_high();


    // Hide the keyboard movement hints if the player gets too close
    {
        let player_screen_pos = screen_coords_from_world(state.player.pos);
        let d = 15;
        if player_screen_pos.x < d || player_screen_pos.y < d ||
            map_size.x - player_screen_pos.x < d || map_size.y - player_screen_pos.y < d
        {
            state.show_keboard_movement_hints = false;
        }
    }


    // NOTE: render the cells on the map. That means the world geometry and items.
    state.world.with_cells(screen_left_top_corner, map_size, |world_pos, cell| {
        let display_pos = screen_coords_from_world(world_pos);
        if !within_map_bounds(display_pos) {
            return;
        }

        // Render the tile
        let mut rendered_tile = cell.tile;

        if show_intoxication_effect {
            // TODO: try to move this calculation of this loop and see
            // what it does to our speed.
            let pos_x: i64 = (world_pos.x + world_size.x) as i64;
            let pos_y: i64 = (world_pos.y + world_size.y) as i64;
            assert!(pos_x >= 0);
            assert!(pos_y >= 0);
            let half_cycle_ms = 700 + ((pos_x * pos_y) % 100) * 5;
            let progress_ms = total_time_ms % half_cycle_ms;
            let forwards = (total_time_ms / half_cycle_ms) % 2 == 0;
            let progress = progress_ms as f32 / half_cycle_ms as f32;
            assert!(progress >= 0.0);
            assert!(progress <= 1.0);

            rendered_tile.fg_color = if forwards {
                graphics::fade_color(color::high, color::high_to, progress)
            } else {
                graphics::fade_color(color::high_to, color::high, progress)
            };
        }

        if in_fov(world_pos) {
            graphics::draw(drawcalls, dt, display_pos, &rendered_tile);
        } else if cell.explored || bonus == player::Bonus::UncoverMap {
            graphics::draw(drawcalls, dt, display_pos, &rendered_tile);
            drawcalls.push(Draw::Background(display_pos, color::dim_background));
        } else {
            // It's not visible. Do nothing.
        }

        // Render the irresistible background of a dose
        for item in cell.items.iter() {
            if item.is_dose() && !player_will_is_max {
                let resist_radius = player_resist_radius(item.irresistible, player_will);
                for point in point::SquareArea::new(world_pos, resist_radius) {
                    if in_fov(point) {
                        let screen_coords = screen_coords_from_world(point);
                        drawcalls.push(Draw::Background(screen_coords, color::dose_background));
                    }
                }
            }
        }

        // Render the items
        if in_fov(world_pos) || cell.explored || bonus == player::Bonus::SeeMonstersAndItems || bonus == player::Bonus::UncoverMap {
            for item in cell.items.iter() {
                graphics::draw(drawcalls, dt, display_pos, item);
            }
        }
    });

    // NOTE: render the dose/food explosion animations
    if let Some(mut anim) = state.explosion_animation {
        anim.update(dt);
        if anim.finished() {
            state.explosion_animation = None;
        } else {
            drawcalls.extend(anim.tiles().map(|(world_pos, color, _)| {
                Draw::Background(screen_coords_from_world(world_pos), color)
            }));
            state.explosion_animation = Some(anim);
        }
    }

    // NOTE: render monsters
    for monster_pos in state.world.monster_positions(screen_left_top_corner, state.map_size) {
        if let Some(monster) = state.world.monster_on_pos(monster_pos) {
            let visible = monster.position.distance(state.player.pos) < (radius as f32);
            if visible || bonus == player::Bonus::UncoverMap || bonus == player::Bonus::SeeMonstersAndItems {
                use graphics::Render;
                let world_pos = monster.position;
                let display_pos = screen_coords_from_world(world_pos);
                if let Some(trail_pos) = monster.trail {
                    if state.cheating {
                        let trail_pos = screen_coords_from_world(trail_pos);
                        if within_map_bounds(trail_pos) {
                            let (glyph, color, _) = monster.render(dt);
                            // TODO: show a fading animation of the trail colour
                            let color = color::Color {r: color.r - 55, g: color.g - 55, b: color.b - 55};
                            drawcalls.push(Draw::Char(trail_pos, glyph, color));
                        }
                    }
                }

                if state.cheating {
                    for &point in &monster.path {
                        let path_pos = screen_coords_from_world(point);
                        let (_, color, _) = monster.render(dt);
                        drawcalls.push(Draw::Background(path_pos, color));
                    }
                }

                if within_map_bounds(display_pos) {
                    graphics::draw(drawcalls, dt, display_pos, monster);
                }
            }
        }
    }

    // NOTE: render the player
    {
        let world_pos = state.player.pos;
        let display_pos = screen_coords_from_world(world_pos);
        if within_map_bounds(display_pos) {
            graphics::draw(drawcalls, dt, display_pos, &state.player);
        }
    }

    render_panel(state.map_size.x, state.panel_width, display_size, &state, dt, drawcalls, fps);
    if state.show_keboard_movement_hints {
        render_controls_help(state.map_size, drawcalls);
    }

    if state.endgame_screen {
        let doses_in_inventory = state.player.inventory.iter()
            .filter(|item| item.is_dose())
            .count();

        let turns_text = format!("Turns: {}", state.turn);
        let carrying_doses_text = format!("Carrying {} doses", doses_in_inventory);
        let high_streak_text = format!("Longest High streak: {} turns", state.player.longest_high_streak);

        let longest_text = [&turns_text, &carrying_doses_text, &high_streak_text].iter()
            .map(|s| s.chars().count())
            .max()
            .unwrap() as i32;
        let lines_count = 3;

        let rect_dimensions = point::Point {
            // NOTE: 1 tile padding, which is why we have the `+ 2`.
            x: longest_text + 2,
            // NOTE: each line has an empty line below so we just have `+ 1` for the top padding.
            y: lines_count * 2 + 1,
        };
        let rect_start = point::Point {
            x: (state.display_size.x - rect_dimensions.x) / 2,
            y: 7,
        };

        fn centered_text_pos(container_width: i32, text: &str) -> i32 {
            (container_width - text.chars().count() as i32) / 2
        }

        drawcalls.push(
            Draw::Rectangle(rect_start,
                            rect_dimensions,
                            color::background));

        drawcalls.push(
            Draw::Text(rect_start + (centered_text_pos(rect_dimensions.x, &turns_text), 1),
                       turns_text.into(),
                       color::gui_text));
        drawcalls.push(
            Draw::Text(rect_start + (centered_text_pos(rect_dimensions.x, &carrying_doses_text), 3),
                       carrying_doses_text.into(),
                       color::gui_text));
        drawcalls.push(
            Draw::Text(rect_start + (centered_text_pos(rect_dimensions.x, &high_streak_text), 5),
                       high_streak_text.into(),
                       color::gui_text));
    }

    let drawcall_duration = drawcall_stopwatch.finish();
    state.stats.push(stats::FrameStats {
        update: update_duration,
        drawcalls: drawcall_duration,
    });
    Some((settings, state))
}


fn main() {
    use clap::{Arg, ArgGroup, App};

    #[cfg(feature = "libtcod")]
    fn run_libtcod(display_size: point::Point, default_background: color::Color,
                   window_title: &str, font_path: &Path,
                   state: game_state::GameState) {
        println!("Using the libtcod backend.");
        let mut engine = engine::tcod::Engine::new(display_size, default_background, window_title, &font_path);
        engine.main_loop(state, update);
    }
    #[cfg(not(feature = "libtcod"))]
    fn run_libtcod(_display_size: point::Point, _default_background: color::Color,
                   _window_title: &str, _font_path: &Path,
                   _state: game_state::GameState) {
        println!("The \"libtcod\" feature was not compiled in.");
    }

    #[cfg(feature = "piston")]
    fn run_piston(display_size: point::Point,
                  default_background: color::Color,
                  window_title: &str,
                  font_path: &Path,
                  state: game_state::GameState,
                  update: engine::UpdateFn<game_state::GameState>) {
        println!("Using the piston backend.");
        engine::piston::main_loop(display_size, default_background, window_title, &font_path,
                                  state, update);
    }
    #[cfg(not(feature = "piston"))]
    fn run_piston(_display_size: point::Point,
                  _default_background: color::Color,
                  _window_title: &str,
                  _font_path: &Path,
                  _state: game_state::GameState,
                  _update: engine::UpdateFn<game_state::GameState>) {
        println!("The \"piston\" feature was not compiled in.");
    }

    #[cfg(feature = "terminal")]
    fn run_terminal() {
        println!("Using the rustbox backend.\n  TODO: this is not implemented yet.");
    }
    #[cfg(not(feature = "terminal"))]
    fn run_terminal() {
        println!("The \"terminal\" feature was not compiled in.");
    }

    #[cfg(feature = "opengl")]
    fn run_opengl(display_size: point::Point,
                  default_background: color::Color,
                  window_title: &str,
                  state: GameState,
                  update: engine::UpdateFn<GameState>) {
        println!("Using the default backend: opengl");
        engine::glium::main_loop(display_size, default_background, window_title,
                                 state, update);
    }
    #[cfg(not(feature = "opengl"))]
    fn run_opengl(_display_size: point::Point,
                  _default_background: color::Color,
                  _window_title: &str,
                  _state: GameState,
                  _update: engine::UpdateFn<GameState>) {
        println!("The \"opengl\" feature was not compiled in.");
    }

    #[cfg(feature = "remote")]
    fn run_remote(display_size: point::Point,
                  default_background: color::Color,
                  window_title: &str,
                  state: GameState,
                  update: engine::UpdateFn<GameState>) {
        engine::remote::main_loop(display_size, default_background, window_title,
                                  state, update);
    }
    #[cfg(not(feature = "remote"))]
    fn run_remote(_display_size: point::Point,
                  _default_background: color::Color,
                  _window_title: &str,
                  _state: GameState,
                  _update: engine::UpdateFn<GameState>) {
        println!("The \"remote\" feature was not compiled in.");
    }

    // Note: at our current font, the height of 43 is the maximum value for
    // 1336x768 monitors.
    let map_size = 43;
    let panel_width = 20;
    let display_size = (map_size + panel_width, map_size).into();
    // NOTE: 2 ^ 30
    let world_size = (1_073_741_824, 1_073_741_824).into();
    let title = "Dose Response";

    // TODO: --invincible
    // TODO: --replay-file-path

    let matches = App::new(title)
        .author("Tomas Sedovic <tomas@sedovic.cz>")
        .about("Roguelike game about addiction")
        .arg(Arg::with_name("replay")
             .value_name("FILE")
             .help("Replay this file instead of starting and playing a new game")
             .takes_value(true))
        .arg(Arg::with_name("replay-full-speed")
             .help("Don't slow the replay down (useful for getting accurate measurements)")
             .long("replay-full-speed"))
        .arg(Arg::with_name("exit-after")
             .help("Exit after the game or replay has finished")
             .long("exit-after"))
        .arg(Arg::with_name("libtcod")
             .long("libtcod")
             .help("Use the libtcod rendering backend"))
        .arg(Arg::with_name("piston")
             .long("piston")
             .help("Use the Piston rendering backend"))
        .arg(Arg::with_name("opengl")
             .long("opengl")
             .help("Use the Glium (OpenGL) rendering backend"))
        .arg(Arg::with_name("terminal")
             .long("terminal")
             .help("Use the Rustbox (terminal-only) rendering backend"))
        .arg(Arg::with_name("remote")
             .long("remote")
             .help("Don't create a game window. The input and output is controled via ZeroMQ."))
        .group(ArgGroup::with_name("graphics")
               .args(&["libtcod", "piston", "opengl", "terminal", "remote"]))
        .get_matches();

    let game_state = if let Some(replay) = matches.value_of("replay") {
        let replay_path = Path::new(replay);
        GameState::replay_game(world_size, map_size, panel_width, display_size,
                               &replay_path,
                               matches.is_present("replay-full-speed"),
                               matches.is_present("exit-after"))
    } else {
        if matches.is_present("replay-full-speed") {
            panic!("The `full-replay-speed` option can only be used if the replay log is passed.");
        }
        GameState::new_game(world_size, map_size, panel_width, display_size, matches.is_present("exit-after"))
    };

    if  matches.is_present("libtcod") {
        run_libtcod(display_size, color::background, title, &Path::new(""), game_state);
    }
    else if matches.is_present("piston") {
        run_piston(display_size, color::background, title, &Path::new(""),
                   game_state, update);
    } else if matches.is_present("terminal") {
        run_terminal();
    } else if matches.is_present("remote") {
        run_remote(display_size, color::background, title, game_state, update);
    } else {
        run_opengl(display_size, color::background, title, game_state, update);
    }

}
