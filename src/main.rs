#![deny(overflowing_literals)]

extern crate clap;
extern crate rand;
extern crate time;

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


use std::borrow::Cow;
use std::collections::VecDeque;
use std::cmp;
use std::io::Write;
use std::path::Path;

use rand::Rng;
use time::Duration;

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
            _ => (),
        }
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

fn use_dose(player: &mut player::Player, world: &mut world::World,
            explosion_animation: &mut Option<animation::Explosion>,
            item: item::Item) {
    use player::Modifier::*;
    if let Intoxication{state_of_mind, ..} = item.modifier {
        let radius = match state_of_mind <= 100 {
            true => 4,
            false => 6,
        };
        player.take_effect(item.modifier);
        *explosion_animation = Some(explode(player.pos, radius, world));
    } else {
        unreachable!();
    }
}

fn explode(center: point::Point,
           radius: i32,
           world: &mut world::World) -> animation::Explosion {
    for pos in point::SquareArea::new(center, radius) {
        kill_monster(pos, world);
    }
    animation::Explosion::new(center, radius, 2, color::explosion)
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
    cmp::max(dose_irresistible_value - will, 0)
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

    // NOTE: The player has stayed sober long enough. Victory! \o/
    if state.player.sobriety_counter.is_max() {
        state.side = Side::Victory;
    }

    state.world.explore(state.player.pos, exploration_radius(state.player.mind));
}

fn process_player_action<R, W>(player: &mut player::Player,
                               commands: &mut VecDeque<Command>,
                               world: &mut world::World,
                               explosion_animation: &mut Option<animation::Explosion>,
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
            if player.pos.tile_distance(dose_pos) <= resist_radius as i32 {
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

        // NOTE: If we picked up doses on max Will and then lost it,
        // take them all turn by turn undonditionally:
        if !player.will.is_max() {
            if player.inventory.iter().position(|&i| i.kind == item::Kind::StrongDose).is_some() {
                action = Action::Use(item::Kind::StrongDose);
            } else if player.inventory.iter().position(|&i| i.kind == item::Kind::Dose).is_some() {
                action = Action::Use(item::Kind::Dose);
            }
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
                                        Dose | StrongDose => {
                                            if player.will.is_max() {
                                                player.inventory.push(item);
                                            } else {
                                                use_dose(player, world, explosion_animation, item);
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
                    *explosion_animation = Some(explode(player.pos, food_explosion_radius, world));
                }
            }

            Action::Use(item::Kind::Dose) => {
                if let Some(dose_index) = player.inventory.iter().position(|&i| i.kind == item::Kind::Dose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, world, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::StrongDose) => {
                if let Some(dose_index) = player.inventory.iter().position(|&i| i.kind == item::Kind::StrongDose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, world, explosion_animation, dose);
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
                // NOTE: the pathfinding has already happened so this should always be a neighbouring tile
                assert!(pos.tile_distance(destination) <= 1);
                world.move_monster(pos, destination);
                monster_position = destination;
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
        lines.push("Inventory:".into());
        let food_amount = player.inventory.iter().filter(|i| i.kind == item::Kind::Food).count();
        if food_amount > 0 {
            lines.push(format!("[1] Food: {}", food_amount).into());
        }

        let dose_amount = player.inventory.iter().filter(|i| i.kind == item::Kind::Dose).count();
        if dose_amount > 0 {
            lines.push(format!("[2] Dose: {}", dose_amount).into());
        }

        let strong_dose_amount = player.inventory.iter().filter(|i| i.kind == item::Kind::StrongDose).count();
        if strong_dose_amount > 0 {
            lines.push(format!("[3] Strong Dose: {}", strong_dose_amount).into());
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
                           color::gui_progress_bar_fg, color::gui_progress_bar_bg);

    let bottom = display_size.y - 1;
    drawcalls.push(Draw::Text(point::Point{x: x + 1, y: bottom - 1},
                              format!("dt: {}ms", dt.num_milliseconds()).into(), fg));
    drawcalls.push(Draw::Text(point::Point{x: x + 1, y: bottom}, format!("FPS: {}", fps).into(), fg));

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
    state.clock = state.clock + dt;

    state.keys.extend(new_keys.iter().cloned());

    // Quit the game when Q is pressed
    if state.keys.matches_code(KeyCode::Q) {
        return None;
    }

    // Restart the game on F5
    if state.keys.matches_code(KeyCode::F5) {
        let state = GameState::new_game(state.world_size, state.map_size.x, state.panel_width, state.display_size);
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
    let timed_step = if state.replay && !state.paused && state.clock.num_milliseconds() >= 50 {
        //state.clock = Duration::zero();
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

    if running || paused_one_step || timed_step && state.side != Side::Victory{
        process_keys(&mut state.keys, &mut state.commands);

        // NOTE: Process player
        process_player(&mut state);

        // NOTE: Process monsters
        if state.player.ap() <= 0 {
            process_monsters(&mut state.world, &mut state.player, screen_left_top_corner, state.map_size, &mut state.rng);
            state.player.new_turn();
        }
    }

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
                state.see_entire_screen = true;
            }
            state.screen_fading = Some(anim);
        }
    }

    let mut bonus = state.player.bonus;
    // TODO: setting this as a bonus is a hack. Pass it to all renderers
    // directly instead.
    if state.see_entire_screen {
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
        if anim.timer.finished() {
            state.explosion_animation = None;
        } else {
            for world_pos in point::SquareArea::new(anim.center, anim.current_radius) {
                if state.world.within_bounds(world_pos) {
                    let display_pos = screen_coords_from_world(world_pos);
                        drawcalls.push(Draw::Background(display_pos, anim.color));
                }
            }
            state.explosion_animation = Some(anim);
        }
    }

    // NOTE: render monsters
    for monster_pos in state.world.monster_positions(screen_left_top_corner, state.map_size) {
        if let Some(monster) = state.world.monster_on_pos(monster_pos) {
            let visible = monster.position.distance(state.player.pos) < (radius as f32);
            if visible || bonus == player::Bonus::UncoverMap || bonus == player::Bonus::SeeMonstersAndItems {
                let world_pos = monster.position;
                let display_pos = screen_coords_from_world(world_pos);
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
                   _window_title: &str, _font_path: &Path) {
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
                  font_path: &Path,
                  state: GameState,
                  update: engine::UpdateFn<GameState>) {
        println!("Using the default backend: opengl");
        engine::glium::main_loop(display_size, default_background, window_title, &font_path,
                                 state, update);
    }
    #[cfg(not(feature = "opengl"))]
    fn run_opengl(_display_size: point::Point,
                  _default_background: color::Color,
                  _window_title: &str,
                  _font_path: &Path,
                  _state: GameState,
                  _update: engine::UpdateFn<GameState>) {
        println!("The \"opengl\" feature was not compiled in.");
    }


    // Note: at our current font, the height of 43 is the maximum value for
    // 1336x768 monitors.
    let map_size = 43;
    let panel_width = 20;
    let display_size = (map_size + panel_width, map_size).into();
    // NOTE: 2 ^ 30
    let world_size = (1_073_741_824, 1_073_741_824).into();
    let title = "Dose Response";
    let font_dir = Path::new("fonts");
    let font_path = font_dir.join("dejavu16x16_gs_tc.png");

    let matches = App::new(title)
        .author("Tomas Sedovic <tomas@sedovic.cz>")
        .about("Roguelike game about addiction")
        .arg(Arg::with_name("replay")
             .value_name("FILE")
             .help("Replay this file instead of starting and playing a new game")
             .takes_value(true))
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
        .group(ArgGroup::with_name("graphics")
               .args(&["libtcod", "piston", "opengl", "terminal"]))
        .get_matches();

    let game_state = if let Some(replay) = matches.value_of("replay") {
        let replay_path = Path::new(replay);
        GameState::replay_game(world_size, map_size, panel_width, display_size, &replay_path)
    } else {
        GameState::new_game(world_size, map_size, panel_width, display_size)
    };

    if  matches.is_present("libtcod") {
        run_libtcod(display_size, color::background, title, &font_path, game_state);
    }
    else if matches.is_present("piston") {
        run_piston(display_size, color::background, title, &font_path,
                   game_state, update);
    } else if matches.is_present("terminal") {
        run_terminal();
    } else {
        run_opengl(display_size, color::background, title, &font_path,
                   game_state, update);
    }

}
