use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::i64;
use std::iter::FromIterator;

use rand::Rng;
use time::Duration;

use animation::{self, AreaOfEffect};
use color;
use formula;
use engine::{Draw, Settings};
use item;
use keys::{Key, KeyCode, Keys};
use level::{TileKind, Walkability};
use monster;
use pathfinding;
use player;
use point::Point;
use rect::Rectangle;
use render;
use state::{self, Command, Side, State};
use stats::{Stats, FrameStats};
use timer::{Stopwatch, Timer};
use world::{World, Chunk};


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Move(Point),
    Attack(Point, player::Modifier),
    Use(item::Kind),
}


pub fn update(mut state: State,
              dt: Duration,
              _display_size: Point,
              fps: i32,
              new_keys: &[Key],
              mut settings: Settings,
              drawcalls: &mut Vec<Draw>)
              -> Option<(Settings, State)> {
    let update_stopwatch = Stopwatch::start();
    state.clock = state.clock + dt;
    state.replay_step = state.replay_step + dt;

    state.keys.extend(new_keys.iter().cloned());

    // Quit the game when Q is pressed or on replay and requested
    if state.keys.matches_code(KeyCode::Q) ||
       (!state.player.alive() && state.exit_after) ||
       (state.replay && state.exit_after &&
        (state.commands.is_empty() ||
         (!state.player.alive() && state.screen_fading.is_none()))) {
        show_exit_stats(&state.stats);
        return None;
    }

    // Restart the game on F5
    if state.keys.matches_code(KeyCode::F5) {
        let state = State::new_game(state.world_size,
                                    state.map_size.x,
                                    state.panel_width,
                                    state.display_size,
                                    state.exit_after,
                                    &state::generate_replay_path(),
                                    state.player.invincible);
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

    let paused_one_step = state.paused &&
                          state.keys.matches_code(KeyCode::Right);
    let timed_step = if state.replay && !state.paused &&
                        (state.replay_step.num_milliseconds() >= 50 ||
                         state.replay_full_speed) {
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
        let x = (((state.new_screen_pos.x - state.old_screen_pos.x) as f32) *
                 percentage) as i32;
        let y = (((state.new_screen_pos.y - state.old_screen_pos.y) as f32) *
                 percentage) as i32;
        state.screen_position_in_world = state.old_screen_pos + (x, y);
    }


    let player_was_alive = state.player.alive();
    let running = !state.paused && !state.replay;
    let mut spent_turn = false;
    let no_animations = state.explosion_animation.is_none() &&
                        state.pos_timer.finished();
    let simulation_area = Rectangle::center(state.player.pos, state.map_size);

    if (running || paused_one_step || timed_step) &&
       state.side != Side::Victory && no_animations {
        process_keys(&mut state.keys, &mut state.commands);

        let command_count = state.commands.len();

        // NOTE: Process player
        process_player(&mut state);

        // NOTE: Process monsters
        if state.player.ap() <= 0 && state.explosion_animation.is_none() {
            process_monsters(&mut state.world,
                             &mut state.player,
                             simulation_area,
                             &mut state.rng);
            state.player.new_turn();
        }

        spent_turn = command_count > state.commands.len();
    }

    if spent_turn {
        state.turn += 1;
    }

    // NOTE: Load up new chunks if necessary
    if spent_turn {
        for pos in simulation_area.points() {
            state.world.ensure_chunk_at_pos(pos);
        }
    }

    // Run the dose explosion effect here:
    if let Some(ref anim) = state.explosion_animation {
        for (pos, _, effect) in anim.tiles() {
            if effect.contains(animation::KILL) {
                kill_monster(pos, &mut state.world);
            }
            if effect.contains(animation::SHATTER) {
                if let Some(cell) = state.world.cell_mut(pos) {
                    cell.tile.kind = TileKind::Empty;
                    cell.items.clear();
                }
            }
        }
    }

    // Log or check verifications
    if spent_turn {
        if state.replay {
            if let Some(expected) = state.verifications.pop_front() {
                let actual = state.verification();
                verify_states(expected, actual);

                if player_was_alive && !state.player.alive() {
                    if !state.commands.is_empty() {
                        panic!("Game quit too early -- there are still {} \
                                commands queued up.",
                               state.commands.len());
                    }
                }
            } else {
                // NOTE: no verifications were loaded. Probably
                // replaying a release build.
            }
        } else if cfg!(debug_assertions) {
            // We're in the debug build, log the verification
            let verification = state.verification();
            state::log_verification(&mut state.command_logger, verification);
        } else {
            // NOTE: We're in the release build, *DON'T* log the
            // verification. They take up insane amounts of disk
            // space!
        }
    }

    // Set the fadeout animation on death
    if player_was_alive && !state.player.alive() {
        // NOTE: Player just died
        state.screen_fading =
            Some(animation::ScreenFade::new(color::death_animation,
                                            Duration::milliseconds(500),
                                            Duration::milliseconds(200),
                                            Duration::milliseconds(300)));
    }

    let update_duration = update_stopwatch.finish();
    let drawcall_stopwatch = Stopwatch::start();
    let screen_left_top_corner = state.screen_position_in_world -
                                 (state.map_size / 2);
    let screen_coords_from_world = |pos| pos - screen_left_top_corner;

    // NOTE: update the dose/food explosion animations
    state.explosion_animation = state
        .explosion_animation
        .and_then(|mut animation| {
                      animation.update(dt);
                      if animation.finished() {
                          None
                      } else {
                          Some(animation)
                      }
                  });

    // NOTE: re-centre the display if the player reached the end of the screen
    if state.pos_timer.finished() {
        let display_pos = state.player.pos - screen_left_top_corner;
        let dur = Duration::milliseconds(400);
        let exploration_radius = formula::exploration_radius(state.player.mind);
        // TODO: move the screen roughly the same distance along X and Y
        if display_pos.x < exploration_radius ||
           display_pos.x >= state.map_size.x - exploration_radius {
            // change the screen centre to that of the player
            state.pos_timer = Timer::new(dur);
            state.old_screen_pos = state.screen_position_in_world;
            state.new_screen_pos = (state.player.pos.x, state.old_screen_pos.y)
                .into();
        } else if display_pos.y < exploration_radius ||
                  display_pos.y >= state.map_size.y - exploration_radius {
            // change the screen centre to that of the player
            state.pos_timer = Timer::new(dur);
            state.old_screen_pos = state.screen_position_in_world;
            state.new_screen_pos = (state.old_screen_pos.x, state.player.pos.y)
                .into();
        } else {
            // Do nothing
        }
    }

    // NOTE: process the screen fading animation on death
    if let Some(mut anim) = state.screen_fading {
        if anim.timer.finished() {
            state.screen_fading = None;
            println!("Game real time: {:?}", state.clock);
        } else {
            use animation::ScreenFadePhase;
            let prev_phase = anim.phase;
            anim.update(dt);
            let new_phase = anim.phase;
            // TODO: this is a bit hacky, but we want to uncover the screen only
            // after we've faded out:
            if (prev_phase != new_phase) &&
               prev_phase == ScreenFadePhase::FadeOut {
                state.endgame_screen = true;
            }
            state.screen_fading = Some(anim);
        }
    }

    // Hide the keyboard movement hints if the player gets too close
    {
        let player_screen_pos = screen_coords_from_world(state.player.pos);
        let d = 15;
        if player_screen_pos.x < d || player_screen_pos.y < d ||
           state.map_size.x - player_screen_pos.x < d ||
           state.map_size.y - player_screen_pos.y < d {
            state.show_keboard_movement_hints = false;
        }
    }

    render::render_game(&state, dt, fps, drawcalls);

    let drawcall_duration = drawcall_stopwatch.finish();
    state
        .stats
        .push(FrameStats {
                  update: update_duration,
                  drawcalls: drawcall_duration,
              });
    Some((settings, state))
}


fn process_monsters<R: Rng>(world: &mut World,
                            player: &mut player::Player,
                            area: Rectangle,
                            rng: &mut R) {
    if !player.alive() {
        return;
    }
    // NOTE: one quarter of the map area should be a decent overestimate
    let monster_count_estimate = area.dimensions().x * area.dimensions().y / 4;
    assert!(monster_count_estimate > 0);
    let mut monster_positions_to_process =
        VecDeque::with_capacity(monster_count_estimate as usize);
    monster_positions_to_process
        .extend(world
                    .chunks(area)
                    .flat_map(Chunk::monsters)
                    .filter(|m| m.alive() && area.contains(m.position))
                    .map(|m| m.position));

    for &pos in monster_positions_to_process.iter() {
        if let Some(monster) = world.monster_on_pos(pos) {
            monster.new_turn();
        }
    }

    while let Some(mut monster_position) =
        monster_positions_to_process.pop_front() {
        let monster_readonly = world
            .monster_on_pos(monster_position)
            .expect("Monster should exist on this position")
            .clone();
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

                let path_changed = monster_readonly
                    .path
                    .last()
                    .map(|&cached_destination| {
                             cached_destination != destination
                         })
                    .unwrap_or(true);

                // NOTE: we keep a cache of any previously calculated
                // path in `monster.path`. If the precalculated path
                // is blocked or there is none, calculate a new one
                // and cache it. Otherwise, just walk it.
                let (newpos, newpath) = if
                    monster_readonly.path.is_empty() || path_changed ||
                    !world.walkable(monster_readonly.path[0],
                                    Walkability::BlockingMonsters) {
                    // Calculate a new path or recalculate the existing one.
                    let mut path =
                        pathfinding::Path::find(pos,
                                                destination,
                                                world,
                                                Walkability::BlockingMonsters);
                    let newpos = path.next().unwrap_or(pos);
                    // Cache the path-finding result
                    let newpath = path.collect();
                    (newpos, newpath)
                } else {
                    (monster_readonly.path[0],
                     monster_readonly.path[1..].into())
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

        if world
               .monster_on_pos(monster_position)
               .map_or(false, |m| m.has_ap(1)) {
            monster_positions_to_process.push_back(monster_position);
        }

    }
}


fn process_player_action<R, W>(
    player: &mut player::Player,
    commands: &mut VecDeque<Command>,
    world: &mut World,
    explosion_animation: &mut Option<Box<AreaOfEffect>>,
    rng: &mut R,
    command_logger: &mut W)
    where R: Rng,
          W: Write
{
    if !player.alive() || !player.has_ap(1) {
        return;
    }

    if let Some(command) = commands.pop_front() {
        state::log_command(command_logger, command);
        let mut action = match command {
            Command::N => Action::Move(player.pos + (0, -1)),
            Command::S => Action::Move(player.pos + (0, 1)),
            Command::W => Action::Move(player.pos + (-1, 0)),
            Command::E => Action::Move(player.pos + (1, 0)),

            Command::NW => Action::Move(player.pos + (-1, -1)),
            Command::NE => Action::Move(player.pos + (1, -1)),
            Command::SW => Action::Move(player.pos + (-1, 1)),
            Command::SE => Action::Move(player.pos + (1, 1)),

            Command::UseFood => Action::Use(item::Kind::Food),
            Command::UseDose => Action::Use(item::Kind::Dose),
            Command::UseCardinalDose => Action::Use(item::Kind::CardinalDose),
            Command::UseDiagonalDose => Action::Use(item::Kind::DiagonalDose),
            Command::UseStrongDose => Action::Use(item::Kind::StrongDose),
        };

        if *player.stun > 0 {
            action = Action::Move(player.pos);
        } else if *player.panic > 0 {
            let new_pos =
                world.random_neighbour_position(
                    rng, player.pos, Walkability::WalkthroughMonsters);
            action = Action::Move(new_pos);

        } else if let Some((dose_pos, dose)) =
            world.nearest_dose(player.pos, 5) {
            let resist_radius =
                formula::player_resist_radius(dose.irresistible,
                                              *player.will) as
                usize;
            if player.pos.tile_distance(dose_pos) < resist_radius as i32 {
                let mut path =
                    pathfinding::Path::find(player.pos,
                                            dose_pos,
                                            world,
                                            Walkability::WalkthroughMonsters);

                let new_pos_opt = if path.len() <= resist_radius {
                    path.next()
                } else {
                    None
                };

                if let Some(new_pos) = new_pos_opt {
                    action = Action::Move(new_pos);
                } else {
                    // NOTE: no path leading to the irresistable dose
                }
            }
        }

        // NOTE: If we have doses in the inventory that we wouldn't be
        // able to pick up anymore, use them up one by one each turn:
        let carried_irresistible_dose = player
            .inventory
            .iter()
            .find(|i| {
                      i.is_dose() &&
                      formula::player_resist_radius(i.irresistible,
                                                    *player.will) >
                      0
                  })
            .map(|i| i.kind);
        if let Some(kind) = carried_irresistible_dose {
            action = Action::Use(kind);
        }

        match action {
            Action::Move(dest) => {
                let dest_walkable =
                    world.walkable(dest, Walkability::BlockingMonsters);
                let bumping_into_monster = world.monster_on_pos(dest).is_some();
                if bumping_into_monster {
                    player.spend_ap(1);
                    //println!("Player attacks {:?}", monster);
                    if let Some(kind) = world.monster_on_pos(dest).map(|m| m.kind) {
                        match kind {
                            monster::Kind::Anxiety => {
                                player.anxiety_counter += 1;
                                if player.anxiety_counter.is_max() {
                                    player.will += 1;
                                    player.anxiety_counter.set_to_min();
                                }
                            }
                            _ => {}
                        }
                        kill_monster(dest, world);
                    }

                } else if dest_walkable {
                    player.spend_ap(1);
                    player.move_to(dest);
                    while let Some(item) = world.pickup_item(dest) {
                        use item::Kind::*;
                        match item.kind {
                            Food => player.inventory.push(item),
                            Dose | StrongDose | CardinalDose | DiagonalDose => {
                                if formula::player_resist_radius(
                                    item.irresistible, *player.will) == 0 {
                                    player.inventory.push(item);
                                } else {
                                    use_dose(
                                        player, explosion_animation, item);
                                }
                            }
                        }
                    }
                } else {
                    // NOTE: we bumped into a wall, don't do anything
                }
            }

            Action::Use(item::Kind::Food) => {
                if let Some(food_idx) =
                    player
                        .inventory
                        .iter()
                        .position(|&i| i.kind == item::Kind::Food) {
                    player.spend_ap(1);
                    let food = player.inventory.remove(food_idx);
                    player.take_effect(food.modifier);
                    let food_explosion_radius = 2;
                    let animation =
                        animation::SquareExplosion::new(player.pos,
                                                        food_explosion_radius,
                                                        1,
                                                        color::explosion);
                    *explosion_animation = Some(Box::new(animation));
                }
            }

            Action::Use(item::Kind::Dose) => {
                if let Some(dose_index) =
                    player
                        .inventory
                        .iter()
                        .position(|&i| i.kind == item::Kind::Dose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::StrongDose) => {
                if let Some(dose_index) =
                    player
                        .inventory
                        .iter()
                        .position(|&i| i.kind == item::Kind::StrongDose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::CardinalDose) => {
                if let Some(dose_index) =
                    player
                        .inventory
                        .iter()
                        .position(|&i| i.kind == item::Kind::CardinalDose) {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::DiagonalDose) => {
                if let Some(dose_index) =
                    player
                        .inventory
                        .iter()
                        .position(|&i| i.kind == item::Kind::DiagonalDose) {
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

fn process_player(state: &mut State) {
    let previous_action_points = state.player.ap();

    process_player_action(&mut state.player,
                          &mut state.commands,
                          &mut state.world,
                          &mut state.explosion_animation,
                          &mut state.rng,
                          &mut state.command_logger);

    let spent_ap_this_turn = previous_action_points > state.player.ap();

    // Increase the sobriety counter if the player behaved themself.
    if spent_ap_this_turn && !state.player.mind.is_high() &&
       state.player.will.is_max() {
        state.player.sobriety_counter += 1;
    }

    // Set the longest high streak
    if spent_ap_this_turn {
        if state.player.mind.is_high() {
            state.player.current_high_streak += 1;
            if state.player.current_high_streak >
               state.player.longest_high_streak {
                state.player.longest_high_streak =
                    state.player.current_high_streak;
            }
        } else {
            state.player.current_high_streak = 0;
        }
    }

    // NOTE: The player has stayed sober long enough. Victory! \o/
    if state.player.sobriety_counter.is_max() {
        state.side = Side::Victory;
        state.endgame_screen = true;
    }

    state
        .world
        .explore(state.player.pos,
                 formula::exploration_radius(state.player.mind));
}


fn process_keys(keys: &mut Keys, commands: &mut VecDeque<Command>) {
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

            // NotEye (arrow keys plus Ctrl and Shift modifiers for
            // horizontal movement)
            Key { code: Up, .. } => commands.push_back(Command::N),
            Key { code: Down, .. } => commands.push_back(Command::S),
            Key {
                code: Left,
                ctrl: false,
                shift: true,
                ..
            } => commands.push_back(Command::NW),
            Key {
                code: Left,
                ctrl: true,
                shift: false,
                ..
            } => commands.push_back(Command::SW),
            Key { code: Left, .. } => commands.push_back(Command::W),
            Key {
                code: Right,
                ctrl: false,
                shift: true,
                ..
            } => commands.push_back(Command::NE),
            Key {
                code: Right,
                ctrl: true,
                shift: false,
                ..
            } => commands.push_back(Command::SE),
            Key { code: Right, .. } => commands.push_back(Command::E),

            // Vi keys (hjkl for cardinal and yunm for diagonal movement)
            Key { code: K, .. } => commands.push_back(Command::N),
            Key { code: J, .. } => commands.push_back(Command::S),
            Key { code: H, .. } => commands.push_back(Command::W),
            Key { code: L, .. } => commands.push_back(Command::E),
            Key { code: Y, .. } => commands.push_back(Command::NW),
            Key { code: N, .. } => commands.push_back(Command::SW),
            Key { code: U, .. } => commands.push_back(Command::NE),
            Key { code: M, .. } => commands.push_back(Command::SE),

            // Non-movement commands
            Key { code: E, .. } |
            Key { code: D1, .. } => {
                commands.push_back(Command::UseFood);
            }
            Key { code: D2, .. } => {
                commands.push_back(Command::UseDose);
            }
            Key { code: D3, .. } => {
                commands.push_back(Command::UseStrongDose);
            }
            _ => {
                match inventory_commands(key) {
                    Some(command) => commands.push_back(command),
                    None => (),
                }
            }
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
            _ => unreachable!(
                "There should only ever be 9 item kinds at most."),
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


pub fn inventory_key(kind: item::Kind) -> u8 {
    use item::Kind::*;
    match kind {
        Food => 1,
        Dose => 2,
        CardinalDose => 3,
        DiagonalDose => 4,
        StrongDose => 5,
    }
}


fn kill_monster(monster_position: Point, world: &mut World) {
    let invincible = world
        .monster_on_pos(monster_position)
        .map_or(false, |m| m.invincible);
    if invincible {
        // It's invincible: no-op
    } else {
        if let Some(monster) = world.monster_on_pos(monster_position) {
            monster.dead = true;
        }
        world.remove_monster(monster_position);
    }
}

fn use_dose(player: &mut player::Player,
            explosion_animation: &mut Option<Box<AreaOfEffect>>,
            item: item::Item) {
    use player::Modifier::*;
    use item::Kind::*;
    // TODO: do a different explosion animation for the cardinal dose
    if let Intoxication { state_of_mind, .. } = item.modifier {
        let radius = match state_of_mind <= 100 {
            true => 4,
            false => 6,
        };
        player.take_effect(item.modifier);
        let animation: Box<AreaOfEffect> = match item.kind {
            Dose | StrongDose => {
                Box::new(animation::SquareExplosion::new(player.pos,
                                                         radius,
                                                         2,
                                                         color::explosion))
            }
            CardinalDose => {
                Box::new(animation::CardinalExplosion::new(
                    player.pos,
                    radius,
                    2,
                    color::explosion,
                    color::shattering_explosion))
            }
            DiagonalDose => {
                Box::new(animation::DiagonalExplosion::new(
                    player.pos,
                    radius,
                    2,
                    color::explosion,
                    color::shattering_explosion))
            }
            Food => unreachable!(),

        };
        *explosion_animation = Some(animation);
    } else {
        unreachable!();
    }
}


fn show_exit_stats(stats: &Stats) {
    println!("Slowest update durations: {:?}\n\nSlowest drawcall \
              durations: {:?}",
             stats
                 .longest_update_durations()
                 .iter()
                 .map(|dur| dur.num_microseconds().unwrap_or(i64::MAX))
                 .map(|us| us as f32 / 1000.0)
                 .collect::<Vec<_>>(),
             stats
                 .longest_drawcall_durations()
                 .iter()
                 .map(|dur| dur.num_microseconds().unwrap_or(i64::MAX))
                 .map(|us| us as f32 / 1000.0)
                 .collect::<Vec<_>>());
    println!("\nMean update duration: {} ms\nMean drawcall duration: {} ms",
             stats.mean_update(),
             stats.mean_drawcalls());
}


fn verify_states(expected: state::Verification, actual: state::Verification) {
    if expected.chunk_count != actual.chunk_count {
        println!("Expected chunks: {}, actual: {}",
                 expected.chunk_count,
                 actual.chunk_count);
    }
    if expected.player_pos != actual.player_pos {
        println!("Expected player position: {}, actual: {}",
                 expected.player_pos,
                 actual.player_pos);
    }
    if expected.monsters.len() != actual.monsters.len() {
        println!("Expected monster count: {}, actual: {}",
                 expected.monsters.len(),
                 actual.monsters.len());
    }
    if expected.monsters != actual.monsters {
        let expected_monsters: HashMap<Point, (Point, monster::Kind)> =
            FromIterator::from_iter(expected
                                        .monsters
                                        .iter()
                                        .map(|&(pos,
                                                chunk_pos,
                                                monster)| {
                                                 (pos, (chunk_pos, monster))
                                             }));
        let actual_monsters: HashMap<Point, (Point, monster::Kind)> =
            FromIterator::from_iter(actual
                                        .monsters
                                        .iter()
                                        .map(|&(pos,
                                                chunk_pos,
                                                monster)| {
                                                 (pos, (chunk_pos, monster))
                                             }));

        for (pos, expected) in &expected_monsters {
            match actual_monsters.get(pos) {
                Some(actual) => {
                    if expected != actual {
                        println!("Monster at {} differ. Expected: {:?}, \
                                  actual: {:?}",
                                 pos,
                                 expected,
                                 actual);
                    }
                }
                None => {
                    println!("Monster expected at {}: {:?}, but it's not \
                              there.",
                             pos,
                             expected);
                }
            }
        }

        for (pos, actual) in &actual_monsters {
            if expected_monsters.get(pos).is_none() {
                println!("There is an unexpected monster at: {}: {:?}.",
                         pos,
                         actual);
            }
        }
    }
    assert!(expected == actual, "Validation failed!");
}
