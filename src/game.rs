use animation::{self, AreaOfEffect};
use blocker::Blocker;
use color;
use engine::{TILESIZE, Draw, Mouse, Settings, TextMetrics};
use formula;
use item;
use keys::{Key, KeyCode, Keys};
use level::TileKind;
use windows::{endgame, help, main_menu, sidebar};
use monster::{self, CompanionBonus};
use pathfinding;
use player;
use point::Point;
use ranged_int::{InclusiveRange, Ranged};

use rand::Rng;
use rect::Rectangle;
use render;
use state::{self, Command, Side, State, Window};
use stats::{FrameStats, Stats};
use std::collections::{HashMap, VecDeque};
use std::u64;
use std::io::Write;
use std::iter::FromIterator;
use std::time::Duration;
use timer::{Stopwatch, Timer};
use util;
use world::World;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Move(Point),
    Attack(Point, player::Modifier),
    Use(item::Kind),
}

pub enum RunningState {
    Running,
    Stopped,
    NewGame(State),
}

pub fn update(
    state: &mut State,
    dt: Duration,
    _display_size: Point,
    fps: i32,
    new_keys: &[Key],
    mouse: Mouse,
    settings: &mut Settings,
    metrics: &TextMetrics,
    drawcalls: &mut Vec<Draw>,
) -> RunningState {
    let update_stopwatch = Stopwatch::start();
    state.clock = state.clock + dt;
    state.replay_step = state.replay_step + dt;

    state.keys.extend(new_keys.iter().cloned());
    state.mouse = mouse;

    // Quit the game when Q is pressed or on replay and requested
    if (!state.player.alive() && state.exit_after)
        || (state.replay && state.exit_after
            && (state.commands.is_empty()
                || (!state.player.alive() && state.screen_fading.is_none())))
    {
        show_exit_stats(&state.stats);
        return RunningState::Stopped;
    }

    if cfg!(feature = "fullscreen") {
        // Full screen on Alt-Enter
        if state.keys.matches(|k| k.alt && k.code == KeyCode::Enter) {
            println!("Pressed Alt+Enter, toggling fullscreen.");
            settings.fullscreen = !settings.fullscreen;
        }
    }

    let current_window = state.window_stack.top();
    let game_update_result = match current_window {
        Window::MainMenu => process_main_menu(state, settings, &main_menu::Window, metrics),
        Window::Game => process_game(state, &sidebar::Window, metrics, dt),
        Window::Help => process_help_window(state, &help::Window, metrics),
        Window::Endgame => process_endgame_window(state, &endgame::Window, metrics),
        Window::Message(_) => process_message_window(state),
    };

    // NOTE: process the screen fading animation animation.
    // This must happen outside of the window-custom code because the fadeout could
    // span multiple windows.
    if let Some(mut anim) = state.screen_fading {
        if anim.timer.finished() {
            state.screen_fading = None;
        } else {
            use animation::ScreenFadePhase;
            let prev_phase = anim.phase;
            anim.update(dt);
            let new_phase = anim.phase;
            // TODO: this is a bit hacky, but we want to uncover the screen only
            // after we've faded out:
            if (prev_phase != new_phase) && prev_phase == ScreenFadePhase::FadeOut {
                if state.show_endscreen_and_uncover_map_during_fadein {
                    state.uncovered_map = true;
                    state.window_stack.push(Window::Endgame);
                }
            }
            state.screen_fading = Some(anim);
        }
    }

    // NOTE: Clear any unprocessed keys
    while let Some(_key) = state.keys.get() {}

    let update_duration = update_stopwatch.finish();

    let drawcall_stopwatch = Stopwatch::start();
    render::render(&state, dt, fps, metrics, drawcalls);
    let drawcall_duration = drawcall_stopwatch.finish();

    if cfg!(feature = "stats") {
        state.stats.push(FrameStats {
            update: update_duration,
            drawcalls: drawcall_duration,
        });
    }

    if let RunningState::Stopped = game_update_result  {
        if cfg!(feature = "stats") {
            show_exit_stats(&state.stats);
        }
    }

    game_update_result
}

fn process_game(
    state: &mut State,
    window: &sidebar::Window,
    metrics: &TextMetrics,
    dt: Duration,
) -> RunningState {
    use self::sidebar::Action;

    let mut option = None;

    if state.mouse.left {
        option = window.hovered(&state, metrics);
    }

    if option.is_none() {
        option = if state.keys.matches_code(KeyCode::Esc) {
            Some(Action::MainMenu)
        } else if state.keys.matches_code(KeyCode::QuestionMark) {
            Some(Action::Help)
        } else {
            None
        };
    }

    match option {
        Some(Action::MainMenu) => {
            state.window_stack.push(Window::MainMenu);
            return RunningState::Running;
        }
        Some(Action::Help) => {
            state.window_stack.push(Window::Help);
            return RunningState::Running;
        }
        _ => {}
    }

    // Show the endgame screen on any pressed key:
    if state.game_ended && state.screen_fading.is_none()
        && (state.keys.matches(|_| true) || state.mouse.right)
    {
        state.window_stack.push(Window::Endgame);
        return RunningState::Running;
    }

    // Uncover map / set the Cheat mode
    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::F6) {
        state.cheating = !state.cheating;
    }

    // NOTE: this will not show up in the replay so that'll be out of
    // sync. We can pass `--invincible` while running the replay
    // though and that should always work, I think.
    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::I) && state.cheating {
        println!("Making the player invincible!");
        state.player.invincible = true;
    }

    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::F) && state.cheating {
        state.player.inventory.push(formula::FOOD_PREFAB);
    }

    state.paused = if state.replay && state.keys.matches_code(KeyCode::Space) {
        !state.paused
    } else {
        state.paused
    };

    let paused_one_step = state.paused && state.keys.matches_code(KeyCode::Right);
    let timed_step = if state.replay && !state.paused
        && (util::num_milliseconds(state.replay_step) >= 50 || state.replay_full_speed)
    {
        state.replay_step = Duration::new(0, 0);
        true
    } else {
        false
    };

    // Animation to re-center the screen around the player when they
    // get too close to an edge.
    state.pos_timer.update(dt);
    if !state.pos_timer.finished() {
        let percentage = util::sine_curve(state.pos_timer.percentage_elapsed());
        let tilesize = TILESIZE as f32;
        let x = ((state.old_screen_pos.x - state.new_screen_pos.x) as f32) * percentage * tilesize;
        let y = ((state.old_screen_pos.y - state.new_screen_pos.y) as f32) * percentage * tilesize;
        state.offset_px = Point::new(x as i32, y as i32);
    } else {
        state.screen_position_in_world = state.new_screen_pos;
        state.offset_px = Point::zero();
    }

    let player_was_alive = state.player.alive();
    let running = !state.paused && !state.replay;
    let mut spent_turn = false;
    let no_animations = state.explosion_animation.is_none() && state.pos_timer.finished();
    let simulation_area = Rectangle::center(state.player.pos, state.map_size);

    if (running || paused_one_step || timed_step) && state.side != Side::Victory && no_animations {
        process_keys(&mut state.keys, &mut state.commands);
        let mouse_command = match option {
            Some(Action::UseFood) => Some(Command::UseFood),
            Some(Action::UseDose) => Some(Command::UseDose),
            Some(Action::UseCardinalDose) => Some(Command::UseCardinalDose),
            Some(Action::UseDiagonalDose) => Some(Command::UseDiagonalDose),
            Some(Action::UseStrongDose) => Some(Command::UseStrongDose),
            _ => None,
        };

        if let Some(command) = mouse_command {
            state.commands.push_front(command);
        }

        let command_count = state.commands.len();

        // NOTE: Process player
        process_player(state, simulation_area);

        // NOTE: Process monsters
        if state.player.ap() <= 0 && state.explosion_animation.is_none() {
            process_monsters(
                &mut state.world,
                &mut state.player,
                simulation_area,
                &mut state.rng,
            );
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
            if effect.contains(animation::TileEffect::KILL) {
                kill_monster(pos, &mut state.world);
            }
            if effect.contains(animation::TileEffect::SHATTER) {
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
                        panic!(
                            "Game quit too early -- there are still {} \
                             commands queued up.",
                            state.commands.len()
                        );
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
        use player::CauseOfDeath::*;
        let cause_of_death = formula::cause_of_death(&state.player);
        let fade_color = match cause_of_death {
            Some(Exhausted) => color::exhaustion_animation,
            Some(Overdosed) => color::overdose_animation,
            Some(_) => color::death_animation,
            None => {
                // NOTE: this shouldn't happen (there should always be
                // a cause of death) but if it deas, we won't crash
                color::death_animation
            }
        };
        let fade = formula::mind_fade_value(state.player.mind);
        let (fade_percentage, fade_duration) = if fade > 0.0 {
            (1.0 - fade, 2500)
        } else {
            (0.0, 500)
        };
        state.game_ended = true;
        state.show_endscreen_and_uncover_map_during_fadein = true;
        state.screen_fading = Some(animation::ScreenFade::new(
            fade_color,
            Duration::from_millis(fade_duration),
            Duration::from_millis(200),
            Duration::from_millis(300),
            fade_percentage,
        ));
        println!("Game real time: {:?}", state.clock);
    }

    let screen_left_top_corner = state.screen_position_in_world - (state.map_size / 2);
    let screen_coords_from_world = |pos| pos - screen_left_top_corner;

    // NOTE: update the dose/food explosion animations
    state.explosion_animation = state.explosion_animation.take().and_then(|mut animation| {
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
        let dur = Duration::from_millis(400);
        let exploration_radius = formula::exploration_radius(state.player.mind);
        // TODO: move the screen roughly the same distance along X and Y
        if display_pos.x < exploration_radius
            || display_pos.x >= state.map_size.x - exploration_radius
            || display_pos.y < exploration_radius
            || display_pos.y >= state.map_size.y - exploration_radius
        {
            state.pos_timer = Timer::new(dur);
            state.old_screen_pos = state.screen_position_in_world;
            // change the screen centre to that of the player
            state.new_screen_pos = state.player.pos;
        } else {
            // Do nothing
        }
    }

    // Hide the keyboard movement hints if the player gets too close
    {
        let player_screen_pos = screen_coords_from_world(state.player.pos);
        let d = 15;
        if player_screen_pos.x < d || player_screen_pos.y < d
            || state.map_size.x - player_screen_pos.x < d
            || state.map_size.y - player_screen_pos.y < d
        {
            state.show_keboard_movement_hints = false;
        }
    }

    RunningState::Running
}

fn process_main_menu(
    state: &mut State,
    settings: &mut Settings,
    window: &main_menu::Window,
    metrics: &TextMetrics,
) -> RunningState {
    use windows::main_menu::MenuItem::*;

    let mut option = None;

    if state.mouse.left {
        option = window.hovered(&state, metrics);
    }

    if option.is_none() {
        if state.keys.matches_code(KeyCode::Esc) || state.keys.matches_code(KeyCode::R) {
            option = Some(Resume);
        } else if state.mouse.right {
            option = Some(Resume);
        } else if state.keys.matches_code(KeyCode::N) {
            option = Some(NewGame);
        } else if state.keys.matches_code(KeyCode::QuestionMark)
            || state.keys.matches_code(KeyCode::H)
        {
            option = Some(Help);
        } else if state.keys.matches_code(KeyCode::F) {
            option = Some(ToggleFullscreen);
        } else if state.keys.matches_code(KeyCode::S) {
            option = Some(SaveAndQuit);
        } else if state.keys.matches_code(KeyCode::Q) {
            option = Some(Quit);
        } else if state.keys.matches_code(KeyCode::L) {
            option = Some(Load);
        }
    }

    if let Some(option) = option {
        match option {
            Resume => {
                state.window_stack.pop();
                return RunningState::Running;
            }

            NewGame => {
                // TODO: when this is the first run, this should resume the game that's already
                // loaded in the background.
                return RunningState::NewGame(create_new_game_state(state));
            }

            Help => {
                state.window_stack.push(Window::Help);
                return RunningState::Running;
            }

            ToggleFullscreen => {
                settings.fullscreen = !settings.fullscreen;
                return RunningState::Running;
            }

            SaveAndQuit => {
                if !state.game_ended {
                    match state.save_to_file() {
                        Ok(()) => return RunningState::Stopped,
                        Err(error) => {
                            // NOTE: we couldn't save the game so we'll keep going
                            println!("Error saving the game: {:?}", error);
                            state
                                .window_stack
                                .push(Window::Message("Error: could not save the game.".into()));
                        }
                    }
                }
                return RunningState::Running;
            }

            Load => match State::load_from_file() {
                Ok(new_state) => {
                    *state = new_state;
                    if state.window_stack.top() == Window::MainMenu {
                        state.window_stack.pop();
                    }
                    return RunningState::Running;
                }
                Err(error) => {
                    println!("Error loading the game: {:?}", error);
                    state
                        .window_stack
                        .push(Window::Message("Error: could not load the game.".into()));
                    return RunningState::Running;
                }
            },

            Quit => {
                return RunningState::Stopped;
            }
        }
    }

    RunningState::Running
}

fn process_help_window(
    state: &mut State,
    window: &help::Window,
    metrics: &TextMetrics,
) -> RunningState {
    use self::help::Action;

    if state.keys.matches_code(KeyCode::Esc) || state.mouse.right {
        state.window_stack.pop();
        return RunningState::Running;
    }

    let mut action = None;

    if state.mouse.left {
        action = window.hovered(&state, metrics);
    }

    if action.is_none() {
        if state.keys.matches_code(KeyCode::Right) {
            action = Some(Action::NextPage);
        } else if state.keys.matches_code(KeyCode::Left) {
            action = Some(Action::PrevPage);
        }
    }

    match action {
        Some(Action::NextPage) => {
            let new_help_window = state.current_help_window.next()
                .unwrap_or(state.current_help_window);
            state.current_help_window = new_help_window;
        }

        Some(Action::PrevPage) => {
            let new_help_window = state.current_help_window.prev()
                .unwrap_or(state.current_help_window);
            state.current_help_window = new_help_window;
        }

        None => {}
    }

    RunningState::Running
}

fn process_endgame_window(
    state: &mut State,
    window: &endgame::Window,
    metrics: &TextMetrics,
) -> RunningState {
    use windows::endgame::Action::*;

    let mut action = None;

    if state.mouse.left {
        action = window.hovered(&state, metrics);
    }

    if action.is_none() {
        if state.keys.matches_code(KeyCode::N) {
            action = Some(NewGame);
        } else if state.keys.matches_code(KeyCode::Esc) {
            action = Some(Menu);
        } else if state.keys.matches_code(KeyCode::QuestionMark) {
            action = Some(Help);
        } else if state.keys.matches_code(KeyCode::H) {
            action = Some(Help);
        }
    }

    match action {
        Some(NewGame) => RunningState::NewGame(create_new_game_state(state)),
        Some(Menu) => {
            state.window_stack.push(Window::MainMenu);
            RunningState::Running
        }
        Some(Help) => {
            state.window_stack.push(Window::Help);
            RunningState::Running
        }
        None => {
            if state.keys.get().is_some() || state.mouse.right {
                state.window_stack.pop();
            }
            RunningState::Running
        }
    }
}

fn process_message_window(state: &mut State) -> RunningState {
    if state.keys.get().is_some() || state.mouse.left || state.mouse.right {
        state.window_stack.pop();
        return RunningState::Running;
    }

    RunningState::Running
}

fn process_monsters<R: Rng>(
    world: &mut World,
    player: &mut player::Player,
    area: Rectangle,
    rng: &mut R,
) {
    if !player.alive() {
        return;
    }
    // NOTE: one quarter of the map area should be a decent overestimate
    let monster_count_estimate = area.size().x * area.size().y / 4;
    assert!(monster_count_estimate > 0);
    let mut monster_positions_vec = world.monsters(area).map(|m| m.position).collect::<Vec<_>>();
    // TODO: Sort by how far it is from the player?
    // NOTE: `world.monsters` does not give a stable result so we need to sort
    // it here to ensure correct replays.
    monster_positions_vec.sort_by_key(|pos| (pos.x, pos.y));
    let mut monster_positions_to_process: VecDeque<_> = monster_positions_vec.into();

    for &pos in monster_positions_to_process.iter() {
        if let Some(monster) = world.monster_on_pos(pos) {
            monster.new_turn();
        }
    }

    while let Some(mut monster_position) = monster_positions_to_process.pop_front() {
        let monster_readonly = world
            .monster_on_pos(monster_position)
            .expect("Monster should exist on this position")
            .clone();
        let action = {
            let (update, action) = monster_readonly.act(player.info(), world, rng);
            if let Some(monster) = world.monster_on_pos(monster_position) {
                monster.ai_state = update.ai_state;
                monster.ap = Ranged::new(
                    monster.ap.to_int(),
                    InclusiveRange(monster.ap.min(), update.max_ap),
                );

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
                    .map(|&cached_destination| cached_destination != destination)
                    .unwrap_or(true);

                // NOTE: we keep a cache of any previously calculated
                // path in `monster.path`. If the precalculated path
                // is blocked or there is none, calculate a new one
                // and cache it. Otherwise, just walk it.
                let (newpos, newpath) = if monster_readonly.path.is_empty() || path_changed
                    || !world.walkable(
                        monster_readonly.path[0],
                        monster_readonly.blockers,
                        player.pos,
                    ) {
                    // Calculate a new path or recalculate the existing one.
                    let mut path = pathfinding::Path::find(
                        pos,
                        destination,
                        world,
                        monster_readonly.blockers,
                        player.pos,
                    );
                    let newpos = path.next().unwrap_or(pos);
                    // Cache the path-finding result
                    let newpath = path.collect();
                    (newpos, newpath)
                } else {
                    (monster_readonly.path[0], monster_readonly.path[1..].into())
                };

                world.move_monster(pos, newpos, player.pos);
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
                if !player.alive() {
                    player.perpetrator = Some(monster_readonly.clone());
                    // The player's dead, no need to process other monsters
                    return;
                }
            }

            Action::Use(_) => unreachable!(),
        }

        if world
            .monster_on_pos(monster_position)
            .map_or(false, |m| m.has_ap(1))
        {
            monster_positions_to_process.push_back(monster_position);
        }
    }
}

fn process_player_action<R, W>(
    player: &mut player::Player,
    commands: &mut VecDeque<Command>,
    world: &mut World,
    simulation_area: Rectangle,
    explosion_animation: &mut Option<Box<AreaOfEffect>>,
    rng: &mut R,
    command_logger: &mut W,
) where
    R: Rng,
    W: Write,
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

        if player.stun.to_int() > 0 {
            action = Action::Move(player.pos);
        } else if player.panic.to_int() > 0 {
            let new_pos =
                world.random_neighbour_position(rng, player.pos, Blocker::WALL, player.pos);
            action = Action::Move(new_pos);
        } else if let Some((dose_pos, dose)) = world.nearest_dose(player.pos, 5) {
            let resist_radius =
                formula::player_resist_radius(dose.irresistible, player.will.to_int()) as usize;
            if player.pos.tile_distance(dose_pos) < resist_radius as i32 {
                let mut path =
                    pathfinding::Path::find(player.pos, dose_pos, world, Blocker::WALL, player.pos);

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
                i.is_dose()
                    && formula::player_resist_radius(i.irresistible, player.will.to_int()) > 0
            })
            .map(|i| i.kind);
        if let Some(kind) = carried_irresistible_dose {
            action = Action::Use(kind);
        }

        match action {
            Action::Move(dest) => {
                let dest_walkable =
                    world.walkable(dest, Blocker::WALL | Blocker::MONSTER, player.pos);
                let bumping_into_monster = world.monster_on_pos(dest).is_some();
                if bumping_into_monster {
                    player.spend_ap(1);
                    // println!("Player attacks {:?}", monster);
                    if let Some(kind) = world.monster_on_pos(dest).map(|m| m.kind) {
                        match kind {
                            monster::Kind::Anxiety => {
                                println!(
                                    "Bumped into anxiety! Current anxiety counter: {:?}",
                                    player.anxiety_counter
                                );
                                let increment =
                                    if player.bonuses.contains(&CompanionBonus::DoubleWillGrowth) {
                                        2
                                    } else {
                                        1
                                    };
                                println!("Anxiety increment: {:?}", increment);
                                player.anxiety_counter += increment;
                                println!("New anxiety counter: {:?}", player.anxiety_counter);
                                if player.anxiety_counter.is_max() {
                                    println!("Increasing player's will");
                                    player.will += 1;
                                    player.anxiety_counter.set_to_min();
                                }
                            }
                            // NOTE: NPCs don't give bonuses or accompany the player when high.
                            monster::Kind::Npc if player.mind.is_sober() => {
                                println!("Bumped into NPC: {:?}", world.monster_on_pos(dest));
                                // Clear any existing monsters accompanying the player. The player
                                // can have only one companion at a time right now.
                                //
                                // TODO: it also sounds like we could just track the followers in
                                // the Player/State struct but that needs Monster IDs.
                                let npcs = world
                                    .monsters_mut(simulation_area)
                                    .filter(|m| m.kind == monster::Kind::Npc);
                                for npc in npcs {
                                    if npc.position == dest {
                                        println!("NPC {:?} accompanies the player.", npc);
                                        npc.accompanying_player = true;
                                        assert!(npc.companion_bonus.is_some());
                                    } else if npc.accompanying_player {
                                        println!("NPC {:?} leaves the player.", npc);
                                        npc.accompanying_player = false;
                                    }
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
                                let resist_radius = formula::player_resist_radius(
                                    item.irresistible,
                                    player.will.to_int(),
                                );
                                if resist_radius == 0 {
                                    player.inventory.push(item);
                                } else {
                                    use_dose(player, explosion_animation, item);
                                }
                            }
                        }
                    }
                } else {
                    // NOTE: we bumped into a wall, don't do anything
                }
            }

            Action::Use(item::Kind::Food) => {
                if let Some(food_idx) = player
                    .inventory
                    .iter()
                    .position(|&i| i.kind == item::Kind::Food)
                {
                    player.spend_ap(1);
                    let food = player.inventory.remove(food_idx);
                    player.take_effect(food.modifier);
                    let food_explosion_radius = 2;
                    let animation = animation::SquareExplosion::new(
                        player.pos,
                        food_explosion_radius,
                        1,
                        color::explosion,
                    );
                    *explosion_animation = Some(Box::new(animation));
                }
            }

            Action::Use(item::Kind::Dose) => {
                if let Some(dose_index) = player
                    .inventory
                    .iter()
                    .position(|&i| i.kind == item::Kind::Dose)
                {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::StrongDose) => {
                if let Some(dose_index) = player
                    .inventory
                    .iter()
                    .position(|&i| i.kind == item::Kind::StrongDose)
                {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::CardinalDose) => {
                if let Some(dose_index) = player
                    .inventory
                    .iter()
                    .position(|&i| i.kind == item::Kind::CardinalDose)
                {
                    player.spend_ap(1);
                    let dose = player.inventory.remove(dose_index);
                    use_dose(player, explosion_animation, dose);
                }
            }

            Action::Use(item::Kind::DiagonalDose) => {
                if let Some(dose_index) = player
                    .inventory
                    .iter()
                    .position(|&i| i.kind == item::Kind::DiagonalDose)
                {
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

fn process_player(state: &mut State, simulation_area: Rectangle) {
    {
        // appease borrowck
        let player = &mut state.player;

        // NPCs should unfollow an intoxicated player:
        if player.mind.is_high() {
            let npcs = state.world.monsters_mut(simulation_area).filter(|m| {
                m.kind == monster::Kind::Npc && m.accompanying_player && m.companion_bonus.is_some()
            });
            for npc in npcs {
                println!("{:?} will not accompany an intoxicated player.", npc);
                npc.accompanying_player = false;
            }
        }

        let world = &state.world;

        // TODO: this will stop the bonus from working once the
        // companion NPC leaves the simulation_area. Which is
        // currently possible because it doesn't follow the player
        // around.
        let npc_bonuses = world
            .monsters(simulation_area)
            .filter(|m| {
                m.kind == monster::Kind::Npc && m.accompanying_player && m.companion_bonus.is_some()
            })
            .map(|m| m.companion_bonus.unwrap());
        player.bonuses.clear();
        player.bonuses.extend(npc_bonuses);
    }

    let previous_action_points = state.player.ap();

    process_player_action(
        &mut state.player,
        &mut state.commands,
        &mut state.world,
        simulation_area,
        &mut state.explosion_animation,
        &mut state.rng,
        &mut state.command_logger,
    );

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
        state.game_ended = true;
        state.uncovered_map = true;
        state.window_stack.push(Window::Endgame);
    }

    state.world.explore(
        state.player.pos,
        formula::exploration_radius(state.player.mind),
    );
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

            // Vi keys (hjkl for cardinal and yubn for diagonal movement)
            Key { code: K, .. } => commands.push_back(Command::N),
            Key { code: J, .. } => commands.push_back(Command::S),
            Key { code: H, .. } => commands.push_back(Command::W),
            Key { code: L, .. } => commands.push_back(Command::E),
            Key { code: Y, .. } => commands.push_back(Command::NW),
            Key { code: B, .. } => commands.push_back(Command::SW),
            Key { code: U, .. } => commands.push_back(Command::NE),
            Key { code: N, .. } => commands.push_back(Command::SE),

            // Non-movement commands
            Key { code: E, .. } => {
                commands.push_back(Command::UseFood);
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
            _ => unreachable!("There should only ever be 9 item kinds at most."),
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

fn use_dose(
    player: &mut player::Player,
    explosion_animation: &mut Option<Box<AreaOfEffect>>,
    item: item::Item,
) {
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
            Dose | StrongDose => Box::new(animation::SquareExplosion::new(
                player.pos,
                radius,
                2,
                color::explosion,
            )),
            CardinalDose => Box::new(animation::CardinalExplosion::new(
                player.pos,
                radius,
                2,
                color::explosion,
                color::shattering_explosion,
            )),
            DiagonalDose => Box::new(animation::DiagonalExplosion::new(
                player.pos,
                radius,
                2,
                color::explosion,
                color::shattering_explosion,
            )),
            Food => unreachable!(),
        };
        *explosion_animation = Some(animation);
    } else {
        unreachable!();
    }
}

fn show_exit_stats(stats: &Stats) {
    println!(
        "Slowest update durations: {:?}\n\nSlowest drawcall \
         durations: {:?}",
        stats
            .longest_update_durations()
            .iter()
            .map(|dur| util::num_microseconds(*dur).unwrap_or(u64::MAX))
            .map(|us| us as f32 / 1000.0)
            .collect::<Vec<_>>(),
        stats
            .longest_drawcall_durations()
            .iter()
            .map(|dur| util::num_microseconds(*dur).unwrap_or(u64::MAX))
            .map(|us| us as f32 / 1000.0)
            .collect::<Vec<_>>()
    );
    println!(
        "\nMean update duration: {} ms\nMean drawcall duration: {} ms",
        stats.mean_update(),
        stats.mean_drawcalls()
    );
}

fn verify_states(expected: state::Verification, actual: state::Verification) {
    if expected.chunk_count != actual.chunk_count {
        println!(
            "Expected chunks: {}, actual: {}",
            expected.chunk_count, actual.chunk_count
        );
    }
    if expected.player_pos != actual.player_pos {
        println!(
            "Expected player position: {}, actual: {}",
            expected.player_pos, actual.player_pos
        );
    }
    if expected.monsters.len() != actual.monsters.len() {
        println!(
            "Expected monster count: {}, actual: {}",
            expected.monsters.len(),
            actual.monsters.len()
        );
    }
    if expected.monsters != actual.monsters {
        let expected_monsters: HashMap<Point, (Point, monster::Kind)> = FromIterator::from_iter(
            expected
                .monsters
                .iter()
                .map(|&(pos, chunk_pos, monster)| (pos, (chunk_pos, monster))),
        );
        let actual_monsters: HashMap<Point, (Point, monster::Kind)> = FromIterator::from_iter(
            actual
                .monsters
                .iter()
                .map(|&(pos, chunk_pos, monster)| (pos, (chunk_pos, monster))),
        );

        for (pos, expected) in &expected_monsters {
            match actual_monsters.get(pos) {
                Some(actual) => {
                    if expected != actual {
                        println!(
                            "Monster at {} differ. Expected: {:?}, \
                             actual: {:?}",
                            pos, expected, actual
                        );
                    }
                }
                None => {
                    println!(
                        "Monster expected at {}: {:?}, but it's not \
                         there.",
                        pos, expected
                    );
                }
            }
        }

        for (pos, actual) in &actual_monsters {
            if expected_monsters.get(pos).is_none() {
                println!("There is an unexpected monster at: {}: {:?}.", pos, actual);
            }
        }
    }
    assert!(expected == actual, "Validation failed!");
}

fn create_new_game_state(state: &State) -> State {
    State::new_game(
        state.world_size,
        state.map_size.x,
        state.panel_width,
        state.display_size,
        state.exit_after,
        state::generate_replay_path(),
        state.player.invincible,
    )
}
