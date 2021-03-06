use crate::{
    ai,
    animation::{self, AreaOfEffect},
    audio::{Audio, Effect},
    blocker::Blocker,
    color,
    engine::{Display, Mouse, TextMetrics},
    formula,
    graphic::Graphic,
    item,
    keys::{Key, KeyCode, Keys},
    level::TileKind,
    monster::{self, CompanionBonus},
    palette::Palette,
    pathfinding,
    player::{self, Modifier},
    point::{self, Point},
    random::Random,
    ranged_int::{InclusiveRange, Ranged},
    rect::Rectangle,
    render,
    settings::{Settings, Store as SettingsStore},
    state::{self, Challenge, Command, Side, State},
    stats::{FrameStats, Stats},
    timer::{Stopwatch, Timer},
    ui, util,
    window::{self, Window},
    windows::{endgame, help, main_menu, message, settings, sidebar},
    world::World,
};

use std::{
    collections::{HashMap, VecDeque},
    io::Write,
    time::Duration,
};

use egui::{CtxRef, Ui};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Move(Point),
    Attack(Point, player::Modifier),
    Use(item::Kind),
}

pub enum RunningState {
    Running,
    Stopped,
    NewGame(Box<State>),
}

pub fn update(
    state: &mut State,
    egui_ctx: &CtxRef,
    dt: Duration,
    fps: i32,
    new_keys: &[Key],
    mouse: Mouse,
    settings: &mut Settings,
    metrics: &dyn TextMetrics,
    settings_store: &mut dyn SettingsStore,
    display: &mut Display, // TODO: remove this from the engine and keep a transient state instead
    audio: &mut Audio,
) -> RunningState {
    let update_stopwatch = Stopwatch::start();
    state.clock += dt;
    state.replay_step += dt;

    // TODO: only check this every say 10 or 100 frames?
    // We just wanna make sure there are items in the queue.
    if audio.background_sound_queue.len() <= 1 {
        let sound = audio.backgrounds.random(&mut audio.rng);
        match rodio::Decoder::new(sound) {
            Ok(sound) => {
                use rodio::Source;
                use std::convert::TryInto;
                let delay = if audio.background_sound_queue.empty() {
                    Duration::from_secs(0)
                } else {
                    let secs: u64 = audio.rng.range_inclusive(1, 5).try_into().unwrap_or(1);
                    Duration::from_secs(secs)
                };
                audio.background_sound_queue.append(sound.delay(delay));
            }
            Err(error) => {
                log::error!("Error decoding sound: {}. Skipping playback.", error);
            }
        }
    }

    audio.set_background_volume(settings.background_volume);
    audio.set_effects_volume(settings.sound_volume);

    // TODO: remove `state.map_size` if we're always recalculating it
    // here anyway?? The map size now depends solely on the window
    // size in pixels anyway so there's little sense carrying it
    // around.
    let panel_width_tiles = (formula::sidebar_width_px(settings.text_size) as f32
        / settings.tile_size as f32)
        .ceil() as i32;
    if display.size_without_padding() != (state.map_size.x + panel_width_tiles, state.map_size.y) {
        state.map_size = display.size_without_padding() - Point::new(panel_width_tiles, 0);
    }
    assert_eq!(
        display.size_without_padding(),
        (state.map_size.x + panel_width_tiles, state.map_size.y)
    );

    state.keys.extend(new_keys.iter().cloned());
    state.mouse = mouse;

    // Quit the game when Q is pressed or on replay and requested
    if (!state.player.alive() && state.exit_after)
        || (state.replay
            && state.exit_after
            && (state.commands.is_empty()
                || (!state.player.alive() && state.screen_fading.is_none())))
    {
        show_exit_stats(&state.stats);
        return RunningState::Stopped;
    }

    // Full screen on Alt-Enter
    if cfg!(feature = "fullscreen") && state.keys.matches(|k| k.alt && k.code == KeyCode::Enter) {
        log::info!("Pressed Alt+Enter, toggling fullscreen.");
        settings.fullscreen = !settings.fullscreen;
    }

    // Hide the timed message box if it ran out
    let mut window_timed_out = false;
    if let Window::Message { ref mut ttl, .. } = state.window_stack.top_mut() {
        if let Some(ref mut ttl) = ttl {
            *ttl = util::duration_sub_or_zero(*ttl, dt);
        }
        window_timed_out = ttl.map(|ttl| ttl.as_millis() == 0).unwrap_or(false);
    }
    if window_timed_out {
        state.window_stack.pop();
    }

    // NOTE: Clear the whole screen
    display.clear(state.palette.unexplored_background);

    // // NOTE: This renders the game's icon. Change the tilesize to an
    // // appropriate value.
    // //
    // let origin = Point::new(0, 0);
    // display.set_glyph(origin, 'D', color::depression);
    // display.set_glyph(origin + (1, 0), 'r', color::anxiety);
    // display.set_glyph(origin + (0, 1), '@', color::player);
    // display.set_glyph(origin + (1, 1), 'i', color::dose);
    // display.set_fade(color::BLACK, 1.0);

    // TODO: This might be inefficient for windows fully covering
    // other windows.
    let mut game_update_result = RunningState::Running;

    let screen_rect = egui::Rect::from_min_max(egui::Pos2::ZERO, display.screen_size_px.into());
    ui::egui_root(egui_ctx, screen_rect, |ui| {
        // NOTE: cloning the window list here to let us iterate it and mutate state.
        let windows = state.window_stack.clone();
        for window in windows.windows() {
            let top_level = state.window_stack.top() == *window;
            match window {
                Window::MainMenu => {
                    let active = top_level;
                    game_update_result =
                        main_menu::process(state, ui, settings, metrics, display, audio, active);

                    // Clear any fade set by the gameplay rendering
                    display.fade = color::INVISIBLE;
                }
                Window::Game => {
                    let mut highlighted_tiles = Vec::with_capacity(15);
                    let result = process_game(
                        state,
                        ui,
                        settings,
                        metrics,
                        display,
                        audio,
                        dt,
                        fps,
                        top_level,
                        &mut highlighted_tiles,
                    );
                    render::render_game(state, metrics, display, highlighted_tiles);
                    game_update_result = result;
                }
                Window::Settings => {
                    if top_level {
                        game_update_result =
                            settings::process(state, ui, settings, display, audio, settings_store);
                    }
                    // Clear any fade set by the gameplay rendering
                    display.fade = color::INVISIBLE;
                }
                Window::Help => {
                    if top_level {
                        game_update_result = help::process(state, ui, display, audio);
                    }
                    // Clear any fade set by the gameplay rendering
                    display.fade = color::INVISIBLE;
                }
                Window::Endgame => {
                    game_update_result =
                        endgame::process(state, ui, settings, metrics, display, audio, top_level);

                    if cfg!(feature = "recording") {
                        let window = crate::windows::call_to_action::Window;
                        window.render(state, metrics, display);
                    }
                    display.fade = color::INVISIBLE;
                }
                Window::Message {
                    ref title,
                    ref message,
                    ..
                } => {
                    if top_level {
                        game_update_result = message::process(state, ui, title, message, display)
                    }
                }
            }
        }
    });

    // NOTE: process the screen fading animation animation.
    // This must happen outside of the window-custom code because the fadeout could
    // span multiple windows.
    if let Some(mut anim) = state.screen_fading {
        if anim.timer.finished() {
            state.screen_fading = None;
        } else {
            use crate::animation::ScreenFadePhase;
            let prev_phase = anim.phase;
            anim.update(dt);
            let new_phase = anim.phase;
            // TODO: this is a bit hacky, but we want to uncover the screen only
            // after we've faded out:
            if (prev_phase != new_phase)
                && prev_phase == ScreenFadePhase::FadeOut
                && state.show_endscreen_and_uncover_map_during_fadein
            {
                state.uncovered_map = true;
                state.window_stack.push(Window::Endgame);
            }
            state.screen_fading = Some(anim);
        }
    }

    audio.play_mixed_sound_effects();

    // NOTE: Clear any unprocessed keys
    while let Some(_key) = state.keys.get() {}

    let update_duration = update_stopwatch.finish();

    // TODO: we can no longer sensibly distinguish between building the drawcalls
    // due to egui and having to bundle the update and rendering code.
    let drawcall_stopwatch = Stopwatch::start();
    let drawcall_duration = drawcall_stopwatch.finish();

    if cfg!(feature = "stats") {
        state.stats.push(FrameStats {
            update: update_duration,
            drawcalls: drawcall_duration,
        });
        if state.clock > Duration::new(5, 0) {
            state.stats.update_fps(fps);
        }
    }

    if let RunningState::Stopped = game_update_result {
        if cfg!(feature = "stats") {
            show_exit_stats(&state.stats);
        }
    }

    game_update_result
}

fn process_game(
    state: &mut State,
    ui: &mut Ui,
    settings: &Settings,
    _metrics: &dyn TextMetrics,
    display: &Display,
    audio: &mut Audio,
    dt: Duration,
    fps: i32,
    active: bool,
    highlighted_tiles: &mut Vec<Point>,
) -> RunningState {
    use self::sidebar::Action;

    let (mut option, highlighted_tile) =
        sidebar::process(state, ui, settings, dt, fps, display, active);
    if let Some(pos) = highlighted_tile {
        highlighted_tiles.push(pos);
    }

    if !active {
        return RunningState::Running;
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

    if let Some(Action::MainMenu)
    | Some(Action::Help)
    | Some(Action::UseFood)
    | Some(Action::UseDose)
    | Some(Action::UseCardinalDose)
    | Some(Action::UseDiagonalDose)
    | Some(Action::UseStrongDose) = option
    {
        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
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
    if state.game_ended
        && state.screen_fading.is_none()
        && (state.keys.matches(|_| true) || state.mouse.right_clicked)
    {
        state.window_stack.push(Window::Endgame);
        return RunningState::Running;
    }

    let player_was_alive = state.player.alive();

    // Uncover map / set the Cheat mode
    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::F6) {
        state.cheating = !state.cheating;
    }

    // NOTE: this will not show up in the replay so that'll be out of
    // sync. We can pass `--invincible` while running the replay
    // though and that should always work, I think.
    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::I) && state.cheating {
        log::info!("Making the player invincible, you cheat!");
        state.player.invincible = true;
    }

    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::F) && state.cheating {
        log::info!("Adding one Food, you cheat!");
        state.player.inventory.push(formula::FOOD_PREFAB);
    }

    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::W) && state.cheating {
        log::info!("Increasing Will by one, you cheat!");
        state.player.will += 1;
    }

    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::D) && state.cheating {
        log::info!("Killing the player");
        state.player.dead = true;
    }

    if cfg!(feature = "cheating") && state.keys.matches_code(KeyCode::V) && state.cheating {
        let vnpc_pos = place_victory_npc(state);

        // NOTE: Scroll to the Victory NPC position
        {
            state.pos_timer = Timer::new(Duration::from_millis(3000));
            state.old_screen_pos = state.screen_position_in_world;
            state.new_screen_pos = vnpc_pos;
        }
    }

    state.paused = if state.replay && state.keys.matches_code(KeyCode::Space) {
        !state.paused
    } else {
        state.paused
    };

    let paused_one_step = state.paused && state.keys.matches_code(KeyCode::Right);
    let timed_step = if state.replay
        && !state.paused
        && (state.replay_step.as_millis() >= 50 || state.replay_full_speed)
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
        let tilesize = settings.tile_size as f32;
        let percentage = util::sine_curve(state.pos_timer.percentage_elapsed());
        let x = ((state.old_screen_pos.x - state.new_screen_pos.x) as f32) * percentage * tilesize;
        let y = ((state.old_screen_pos.y - state.new_screen_pos.y) as f32) * percentage * tilesize;
        state.offset_px = Point::new(x as i32, y as i32);
    } else {
        state.screen_position_in_world = state.new_screen_pos;
        state.offset_px = Point::zero();
    }

    let running = !state.paused && !state.replay;
    let mut entire_turn_ended = false;
    // Pause entity processing during animations when replaying (so
    // it's all easy to see) but allow the keys to be processed when
    // playing the game normally. I.e. the players can move even
    // during animations if they so please.
    let no_animations = if state.replay {
        state.explosion_animation.is_none() && state.pos_timer.finished()
    } else {
        true
    };
    let simulation_area = Rectangle::center(state.player.pos, state.map_size);

    if (running || paused_one_step || timed_step) && state.side != Side::Victory && no_animations {
        let monster_count = state.world.monsters(simulation_area).count();
        let monster_with_ap_count = state
            .world
            .monsters(simulation_area)
            .filter(|m| m.has_ap(1))
            .count();
        let monster_cumulative_ap: i32 = state
            .world
            .monsters(simulation_area)
            .map(|m| m.ap.to_int())
            .sum();
        log::debug!(
            "Player AP: {}, monsters: {}, active mon: {}, total mon AP: {}",
            state.player.ap(),
            monster_count,
            monster_with_ap_count,
            monster_cumulative_ap
        );

        process_keys(&mut state.keys, &mut state.commands);
        let mouse_command = match option {
            Some(Action::UseFood) => Some(Command::UseFood),
            Some(Action::UseDose) => Some(Command::UseDose),
            Some(Action::UseCardinalDose) => Some(Command::UseCardinalDose),
            Some(Action::UseDiagonalDose) => Some(Command::UseDiagonalDose),
            Some(Action::UseStrongDose) => Some(Command::UseStrongDose),

            Some(Action::MoveN) => Some(Command::N),
            Some(Action::MoveS) => Some(Command::S),
            Some(Action::MoveW) => Some(Command::W),
            Some(Action::MoveE) => Some(Command::E),

            Some(Action::MoveNW) => Some(Command::NW),
            Some(Action::MoveNE) => Some(Command::NE),
            Some(Action::MoveSW) => Some(Command::SW),
            Some(Action::MoveSE) => Some(Command::SE),
            _ => None,
        };

        if let Some(command) = mouse_command {
            state.commands.push_front(command);
        }

        if state.mouse.left_clicked {
            state.path_walking_timer.finish();
        } else {
            state.path_walking_timer.update(dt);
        }

        // NOTE: Show the path from the player to the mouse pointer
        let mouse_inside_map =
            state.mouse.tile_pos >= (0, 0) && state.mouse.tile_pos < state.map_size;

        let visible = state.mouse_world_position().inside_circular_area(
            state.player.pos,
            formula::exploration_radius(state.player.mind),
        );

        if state.world.initialised() && state.player.alive() && mouse_inside_map && visible {
            let source = state.player.pos;
            let destination = state.mouse_world_position();
            let path = pathfinding::Path::find(
                source,
                destination,
                &state.world,
                Blocker::WALL,
                state.player.pos,
                state.player.will.to_int(),
                formula::PATHFINDING_PLAYER_MOUSE_LIMIT,
                &pathfinding::player_cost,
            );
            for point in path.clone() {
                let screen_pos = state.screen_pos_from_world_pos(point);
                highlighted_tiles.push(screen_pos);
            }

            state.player_path = path;
        }

        // NOTE: Process 1 action point of the player and then 1 AP of
        // all monsters. This means that their turns will alternate.
        // E.g. if the player has 2 APs and they're close to a
        // Depression, the player will move 1 turn first, then
        // Depression 1 etc.

        let player_ap = state.player.ap();
        if state.player.ap() >= 1 {
            process_player(state, audio, simulation_area);
        }
        let player_took_action = player_ap > state.player.ap();
        let monsters_can_move = state.player.ap() == 0 || player_took_action;

        if state.explosion_animation.is_none() {
            if monsters_can_move {
                process_monsters(
                    &mut state.world,
                    &mut state.player,
                    simulation_area,
                    &mut state.rng,
                    audio,
                );
            } else {
                log::debug!("Monsters waiting for player.");
            }
        } else {
            log::debug!("Monster's waiting for the explosion to end.");
        }

        // NOTE: the anxiety counter bar is hidden at the start, but
        // we want to show it as soon as it increases.
        if player_took_action && !state.player.anxiety_counter.is_min() {
            state.show_anxiety_counter = true;
        }

        if player_took_action && state.player.mind.is_high() {
            if let Some(victory_npc_id) = state.victory_npc_id.take() {
                log::info!("Player got High, the Victory NPC disappears!");
                if let Some(vnpc) = state.world.monster_mut(victory_npc_id) {
                    // TODO: move this (and other init stuff from
                    // Monster::new) to custom functions?
                    vnpc.kind = monster::Kind::Signpost;
                    vnpc.behavior = ai::Behavior::Immobile;
                    vnpc.ai_state = ai::AIState::NoOp
                }
            }
        }

        // Reset all action points only after everyone is at zero:
        let player_turn_ended = !state.player.has_ap(1);
        let monster_turn_ended = state
            .world
            .monsters(simulation_area)
            .filter(|m| m.has_ap(1))
            .count()
            == 0;

        entire_turn_ended = player_turn_ended && monster_turn_ended;
    }

    // Log or check verifications
    if entire_turn_ended {
        if state.replay {
            if let Some(expected) = state.verifications.pop_front() {
                let actual = state.verification();
                verify_states(&expected, &actual);

                if player_was_alive && !state.player.alive() && !state.commands.is_empty() {
                    panic!(
                        "Game quit too early -- there are still {} \
                         commands queued up.",
                        state.commands.len()
                    );
                }
            } else {
                // NOTE: no verifications were loaded. Probably
                // replaying a release build.
            }
        } else if cfg!(feature = "verifications") {
            let verification = state.verification();
            state::log_verification(&mut state.command_logger, &verification);
        }
    }

    // Reset the player & monster action points
    // NOTE: doing this only after we've logged the validations. Actually maybe we want to do this
    // before we start turn processing??
    if entire_turn_ended {
        log::debug!("Starting new turn for player and monsters.");
        state.player.new_turn();
        for monster in state.world.monsters_mut(simulation_area) {
            monster.new_turn();
        }
    }

    if entire_turn_ended {
        log::debug!("Turn {} has ended.", state.turn);
        state.turn += 1;
    }

    // NOTE: Load up new chunks if necessary
    if entire_turn_ended {
        for pos in simulation_area.points() {
            state.world.ensure_chunk_at_pos(pos);
        }
    }

    // Run the dose explosion effect here:
    if let Some(ref anim) = state.explosion_animation {
        for (pos, _, effect) in anim.tiles() {
            if effect.contains(animation::TileEffect::KILL) {
                kill_monster(pos, &mut state.world, audio);
            }
            if effect.contains(animation::TileEffect::SHATTER) {
                if let Some(cell) = state.world.cell_mut(pos) {
                    cell.tile.kind = TileKind::Empty;
                    cell.tile.graphic = Graphic::Empty;
                    cell.items.clear();
                }
            }
        }
    }

    // Set the fadeout animation on death
    if player_was_alive && !state.player.alive() {
        use crate::player::CauseOfDeath::*;
        audio.mix_sound_effect(Effect::GameOver, Duration::from_millis(0));
        let cause_of_death = formula::cause_of_death(&state.player);
        let fade_color = if cfg!(feature = "recording") {
            state.palette.fade_to_black_animation
        } else {
            match cause_of_death {
                Some(Exhausted) => state.palette.exhaustion_animation,
                Some(Overdosed) => state.palette.overdose_animation,
                Some(_) => state.palette.death_animation,
                None => {
                    // NOTE: this shouldn't happen (there should always be
                    // a cause of death) but if it did, we won't crash
                    state.palette.death_animation
                }
            }
        };

        let fade = formula::mind_fade_value(state.player.mind);

        let fade_out_ms = if cfg!(feature = "recording") {
            700
        } else if state.replay_full_speed {
            500
        } else {
            2500
        };

        let fade_in_ms = if cfg!(feature = "recording") {
            700
        } else if state.replay_full_speed {
            200
        } else {
            500
        };

        let initial_fade_percentage = 1.0 - fade;
        if state.challenge.one_chance {
            state.game_ended = true;
            state.show_endscreen_and_uncover_map_during_fadein = true;
            log::debug!("Game real time: {:?}", state.clock);
        } else {
            // NOTE: Don't die, reset the player to the initial state instead:
            state.player.reset();
        }
        state.screen_fading = Some(animation::ScreenFade::new(
            fade_color,
            Duration::from_millis(fade_out_ms),
            Duration::from_millis(200),
            Duration::from_millis(fade_in_ms),
            initial_fade_percentage,
        ));
    }

    let explored = state
        .world
        .cell(state.mouse_world_position())
        .map_or(true, |cell| cell.explored);

    // NOTE: show tooltip of a hovered-over object
    let tooltip = if !explored && settings.hide_unseen_tiles {
        None
    } else if state.mouse_world_position() == state.player.pos {
        Some("Player Character")
    } else if let Some(monster) = state.world.monster_on_pos(state.mouse_world_position()) {
        Some(monster.name())
    } else if let Some(cell) = state.world.cell(state.mouse_world_position()) {
        cell.items.get(0).map(|item| item.kind.name())
    } else {
        None
    };
    if let Some(tooltip) = tooltip {
        egui::show_tooltip_text(ui.ctx(), egui::Id::new("Tile Tooltip"), tooltip);
    }

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
    let no_left_mouse = !state.mouse.left_is_down && !state.mouse.left_clicked;
    if state.pos_timer.finished() && no_left_mouse {
        let display_pos = state.screen_pos_from_world_pos(state.player.pos);
        // NOTE: this is the re-center speed. We calculate it based on
        // the map size. That way the speed itself remains more or
        // less constant.
        let scroll_rate_ms_per_tile = 14;
        let max_display_size = std::cmp::max(state.map_size.x, state.map_size.y) as u64;
        let ms = if state.replay_full_speed {
            100
        } else {
            scroll_rate_ms_per_tile * max_display_size
        };
        let dur = Duration::from_millis(ms);
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
        // NOTE: this is no longer having any effect. Hints are disabled in `State::new`.

        let player_screen_pos = state.screen_pos_from_world_pos(state.player.pos);
        let d = 10;
        if player_screen_pos.x < d
            || player_screen_pos.y < d
            || state.map_size.x - player_screen_pos.x < d
            || state.map_size.y - player_screen_pos.y < d
        {
            state.show_keyboard_movement_hints = false;
        }

        if cfg!(feature = "recording") {
            state.show_keyboard_movement_hints = false;
        }
    }

    RunningState::Running
}

fn process_monsters(
    world: &mut World,
    player: &mut player::Player,
    area: Rectangle,
    rng: &mut Random,
    audio: &mut Audio,
) {
    if !player.alive() {
        return;
    }
    // NOTE: one quarter of the map area should be a decent overestimate
    let monster_count_estimate = area.size().x * area.size().y / 4;
    assert!(monster_count_estimate > 0);
    let mut monster_positions_vec = world
        .monsters(area)
        .filter(|m| m.has_ap(1))
        .map(|m| (m.ap.to_int(), m.position))
        .collect::<Vec<_>>();
    // NOTE: `world.monsters` does not give a stable result so we need to sort
    // it here to ensure correct replays.
    // NOTE: there's always at most one monster at a given position so this should always produce
    // the same ordering.
    //
    // We sort by action points (so depression always goes first), by
    // distance to player (so a closer monster can move first and make
    // space for another one near by) and then by coordinates just to
    // have some awy to always produce a stable ordering.
    monster_positions_vec
        .sort_by_key(|&(ap, pos)| (ap, player.pos.distance(pos) as i32, pos.x, pos.y));
    let mut monster_positions_to_process: VecDeque<_> = monster_positions_vec.into();

    while let Some((_, monster_position)) = monster_positions_to_process.pop_front() {
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
                let (newpos, newpath) = if monster_readonly.path.is_empty()
                    || path_changed
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
                        player.will.to_int(),
                        formula::PATHFINDING_MONSTER_LIMIT,
                        &pathfinding::monster_cost,
                    );
                    let newpos = path.next().unwrap_or(pos);
                    // Cache the path-finding result
                    let newpath = path.collect();
                    (newpos, newpath)
                } else {
                    (monster_readonly.path[0], monster_readonly.path[1..].into())
                };

                world.move_monster(pos, newpos, player.pos);
                let monster_visible = newpos
                    .inside_circular_area(player.pos, formula::exploration_radius(player.mind));
                if monster_visible {
                    let delay = audio.random_delay();
                    audio.mix_sound_effect(Effect::MonsterMoved, delay);
                }
                if let Some(monster) = world.monster_on_pos(newpos) {
                    monster.path = newpath;
                    if monster.has_ap(1) {
                        monster.trail = Some(newpos);
                    }
                }
            }

            Action::Attack(target_pos, damage) => {
                assert_eq!(target_pos, player.pos);
                player.take_effect(damage);
                if monster_readonly.die_after_attack {
                    kill_monster(monster_readonly.position, world, audio);
                }
                if !player.alive() {
                    player.perpetrator = Some(monster_readonly);
                    // The player's dead, no need to process other monsters
                    return;
                }
            }

            Action::Use(_) => unreachable!(),
        }
    }
}

fn process_player_action<W>(
    player: &mut player::Player,
    commands: &mut VecDeque<Command>,
    world: &mut World,
    simulation_area: Rectangle,
    explosion_animation: &mut Option<Box<dyn AreaOfEffect>>,
    rng: &mut Random,
    command_logger: &mut W,
    window_stack: &mut crate::windows::Windows<Window>,
    bumped_into_a_monster: &mut bool,
    palette: &Palette,
    audio: &mut Audio,
) where
    W: Write,
{
    if !player.alive() {
        log::debug!("Processing player action, but the player is dead.");
        return;
    }
    if !player.has_ap(1) {
        log::debug!(
            "Processing player action, but the player has no AP: {}",
            player.ap()
        );
        return;
    }
    if let Some(command) = commands.pop_front() {
        state::log_command(command_logger, command.clone());
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

            Command::ShowMessageBox {
                ttl,
                title,
                message,
            } => {
                window_stack.push(window::timed_message_box(title, message, ttl));
                return;
            }
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
                let mut path = pathfinding::Path::find(
                    player.pos,
                    dose_pos,
                    world,
                    Blocker::WALL,
                    player.pos,
                    player.will.to_int(),
                    formula::PATHFINDING_DOSE_RESIST_LIMIT,
                    &pathfinding::direct_cost,
                );

                let new_pos_opt = if path.len() <= resist_radius {
                    path.next()
                } else {
                    None
                };

                if let Some(new_pos) = new_pos_opt {
                    action = Action::Move(new_pos);
                } else {
                    // NOTE: no path leading to the irresistible dose
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
                    // info!("Player attacks {:?}", monster);
                    if let Some(kind) = world.monster_on_pos(dest).map(|m| m.kind) {
                        match kind {
                            monster::Kind::Anxiety => {
                                log::debug!(
                                    "Bumped into anxiety! Current anxiety counter: {:?}",
                                    player.anxiety_counter
                                );
                                let increment =
                                    if player.bonuses.contains(&CompanionBonus::DoubleWillGrowth) {
                                        2
                                    } else {
                                        1
                                    };
                                log::debug!("Anxiety increment: {:?}", increment);
                                player.anxiety_counter += increment;
                                log::debug!("New anxiety counter: {:?}", player.anxiety_counter);
                                if player.anxiety_counter.is_max() {
                                    log::info!("Increasing player's will");
                                    player.will += 1;
                                    player.anxiety_counter.set_to_min();
                                }
                            }
                            monster::Kind::Hunger => {
                                let modifier = Modifier::Attribute {
                                    state_of_mind: 3,
                                    will: 0,
                                };
                                player.take_effect(modifier);
                            }
                            // NOTE: NPCs don't give bonuses or accompany the player when high.
                            monster::Kind::Npc if player.mind.is_sober() => {
                                if let Some(monster) = world.monster_on_pos(dest) {
                                    log::info!("Bumped into NPC: {}", monster);
                                }

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
                                        log::info!("NPC {} accompanies the player.", npc);
                                        npc.accompanying_player = true;
                                        assert!(npc.companion_bonus.is_some());
                                    } else if npc.accompanying_player {
                                        log::info!("NPC {} leaves the player.", npc);
                                        npc.accompanying_player = false;
                                    }
                                }
                            }

                            monster::Kind::Signpost => {
                                log::info!("Bumped into a signpost!");
                                window_stack.push(
                                    window::message_box(
                                        "Message",
                                        "\"I thought you were going to stay sober for good. I was wrong. Goodbye.\""));
                            }

                            _ => {}
                        }
                        kill_monster(dest, world, audio);

                        if kind.is_monster() {
                            *bumped_into_a_monster = true;
                        }
                    }
                } else if dest_walkable {
                    player.spend_ap(1);
                    player.move_to(dest);
                    audio.mix_sound_effect(Effect::Walk, Duration::from_millis(0));
                    while let Some(item) = world.pickup_item(dest) {
                        use crate::item::Kind::*;
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
                                    use_dose(player, explosion_animation, item, palette, audio);
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
                    audio.mix_sound_effect(Effect::Explosion, Duration::from_millis(0));
                    let food = player.inventory.remove(food_idx);
                    player.take_effect(food.modifier);
                    let food_explosion_radius = 2;
                    let animation = animation::SquareExplosion::new(
                        player.pos,
                        food_explosion_radius,
                        1,
                        palette.explosion,
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
                    use_dose(player, explosion_animation, dose, palette, audio);
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
                    use_dose(player, explosion_animation, dose, palette, audio);
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
                    use_dose(player, explosion_animation, dose, palette, audio);
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
                    use_dose(player, explosion_animation, dose, palette, audio);
                }
            }

            Action::Attack(_, _) => {
                unreachable!();
            }
        }
    }
}

fn process_player(state: &mut State, audio: &mut Audio, simulation_area: Rectangle) {
    {
        // appease borrowck
        let player = &mut state.player;

        // NPCs should unfollow an intoxicated player:
        if player.mind.is_high() {
            let npcs = state.world.monsters_mut(simulation_area).filter(|m| {
                m.kind == monster::Kind::Npc && m.accompanying_player && m.companion_bonus.is_some()
            });
            for npc in npcs {
                log::info!("{:?} will not accompany an intoxicated player.", npc);
                npc.accompanying_player = false;
            }
        }

        let world = &state.world;

        // NOTE: if the player manages to outrun the NPC (they follow
        // the player but it still can happen), the bonus will
        // disappear.
        let npc_bonuses = world
            .monsters(simulation_area)
            .filter(|m| {
                m.kind == monster::Kind::Npc && m.accompanying_player && m.companion_bonus.is_some()
            })
            .map(|m| m.companion_bonus.unwrap());
        player.bonuses.clear();
        player.bonuses.extend(npc_bonuses);
    }

    // NOTE: If the player is following a path move them one step along the path
    let visible = state.mouse_world_position().inside_circular_area(
        state.player.pos,
        formula::exploration_radius(state.player.mind),
    );
    if state.mouse.left_is_down && visible && state.path_walking_timer.finished() {
        state.path_walking_timer.reset();
        if let Some(destination) = state.player_path.next() {
            let command = match destination - state.player.pos {
                Point { x: 0, y: -1 } => Some(Command::N),
                Point { x: 0, y: 1 } => Some(Command::S),
                Point { x: -1, y: 0 } => Some(Command::W),
                Point { x: 1, y: 0 } => Some(Command::E),

                Point { x: -1, y: -1 } => Some(Command::NW),
                Point { x: -1, y: 1 } => Some(Command::SW),
                Point { x: 1, y: -1 } => Some(Command::NE),
                Point { x: 1, y: 1 } => Some(Command::SE),

                _ => None,
            };
            if let Some(command) = command {
                state.commands.push_front(command)
            }
        }
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
        &mut state.window_stack,
        &mut state.player_bumped_into_a_monster,
        &state.palette,
        audio,
    );

    // If the player ever picks up a dose, mark it in this variable:
    let player_picked_up_a_dose = state.player.inventory.iter().any(item::Item::is_dose);
    if player_picked_up_a_dose {
        state.player_picked_up_a_dose = true;
    }

    let spent_ap_this_turn = previous_action_points > state.player.ap();

    // Place the Victory NPC if the player behaved themself.
    if state.player.will.is_max() && !state.player.mind.is_high() && state.victory_npc_id.is_none()
    {
        let vnpc_pos = place_victory_npc(state);
        // NOTE: Scroll to the Victory NPC position
        state.pos_timer = Timer::new(Duration::from_millis(2000));
        state.old_screen_pos = state.screen_position_in_world;
        state.new_screen_pos = vnpc_pos;
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

    // NOTE: The player has reached the Victory NPC. Win the game! \o/
    if state.player.bonuses.contains(&CompanionBonus::Victory) {
        win_the_game(state);
    }

    state.world.explore(
        state.player.pos,
        formula::exploration_radius(state.player.mind),
    );
}

fn process_keys(keys: &mut Keys, commands: &mut VecDeque<Command>) {
    use crate::keys::KeyCode::*;
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
            Key {
                code: Left,
                alt: true,
                shift: false,
                ..
            } => commands.push_back(Command::SW),
            Key {
                code: Left,
                logo: true,
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
            Key {
                code: Right,
                alt: true,
                shift: false,
                ..
            } => commands.push_back(Command::SE),
            Key {
                code: Right,
                logo: true,
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

            _ => {
                if let Some(command) = inventory_commands(key) {
                    commands.push_back(command)
                }
            }
        }
    }
}

fn inventory_commands(key: Key) -> Option<Command> {
    use crate::item::Kind;
    use crate::keys::KeyCode::*;

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
    // NOTE: use the order defined in `Kind::iter` so the keys always
    // correspond to the order we display the items in.
    for (index, current_kind) in item::Kind::iter().enumerate() {
        if current_kind == kind {
            return (index + 1) as u8;
        }
    }
    unreachable!()
}

fn kill_monster(monster_position: Point, world: &mut World, audio: &mut Audio) {
    let invincible = world
        .monster_on_pos(monster_position)
        .map_or(false, |m| m.invincible);
    if invincible {
        // It's invincible: no-op
    } else {
        if let Some(monster) = world.monster_on_pos(monster_position) {
            monster.dead = true;
            audio.mix_sound_effect(Effect::MonsterHit, Duration::from_millis(0));
        }
        world.remove_monster(monster_position);
    }
}

fn use_dose(
    player: &mut player::Player,
    explosion_animation: &mut Option<Box<dyn AreaOfEffect>>,
    item: item::Item,
    palette: &Palette,
    audio: &mut Audio,
) {
    use crate::item::Kind::*;
    use crate::player::Modifier::*;
    log::debug!("Using dose");
    audio.mix_sound_effect(Effect::Explosion, Duration::from_millis(0));
    // TODO: do a different explosion animation for the cardinal dose
    if let Intoxication { state_of_mind, .. } = item.modifier {
        let radius = if state_of_mind <= 100 { 4 } else { 6 };
        player.take_effect(item.modifier);
        let animation: Box<dyn AreaOfEffect> = match item.kind {
            Dose | StrongDose => Box::new(animation::SquareExplosion::new(
                player.pos,
                radius,
                2,
                palette.explosion,
            )),
            CardinalDose => Box::new(animation::CardinalExplosion::new(
                player.pos,
                radius,
                2,
                palette.explosion,
                palette.shattering_explosion,
            )),
            DiagonalDose => Box::new(animation::DiagonalExplosion::new(
                player.pos,
                radius,
                2,
                palette.explosion,
                palette.shattering_explosion,
            )),
            Food => unreachable!(),
        };
        *explosion_animation = Some(animation);
    } else {
        unreachable!();
    }
}

fn show_exit_stats(stats: &Stats) {
    log::info!(
        "\nSlowest update durations: {:?}\n",
        stats
            .longest_update_durations()
            .iter()
            .map(|dur| dur.as_secs_f32() * 1000.0) // milliseconds in f32
            .rev()
            .collect::<Vec<_>>(),
    );

    log::info!(
        "\nSlowest drawcall durations: {:?}\n",
        stats
            .longest_drawcall_durations()
            .iter()
            .map(|dur| dur.as_secs_f32() * 1000.0) // milliseconds in f32
            .rev()
            .collect::<Vec<_>>(),
    );

    log::info!(
        "\nMean update duration: {} ms\nMean drawcall duration: {} ms",
        stats.mean_update(),
        stats.mean_drawcalls()
    );

    log::info!("Mean FPS: {}", stats.mean_fps());
    log::info!("Lowest FPS: {}", stats.lowest_fps());
}

fn verify_states(expected: &state::Verification, actual: &state::Verification) {
    if expected.chunk_count != actual.chunk_count {
        log::error!(
            "Expected chunks: {}, actual: {}",
            expected.chunk_count,
            actual.chunk_count
        );
    }
    if expected.player_pos != actual.player_pos {
        log::error!(
            "Expected player position: {}, actual: {}",
            expected.player_pos,
            actual.player_pos
        );
    }
    if expected.monsters.len() != actual.monsters.len() {
        log::error!(
            "Expected monster count: {}, actual: {}",
            expected.monsters.len(),
            actual.monsters.len()
        );
    }
    if expected.monsters != actual.monsters {
        let expected_monsters: HashMap<Point, (Point, monster::Kind)> = expected
            .monsters
            .iter()
            .map(|&(pos, chunk_pos, monster)| (pos, (chunk_pos, monster)))
            .collect();
        let actual_monsters: HashMap<Point, (Point, monster::Kind)> = actual
            .monsters
            .iter()
            .map(|&(pos, chunk_pos, monster)| (pos, (chunk_pos, monster)))
            .collect();

        for (pos, expected) in &expected_monsters {
            match actual_monsters.get(pos) {
                Some(actual) => {
                    if expected != actual {
                        log::error!(
                            "Monster at {} differ. Expected: {:?}, \
                             actual: {:?}",
                            pos,
                            expected,
                            actual
                        );
                    }
                }
                None => {
                    log::error!(
                        "Monster expected at {}: {:?}, but it's not \
                         there.",
                        pos,
                        expected
                    );
                }
            }
        }

        for (pos, actual) in &actual_monsters {
            if expected_monsters.get(pos).is_none() {
                log::error!("There is an unexpected monster at: {}: {:?}.", pos, actual);
            }
        }
    }
    assert_eq!(expected, actual, "Validation failed!");
}

pub fn create_new_game_state(state: &State, new_challenge: Challenge) -> State {
    let mut state = State::new_game(
        state.world_size,
        state.map_size,
        state.panel_width,
        state.exit_after,
        state::generate_replay_path(),
        new_challenge,
        state.palette,
    );
    state.generate_world();
    state
}

fn place_victory_npc(state: &mut State) -> Point {
    log::info!("Generating the Victory NPC!");
    let mut distance_range = formula::VICTORY_NPC_DISTANCE;
    // NOTE: Compute path to Victory NPC that is reachable by the
    // player. This may take several attempts. Leave the position
    // immutable at the end.
    let mut vnpc_pos;
    let mut attempts = 250;
    let blockers = Blocker::WALL | Blocker::MONSTER;
    loop {
        if attempts <= 0 {
            // TODO: generate VNPC at a shorter distance instead of crashing?
            log::warn!("Could not find a viable Victory NPC position in 250 tries.");
            let min = distance_range.0 - 20;
            let max = distance_range.1 - 20;
            if min > 5 && max > 5 {
                distance_range = InclusiveRange(min, max);
                attempts = 20;
                log::info!("Reduced VNPC spawn range to: {:?}.", distance_range);
            } else {
                log::warn!(
                    "Could not find a viable Victory NPC position anywhere! Winning game instead."
                );
                state.player.bonuses.push(monster::CompanionBonus::Victory);
                win_the_game(state);
                return state.player.pos;
            }
        } else {
            attempts -= 1;
        }

        // NOTE: this is a little convoluted. We test if the Victory
        // NPC position is walkable. And if it's not, we try other
        // positions in its immediate vicinity instead of generating a
        // new candidate position via `formula::victory_npc_position`.
        //
        // We do this, because the walkability test requires we have a
        // World Chunk in place and generating these can be expensive.
        // So if we picked a completely random position every time, we
        // could end up generating a lot of chunks for no immediate
        // reason.
        //
        // What we do instead is generate one Chunk for the given
        // position and then try nearby areas (which will be
        // overwhelmingly likely in the same Chunk).
        vnpc_pos = formula::victory_npc_position(&mut state.rng, state.player.pos, distance_range);
        log::info!("Trying to find test NPC position {:?}", vnpc_pos);
        state.world.ensure_chunk_at_pos(vnpc_pos);
        if let Some(pos) = walkable_place_nearby(&state.world, vnpc_pos, blockers, state.player.pos)
        {
            log::info!("Position {:?} is walkable!", pos);
            vnpc_pos = pos;
            for cell_pos in point::Line::new(state.player.pos, vnpc_pos) {
                state.world.ensure_chunk_at_pos(cell_pos);
            }
        } else {
            log::warn!(
                "Failed to find empty place around the candidate VNPC position {:?}",
                vnpc_pos
            );
            continue;
        }

        log::info!(
            "player pos: {:?}, vnpc pos: {:?}",
            state.player.pos,
            vnpc_pos
        );
        // TODO: make sure the world chunks exist before trying to find path
        let path_to_vnpc = pathfinding::Path::find(
            state.player.pos,
            vnpc_pos,
            &state.world,
            blockers,
            state.player.pos,
            state.player.will.to_int(),
            formula::PATHFINDING_VNPC_REACHABILITY_LIMIT,
            &pathfinding::direct_cost,
        );
        if path_to_vnpc.len() == 0 {
            log::warn!("Failed to find path from player to Victory NPC!")
        } else {
            log::info!("Path to Victory NPC takes {} steps", path_to_vnpc.len());
            break;
        }
    }
    let vnpc_pos = vnpc_pos;

    if let Some(prev_npc_id) = state.victory_npc_id.take() {
        log::warn!("Replacing an existing NPC! {:?}", prev_npc_id);
        state.world.remove_monster_by_id(prev_npc_id);
    }

    // NOTE: Uncover the map leading to the Victory NPC position
    let positions = point::Line::new(state.player.pos, vnpc_pos);
    for cell_pos in positions {
        state.world.ensure_chunk_at_pos(cell_pos);
        let display_half_size = state.map_size / 2;
        // NOTE: make sure every cell that will be shown has a chunk.
        //
        // If we didn't do this, we would get blank places when the line would cross a boundary
        // of two chunks, but the surrounding chunks were not brought in.
        state
            .world
            .ensure_chunk_at_pos(cell_pos + (display_half_size.x, display_half_size.y));
        state
            .world
            .ensure_chunk_at_pos(cell_pos + (-display_half_size.x, display_half_size.y));
        state
            .world
            .ensure_chunk_at_pos(cell_pos + (display_half_size.x, -display_half_size.y));
        state
            .world
            .ensure_chunk_at_pos(cell_pos + (-display_half_size.x, -display_half_size.y));
        state.world.always_visible(cell_pos, 2);
        state.world.explore(cell_pos, 4);
    }
    state.world.explore(vnpc_pos, 5);
    state.world.always_visible(vnpc_pos, 2);

    if let Some(chunk) = state.world.chunk_mut(vnpc_pos) {
        let mut monster = monster::Monster::new(monster::Kind::Npc, vnpc_pos, state.challenge);
        monster.companion_bonus = Some(CompanionBonus::Victory);
        // NOTE: The NPCs have the same colour range as the player,
        // but let's always pick a colour that's different from the
        // current player's one.
        monster.npc_color_index = {
            let mut index: usize = 0;
            for _ in 0..5 {
                let pick = state
                    .rng
                    .range_inclusive(0, state.palette.player.len() as i32 - 1)
                    as usize;
                if pick != state.player.color_index {
                    index = pick;
                    break;
                }
            }
            index
        };

        monster.ai_state = ai::AIState::NoOp;
        let id = chunk.add_monster(monster);
        state.victory_npc_id = Some(id);
    }

    vnpc_pos
}

fn win_the_game(state: &mut State) {
    state.side = Side::Victory;
    state.game_ended = true;
    state.uncovered_map = true;
    state.window_stack.push(Window::Endgame);
}

/// Return a point close to the given one that is walkable.
fn walkable_place_nearby(
    world: &World,
    pos: Point,
    blockers: Blocker,
    player_pos: Point,
) -> Option<Point> {
    // Radius `2` means the central point and the eight surrounding ones.
    point::SquareArea::new(pos, 2).find(|&point| world.walkable(point, blockers, player_pos))
}
