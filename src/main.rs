#![deny(overflowing_literals)]

extern crate rand;
extern crate time;
pub extern crate tcod;
// extern crate rustbox;


use std::collections::VecDeque;
use std::cmp;
use std::env;
use std::io::Write;
use std::path::Path;

use rand::Rng;
use tcod::input::Key;
use time::Duration;

use color::Color;
use engine::{Engine, KeyCode};
use game_state::{Command, GameState, Side};


mod animation;
mod color;
mod engine;
mod game_state;
mod generators;
mod graphics;
mod item;
mod level;
mod monster;
mod pathfinding;
mod player;
mod point;
mod ranged_int;
mod world;


#[derive(Copy, Clone)]
pub struct Timer {
    max: Duration,
    current: Duration,
}

impl Timer {
    pub fn new(duration: Duration) -> Timer {
        Timer {
            max: duration,
            current: duration,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        if dt > self.current {
            self.current = Duration::zero();
        } else {
            self.current = self.current - dt;
        }
    }

    pub fn percentage_remaining(&self) -> f32 {
        (self.current.num_milliseconds() as f32) / (self.max.num_milliseconds() as f32)
    }

    pub fn percentage_elapsed(&self) -> f32 {
        1.0 - self.percentage_remaining()
    }

    pub fn finished(&self) -> bool {
        self.current.is_zero()
    }

    pub fn reset(&mut self) {
        self.current = self.max;
    }
}

#[derive(Copy, Clone)]
pub struct ScreenFadeAnimation {
    pub color: Color,
    pub fade_out_time: Duration,
    pub wait_time: Duration,
    pub fade_in_time: Duration,
    pub timer: Timer,
    pub phase: ScreenFadePhase,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ScreenFadePhase {
    FadeOut,
    Wait,
    FadeIn,
    Done,
}

impl ScreenFadeAnimation {
    pub fn new(color: Color, fade_out: Duration, wait: Duration,
               fade_in: Duration) -> ScreenFadeAnimation {
        ScreenFadeAnimation {
            color: color,
            fade_out_time: fade_out,
            wait_time: wait,
            fade_in_time: fade_in,
            timer: Timer::new(fade_out),
            phase: ScreenFadePhase::FadeOut,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.timer.update(dt);
        if self.timer.finished() {
            match self.phase {
                ScreenFadePhase::FadeOut => {
                    self.timer = Timer::new(self.wait_time);
                    self.phase = ScreenFadePhase::Wait;
                }
                ScreenFadePhase::Wait => {
                    self.timer = Timer::new(self.fade_in_time);
                    self.phase = ScreenFadePhase::FadeIn;
                }
                ScreenFadePhase::FadeIn => {
                    self.phase = ScreenFadePhase::Done;
                }
                ScreenFadePhase::Done => {
                    // NOTE: we're done. Nothing to do here.
                }
            }
        }
    }
}

fn process_keys(keys: &mut VecDeque<Key>, commands: &mut VecDeque<Command>) {
    use tcod::input::KeyCode::*;
    // TODO: switch to DList and consume it with `mut_iter`.
    loop {
        match keys.pop_front() {
            Some(key) => {
                match key {
                    Key { code: Up, ..} | Key { code: NumPad8, .. } => commands.push_back(Command::N),
                    Key { code: Down, ..} | Key { code: NumPad2, .. }  => commands.push_back(Command::S),
                    Key { code: Left, ctrl: false, shift: true, .. } | Key { code: NumPad7, .. }  => commands.push_back(Command::NW),
                    Key { code: Left, ctrl: true, shift: false, .. } | Key { code: NumPad1, .. }  => commands.push_back(Command::SW),
                    Key { code: Left, .. } | Key { code: NumPad4, .. }  => commands.push_back(Command::W),
                    Key { code: Right, ctrl: false, shift: true, .. } | Key { code: NumPad9, .. }  => commands.push_back(Command::NE),
                    Key { code: Right, ctrl: true, shift: false, .. } | Key { code: NumPad3, .. }  => commands.push_back(Command::SE),
                    Key { code: Right, .. } | Key { code: NumPad6, .. }  => commands.push_back(Command::E),
                    Key { printable: 'e', .. } | Key { printable: '1', .. } => {
                        commands.push_back(Command::UseFood);
                    }
                    Key { printable: '2', ..} => {
                        commands.push_back(Command::UseDose);
                    }
                    Key { printable: '3', ..} => {
                        commands.push_back(Command::UseStrongDose);
                    }
                    _ => (),
                }
            },
            None => break,
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
            explosion_animation: &mut animation::Explosion,
            item: item::Item) {
    use player::Modifier::*;
    if let Intoxication{state_of_mind, ..} = item.modifier {
        let radius = match state_of_mind <= 100 {
            true => 4,
            false => 6,
        };
        player.take_effect(item.modifier);
        let anim = explode(player.pos, radius, world);
        *explosion_animation = anim;
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
    Some((center,
          radius,
          2,  // this means it'll be visible at the first frame
          color::explosion,
          Duration::zero()))
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
                               explosion_animation: &mut animation::Explosion,
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
                    let anim = explode(player.pos, food_explosion_radius, world);
                    *explosion_animation = anim;
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


fn render_panel(x: i32, width: i32, display: &mut engine::Display, state: &GameState, dt: Duration, fps: i32) {
    let fg = color::gui_text;
    let bg = color::dim_background;
    {
        let height = display.size().y;
        display.clear_rect((x, 0), (width, height), bg);
    }

    let player = &state.player;

    let (mind_str, mind_val_percent) = match player.mind {
        player::Mind::Withdrawal(val) => ("Withdrawal", val.percent()),
        player::Mind::Sober(val) => ("Sober", val.percent()),
        player::Mind::High(val) => ("High", val.percent()),
    };

    let mut lines = vec![
        mind_str.into(),
        "".into(), // NOTE: placeholder for the Mind state percentage bar
        "".into(),
        format!("Will: {}", *player.will),
    ];

    if player.inventory.len() > 0 {
        lines.push("Inventory:".into());
        let food_amount = player.inventory.iter().filter(|i| i.kind == item::Kind::Food).count();
        if food_amount > 0 {
            lines.push(format!("[1] Food: {}", food_amount));
        }

        let dose_amount = player.inventory.iter().filter(|i| i.kind == item::Kind::Dose).count();
        if dose_amount > 0 {
            lines.push(format!("[2] Dose: {}", dose_amount));
        }

        let strong_dose_amount = player.inventory.iter().filter(|i| i.kind == item::Kind::StrongDose).count();
        if strong_dose_amount > 0 {
            lines.push(format!("[3] Strong Dose: {}", strong_dose_amount));
        }
    }

    lines.push("".into());

    if player.will.is_max() {
        lines.push(format!("Sobriety: {}", player.sobriety_counter.percent()));
    }

    if state.cheating {
        lines.push("CHEATING".into());
        lines.push("".into());
    }

    if state.side == Side::Victory {
        lines.push(format!("VICTORY!"));
    }

    if player.alive() {
        if *player.stun > 0 {
            lines.push(format!("Stunned({})", *player.stun));
        }
        if *player.panic > 0 {
            lines.push(format!("Panicking({})", *player.panic));
        }
    } else {
        lines.push("Dead".into());
    }

    for (y, line) in lines.iter().enumerate() {
        display.write_text(line, (x + 1, y as i32), fg, bg);
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
    display.progress_bar(mind_val_percent, (x + 1, 1), bar_width,
                         color::gui_progress_bar_fg, color::gui_progress_bar_bg);

    let bottom = display.size().y - 1;
    display.write_text(&format!("dt: {}ms", dt.num_milliseconds()), (x + 1, bottom - 1), fg, bg);
    display.write_text(&format!("FPS: {}", fps), (x + 1, bottom), fg, bg);

}


fn update(mut state: GameState, dt: Duration, engine: &mut engine::Engine) -> Option<GameState> {
    state.clock = state.clock + dt;

    // Quit the game when Q is pressed
    if engine.key_pressed(Key { printable: 'q', pressed: true, code: KeyCode::Char, .. Default::default() }) {
        return None;
    }

    // Restart the game on F5
    if engine.key_pressed(Key { code: KeyCode::F5, pressed: true, .. Default::default() }) {
        engine.keys.clear();
        let state = GameState::new_game(state.world_size, state.map_size.x, state.panel_width, state.display_size);
        return Some(state);
    }

    // Full screen on Alt-Enter
    if let Some(key) = engine.keys.pop_front() {
        if key.code == KeyCode::Enter && (key.left_alt || key.right_alt) {
            engine.toggle_fullscreen();
        } else {
            engine.keys.push_front(key);
        }
    }

    // Uncover map
    if engine.key_pressed(Key { code: KeyCode::F6, pressed: true, .. Default::default() }) {
        state.cheating = !state.cheating;
    }

    state.paused = if state.replay && engine.read_key(KeyCode::Spacebar) {
        !state.paused
    } else {
        state.paused
    };

    let paused_one_step = state.paused && engine.read_key(KeyCode::Right);
    let timed_step = if state.replay && !state.paused && state.clock.num_milliseconds() >= 50 {
        state.clock = Duration::zero();
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
        process_keys(&mut engine.keys, &mut state.commands);

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
            state.pos_timer = Timer::new(dur);
            state.old_screen_pos = state.screen_position_in_world;
            state.new_screen_pos = (state.player.pos.x, state.old_screen_pos.y).into();
        } else if display_pos.y < exploration_radius || display_pos.y >= state.map_size.y - exploration_radius {
            // change the screen centre to that of the player
            state.pos_timer = Timer::new(dur);
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
                engine.display.fade(fade , color::Color{r: 0, g: 0, b: 0});
            }
            Sober(_) | High(_) => {
                // NOTE: Not withdrawn, don't fade
            }
        }
    } else if player_was_alive {  // NOTE: Player just died
        state.screen_fading = Some(ScreenFadeAnimation::new(
            color::death_animation,
            Duration::milliseconds(500),
            Duration::milliseconds(200),
            Duration::milliseconds(300)));
    } else {
        // NOTE: player is already dead (didn't die this frame)
    }

    if let Some(mut anim) = state.screen_fading {
        if anim.timer.finished() {
            state.screen_fading = None;
        } else {
            let fade = match anim.phase {
                ScreenFadePhase::FadeOut => anim.timer.percentage_remaining(),
                ScreenFadePhase::Wait => 0.0,
                ScreenFadePhase::FadeIn => anim.timer.percentage_elapsed(),
                ScreenFadePhase::Done => {
                    // NOTE: this should have been handled by the if statement above.
                    unreachable!();
                }
            };
            engine.display.fade(fade, anim.color);
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

    // Render the level and items:
    let player_will_is_max = state.player.will.is_max();
    let player_will = *state.player.will;
    // NOTE: this is here to appease the borrow checker. If we
    // borrowed the state here as immutable, we wouln't need it.
    let is_high = state.player.mind.is_high();

    state.world.with_cells(screen_left_top_corner, map_size, |world_pos, cell| {
        let display_pos = screen_coords_from_world(world_pos);
        if !within_map_bounds(display_pos) {
            return;
        }

        // Render the tile
        let mut rendered_tile = cell.tile;

        if is_high {
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
            graphics::draw(&mut engine.display, dt, display_pos, &rendered_tile);
        } else if cell.explored || bonus == player::Bonus::UncoverMap {
            graphics::draw(&mut engine.display, dt, display_pos, &rendered_tile);
            engine.display.set_background(display_pos, color::dim_background);
        }

        // Render the irresistible background of a dose
        for item in cell.items.iter() {
            if item.is_dose() && !player_will_is_max {
                let resist_radius = player_resist_radius(item.irresistible, player_will);
                for point in point::SquareArea::new(world_pos, resist_radius) {
                    if in_fov(point) {
                        let screen_coords = screen_coords_from_world(point);
                        engine.display.set_background(screen_coords, color::dose_background);
                    }
                }
            }
        }

        // Render the items
        if in_fov(world_pos) || cell.explored || bonus == player::Bonus::SeeMonstersAndItems || bonus == player::Bonus::UncoverMap {
            for item in cell.items.iter() {
                graphics::draw(&mut engine.display, dt, display_pos, item);
            }
        }
    });

    if let Some((center, max_r, r, c, elapsed)) = state.explosion_animation {
        let one_level_duration = Duration::milliseconds(100);
        let mut elapsed = elapsed + dt;
        let r = if elapsed > one_level_duration {
            elapsed = elapsed - one_level_duration;
            r + 1
        } else {
            r
        };
        if r <= max_r {
            state.explosion_animation = Some((center, max_r, r, c, elapsed));
            for world_pos in point::SquareArea::new(center, r) {
                if state.world.within_bounds(world_pos) {
                    let display_pos = screen_coords_from_world(world_pos);
                    if within_map_bounds(display_pos) {
                        engine.display.set_background(display_pos, c);
                    }
                }
            }
        } else {
            state.explosion_animation = None;
        }

    }

    for monster_pos in state.world.monster_positions(screen_left_top_corner, state.map_size) {
        if let Some(monster) = state.world.monster_on_pos(monster_pos) {
            let visible = monster.position.distance(state.player.pos) < (radius as f32);
            if visible || bonus == player::Bonus::UncoverMap || bonus == player::Bonus::SeeMonstersAndItems {
                let world_pos = monster.position;
                let display_pos = screen_coords_from_world(world_pos);
                if within_map_bounds(display_pos) {
                    graphics::draw(&mut engine.display, dt, display_pos, monster);
                }
            }
        }
    }

    {
        let world_pos = state.player.pos;
        let display_pos = screen_coords_from_world(world_pos);
        if within_map_bounds(display_pos) {
            graphics::draw(&mut engine.display, dt, display_pos, &state.player);
        }
    }
    let fps = engine.fps();
    render_panel(state.map_size.x, state.panel_width, &mut engine.display, &state, dt, fps);
    Some(state)
}


fn main() {
    // NOTE: at our current font, the height of 43 is the maximum value for
    // 1336x768 monitors.
    let map_size = 43;
    let panel_width = 20;
    let display_size = (map_size + panel_width, map_size).into();
    // NOTE: 2 ^ 30
    let world_size = (1_073_741_824, 1_073_741_824).into();
    let title = "Dose Response";
    let font_dir = Path::new("fonts");
    let font_path = font_dir.join("dejavu16x16_gs_tc.png");

    let game_state = match env::args().count() {
        1 => {  // Run the game with a new seed, create the replay log
            // TODO: directory creation is unix-specific because permissions.
            // This should probably be taken out of GameState and moved here or
            // to some platform-specific layer.
            GameState::new_game(world_size, map_size, panel_width, display_size)
        },
        2 => {  // Replay the game from the entered log
            GameState::replay_game(world_size, map_size, panel_width, display_size)
        },
        _ => panic!("You must pass either pass zero or one arguments."),
    };

    let screen_pixel_size = tcod::system::get_current_resolution();
    println!("Current resolution: {:?}", screen_pixel_size);
    // TODO: maybe we could just query the current resolution with SDL2 and then use the value here?
    // Question is, will that clash with the existing SDL context that libtcod sets up?
    //
    // TODO: Alternatively, can we use libtcod + sdl2? It doesn't seem
    // to be in the makefiles for now, but maybe we can just enable it
    // somehow.
    //
    // TODO: check the screen_width/screen_height values against known
    // (supported?) monitor resolutions. Only force fullscreen res if it's
    // one of the known ones.
    tcod::system::force_fullscreen_resolution(screen_pixel_size.0, screen_pixel_size.1);

    let mut engine = Engine::new(display_size, color::background, title, &font_path);
    engine.main_loop(game_state, update);
}
