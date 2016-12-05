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
                    Key { printable: 'e', .. } => {
                        commands.push_back(Command::Eat);
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
    Eat,
}


fn kill_monster(monster: &mut monster::Monster, level: &mut level::Level) {
    monster.dead = true;
    level.remove_monster(monster.id(), monster);
}

// TODO: prolly refactor to a struct?
// Fields: position, max radius, current radius, colour, elapsed time
pub type ExplosionAnimation = Option<(point::Point, i32, i32, color::Color, Duration)>;

fn explode(center: point::Point,
           radius: i32,
           level: &mut level::Level,
           monsters: &mut Vec<monster::Monster>) -> ExplosionAnimation {
    for pos in point::SquareArea::new(center, radius) {
        if let Some(monster_id) = level.monster_on_pos(pos) {
            kill_monster(&mut monsters[monster_id], level);
        }
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


fn process_player<R, W>(player: &mut player::Player,
                        commands: &mut VecDeque<Command>,
                        level: &mut level::Level,
                        monsters: &mut Vec<monster::Monster>,
                        explosion_animation: &mut ExplosionAnimation,
                        rng: &mut R,
                        command_logger: &mut W)
    where R: Rng, W: Write {
    if !player.alive() {
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

            Command::Eat => Action::Eat,
        };
        if *player.stun > 0 {
            action = Action::Move(player.pos);
        } else if *player.panic > 0 {
            let new_pos = level.random_neighbour_position(
                rng, player.pos, level::Walkability::WalkthroughMonsters);
            action = Action::Move(new_pos);
        } else if let Some((dose_pos, dose)) = level.nearest_dose(player.pos, 5) {
            let new_pos_opt = {
                let point::Point{x: w, y: h} = level.size();
                let mut path = tcod::pathfinding::AStar::new_from_callback(
                    w, h,
                    |_from, to| -> f32 {
                        match level.walkable(to, level::Walkability::WalkthroughMonsters) {
                            true => 1.0,
                            false => 0.0,
                        }
                    },
                    1.0);
                path.find(player.pos.tuple(), dose_pos.tuple());
                if path.len() <= player_resist_radius(dose.irresistible, *player.will) {
                    path.walk_one_step(false)
                } else {
                    None
                }
            };
            if let Some(new_pos) = new_pos_opt {
                action = Action::Move(new_pos.into());
            } else {
                //println!("Can't find path to irresistable dose at {:?} from player's position {:?}.", dose_pos, player.pos);
            }
        }
        match action {
            Action::Move(dest) => {
                if level.within_bounds(dest) {
                    if let Some(monster_id) = level.monster_on_pos(dest) {
                        player.spend_ap(1);
                        let monster = &mut monsters[monster_id];
                        assert_eq!(monster.id(), monster_id);
                        //println!("Player attacks {:?}", monster);
                        kill_monster(monster, level);
                        match monster.kind {
                            monster::Kind::Anxiety => {
                                player.anxiety_counter += 1;
                                if *player.anxiety_counter == 10 {
                                    player.will += 1;
                                    player.anxiety_counter.set(0);
                                }
                            }
                            _ => {}
                        }
                    } else if level.walkable(dest, level::Walkability::BlockingMonsters) {
                        player.spend_ap(1);
                        player.move_to(dest);
                        loop {
                            match level.pickup_item(dest) {
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
                                                let anim = explode(player.pos, radius, level, monsters);
                                                *explosion_animation = anim;
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
                    let food = player.inventory.remove(food_idx);
                    player.take_effect(food.modifier);
                    let food_explosion_radius = 2;
                    let anim = explode(player.pos, food_explosion_radius, level, monsters);
                    *explosion_animation = anim;
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
                let newpos_opt = if pos.tile_distance(destination) <= 1 {
                    Some(destination)
                } else {
                    let point::Point{x: w, y: h} = level.size();
                    {   // Find path && walk one step:
                        let mut path = tcod::pathfinding::AStar::new_from_callback(
                            w, h,
                            |_from: (i32, i32), to: (i32, i32)| -> f32 {
                                if level.walkable(to, level::Walkability::BlockingMonsters) {
                                    1.0
                                } else {
                                    0.0
                                }
                            },
                            1.0);
                        path.find(pos.tuple(), destination.tuple());
                        assert!(path.len() != 1, "The path shouldn't be trivial. We already handled that.");
                        path.walk_one_step(false).map(|p| p.into())
                    }
                };
                monster.spend_ap(1);
                match newpos_opt {
                    Some(step) => {
                        if level.monster_on_pos(step).is_none() {
                            level.move_monster(monster, step);
                        } else if step == monster.position {
                            //println!("{:?} cannot move so it waits.", monster);
                        } else {
                            unreachable!();
                        }
                    }
                    None => {
                        //println!("{:?} can't find a path so it waits.", monster);
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


fn render_gui(x: i32, display: &mut engine::Display, state: &GameState, dt: Duration, fps: i32) {
    let fg = color::Color{r: 255, g: 255, b: 255};
    // TODO: set the background colour of the panel (or the rendered map)
    let bg = color::Color{r: 0, g: 0, b: 0};

    let player = &state.player;

    let (mind_str, mind_val_percent) = match player.mind {
        player::Mind::Withdrawal(val) => ("Withdrawal", val.percent()),
        player::Mind::Sober(val) => ("Sober", val.percent()),
        player::Mind::High(val) => ("High", val.percent()),
    };

    let mut lines = vec![
        // TODO: render the value as a bar here
        mind_str.into(),
        format!("{}%", mind_val_percent * 100.0),
        "".into(),
        format!("Will: {}", *player.will),
        format!("Food: {}", player.inventory.len()),
        "".into(),
    ];
    if state.cheating {
        lines.push("CHEATING".into());
        lines.push(format!("SoM: {:?}", player.mind));
        lines.push("".into());
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

    let bottom = display.size().y - 1;
    display.write_text(&format!("dt: {}ms", dt.num_milliseconds()), (x + 1, bottom - 1), fg, bg);
    display.write_text(&format!("FPS: {}", fps), (x + 1, bottom), fg, bg);

}


fn update(mut state: GameState, dt: Duration, engine: &mut engine::Engine) -> Option<GameState> {
    if engine.key_pressed(Key { printable: 'q', pressed: true, code: KeyCode::Char, .. Default::default() }) {
        return None;
    }
    if let Some(key) = engine.keys.pop_front() {
        if key.code == KeyCode::Enter && (key.left_alt || key.right_alt) {
            engine.toggle_fullscreen();
        } else {
            engine.keys.push_front(key);
        }
    }
    if engine.key_pressed(Key { code: KeyCode::F5, pressed: true, .. Default::default() }) {
        //println!("Restarting game");
        engine.keys.clear();
        let state = GameState::new_game(state.world_size, state.map_size, state.panel_width, state.display_size);
        return Some(state);
    }
    state.clock = state.clock + dt;

    if engine.key_pressed(Key { code: KeyCode::F6, pressed: true, .. Default::default() }) {
        state.cheating = !state.cheating;
        //println!("Cheating set to: {}", state.cheating);
    }

    state.paused = if state.replay && engine.read_key(KeyCode::Spacebar) {
        !state.paused
    } else {
        state.paused
    };

    let running = !state.paused && !state.replay;
    let paused_one_step = state.paused && engine.read_key(KeyCode::Right);
    let timed_step = if state.replay && !state.paused && state.clock.num_milliseconds() >= 50 {
        state.clock = Duration::zero();
        true
    } else {
        false
    };

    let previous_intoxication_state = state.player.mind;
    let player_was_alive = state.player.alive();

    // Animation to re-center the screen around the player when they
    // get too close to an edge.
    state.pos_timer.update(dt);
    if !state.pos_timer.finished() {
        let percentage = state.pos_timer.percentage_elapsed();
        let x = (((state.new_screen_pos.x - state.old_screen_pos.x) as f32) * percentage) as i32;
        let y = (((state.new_screen_pos.y - state.old_screen_pos.y) as f32) * percentage) as i32;
        //println!("percentage: {}, old: {:?}, final: {:?}; x, y: {}, {}", percentage, (oldx, oldy), (finalx, finaly), x, y);
        state.screen_position_in_world = state.old_screen_pos + (x, y);
    }

    if running || paused_one_step || timed_step {
        process_keys(&mut engine.keys, &mut state.commands);

        // Process player
        match state.side {
            Side::Player => {
                process_player(&mut state.player,
                               &mut state.commands,
                               &mut state.level,
                               &mut state.monsters,
                               &mut state.explosion_animation,
                               &mut state.rng,
                               &mut state.command_logger);
                state.level.explore(state.player.pos, exploration_radius(state.player.mind));

                // move screen if the player goes near the edge of the screen
                let screen_left_top_corner = state.screen_position_in_world - (state.display_size / 2);

                let display_pos = state.player.pos - screen_left_top_corner;
                if state.pos_timer.finished() {
                    let dur = Duration::milliseconds(400);
                    // TODO: move the screen roughly the same distance along X and Y
                    if display_pos.x <= 10 || display_pos.x >= state.display_size.x - 10 {
                            // change the screen centre to that of the player
                            state.pos_timer = Timer::new(dur);
                            state.old_screen_pos = state.screen_position_in_world;
                            state.new_screen_pos = (state.player.pos.x, state.old_screen_pos.y).into();
                    } else if display_pos.y <= 7 || display_pos.y >= state.display_size.y - 7 {
                            // change the screen centre to that of the player
                            state.pos_timer = Timer::new(dur);
                            state.old_screen_pos = state.screen_position_in_world;
                            state.new_screen_pos = (state.old_screen_pos.x, state.player.pos.y).into();
                    }
                }

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


    // Rendering & related code here:
    if state.player.alive() {
        use player::Mind::*;

        if previous_intoxication_state != state.player.mind {
            let was_high = match previous_intoxication_state {
                High(_) => true,
                _ => false,
            };
            let is_high = match state.player.mind {
                High(_) => true,
                _ => false,
            };

            if !was_high && is_high {
                // Set animation on each level's tile:
                for (pos, cell) in state.level.iter_mut() {
                    let dur_ms = 700 + (((pos.x * pos.y) % 100) as i64) * 5;
                    cell.tile.set_animation(graphics::Animation::ForegroundCycle{
                        from: color::high,
                        to: color::high_to,
                        duration: Duration::milliseconds(dur_ms),
                    });
                }
            } else if was_high && !is_high {
                // Stop animation on the level's tiles:
                for (_pos, cell) in state.level.iter_mut() {
                    cell.tile.set_animation(graphics::Animation::None);
                }
            } else {
                // NOTE: the animation is what it's supposed to be. Do nothing.
            }
        }


        // Fade when withdrawn:
        match state.player.mind {
            Withdrawal(value) => {
                let mut fade = value.percent() * 0.025 + 0.25;
                if fade < 0.25 {
                    fade = 0.25;
                }
                engine.display.fade(fade , color::Color{r: 0, g: 0, b: 0});
            }
            Sober(_) | High(_) => {
                // NOTE: Not withdrawn, don't fade
            }
        }

        // NOTE: Update the animation state of each tile:
        for (_, cell) in state.level.iter_mut() {
            cell.tile.update(dt);
        }
    } else if player_was_alive {  // NOTE: Player just died
        // Make sure we're not showing the High gfx effect when dead
        for (_pos, cell) in state.level.iter_mut() {
            cell.tile.set_animation(graphics::Animation::None);
        }
        state.screen_fading = Some(ScreenFadeAnimation::new(
            color::Color{r: 255, g: 0, b: 0},
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

    let screen_left_top_corner = state.screen_position_in_world - (state.map_size / 2, state.map_size / 2);

    let player_pos = state.player.pos;
    // NOTE: map is the displayed playable area. I.e. fits the screen
    // but doesn't include the side bar.
    let map_size = state.map_size;
    let within_map_bounds = |pos| pos >= (0, 0) && pos < (map_size, map_size);
    let in_fov = |pos| player_pos.distance(pos) < (radius as f32);
    let screen_coords_from_world = |pos| pos - screen_left_top_corner;

    // Render the level and items:
    for (world_pos, cell) in state.level.iter() {
        let display_pos = screen_coords_from_world(world_pos);
        if !within_map_bounds(display_pos) {
            continue;
        }
        // Render the tile
        if in_fov(world_pos) {
            graphics::draw(&mut engine.display, dt, display_pos, &cell.tile);
        } else if cell.explored || bonus == player::Bonus::UncoverMap {
            // TODO: need to supply the dark bg here?
            graphics::draw(&mut engine.display, dt, display_pos, &cell.tile);
            for item in cell.items.iter() {
                graphics::draw(&mut engine.display, dt, display_pos, item);
            }
            engine.display.set_background(display_pos, color::dim_background);
        }

        // Render the irresistible background of a dose
        for item in cell.items.iter() {
            if item.kind == item::Kind::Dose {
                let resist_radius = player_resist_radius(item.irresistible, *state.player.will);
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
    }

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
                if state.level.within_bounds(world_pos) {
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

    // TODO: assert no monster is on the same coords as the player
    // assert!(pos != self.player().coordinates(), "Monster can't be on the same cell as player.");
    for monster in state.monsters.iter().filter(|m| !m.dead) {
        let visible = monster.position.distance(state.player.pos) < (radius as f32);
        if visible || bonus == player::Bonus::UncoverMap || bonus == player::Bonus::SeeMonstersAndItems {
            let world_pos = monster.position;
            let display_pos = screen_coords_from_world(world_pos);
            if within_map_bounds(display_pos) {
                graphics::draw(&mut engine.display, dt, display_pos, monster);
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
    render_gui(state.map_size, &mut engine.display, &state, dt, fps);
    Some(state)
}


fn main() {
    // NOTE: at our current font, the height of 43 is the maximum value for
    // 1336x768 monitors.
    let map_size = 43;
    let panel_width = 20;
    let display_size = (map_size + panel_width, map_size).into();
    let world_size = (200, 200).into();
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
