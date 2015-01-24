#![deny(overflowing_literals)]
#![allow(unstable)]

extern crate libc;
extern crate time;
extern crate tcod;
extern crate "tcod-sys" as tcod_ffi;


use std::collections::RingBuf;
use std::os;
use std::rand::Rng;
use std::time::Duration;

use tcod::KeyState;
use tcod::Key::{Printable, Special};

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


#[derive(Copy)]
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
}

#[derive(Copy)]
pub struct ScreenFadeAnimation {
    pub color: Color,
    pub fade_out_time: Duration,
    pub wait_time: Duration,
    pub fade_in_time: Duration,
    pub timer: Timer,
    pub phase: ScreenFadePhase,
}

#[derive(Copy, PartialEq)]
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


#[derive(Copy, PartialEq, Show)]
pub enum Action {
    Move((i32, i32)),
    Attack((i32, i32), player::Modifier),
    Eat,
}


fn kill_monster(monster: &mut monster::Monster, level: &mut level::Level) {
    monster.dead = true;
    level.remove_monster(monster.id(), monster);
}

// TODO: prolly refactor to a struct?
// Fields: position, max radius, current radius, colour, elapsed time
type ExplosionAnimation = Option<((i32, i32), i32, i32, color::Color, Duration)>;

fn explode(center: point::Point,
           radius: i32,
           level: &mut level::Level,
           monsters: &mut Vec<monster::Monster>) -> ExplosionAnimation {
    for pos in point::points_within_radius(center, radius) {
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

fn exploration_radius(state_of_mind: i32) -> i32 {
    use player::IntoxicationState::*;
    match player::IntoxicationState::from_int(state_of_mind) {
        Exhausted | DeliriumTremens => 4,
        Withdrawal => 5,
        Sober => 6,
        High => 7,
        VeryHigh | Overdosed => 8
    }
}


fn process_player<R: Rng>(player: &mut player::Player,
                          commands: &mut RingBuf<Command>,
                          level: &mut level::Level,
                          monsters: &mut Vec<monster::Monster>,
                          explosion_animation: &mut ExplosionAnimation,
                          rng: &mut R,
                          command_logger: &mut game_state::CommandLogger) {
    if !player.alive() {
        return
    }

    if let Some(command) = commands.pop_front() {
        command_logger.log(command);
        let (x, y) = player.pos;
        let mut action = match command {
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
        if *player.stun > 0 {
            action = Action::Move(player.pos);
        } else if *player.panic > 0 {
            let new_pos = level.random_neighbour_position(
                rng, player.pos, level::Walkability::WalkthroughMonsters);
            action = Action::Move(new_pos);
        } else if let Some((dose_pos, dose)) = level.nearest_dose(player.pos, 5) {
            let new_pos_opt = {
                use std::cmp;
                let (w, h) = level.size();
                let mut path = tcod::AStarPath::new_from_callback(
                    w, h,
                    |&mut: _from: point::Point, to: point::Point| -> f32 {
                        match level.walkable(to, level::Walkability::WalkthroughMonsters) {
                            true => 1.0,
                            false => 0.0,
                        }
                    },
                    1.0);
                path.find(player.pos, dose_pos);
                let player_resist_radius = cmp::max(dose.irresistible - *player.will, 0);
                if path.len() <= player_resist_radius {
                    path.walk_one_step(false)
                } else {
                    None
                }
            };
            if let Some(new_pos) = new_pos_opt {
                action = Action::Move(new_pos);
            } else {
                println!("Can't find path to irresistable dose at {:?} from player's position {:?}.",
                         dose_pos, player.pos);
            }
        }
        match action {
            Action::Move((x, y)) => {
                let (w, h) = level.size();
                let within_level = (x >= 0) && (y >= 0) && (x < w) && (y < h);
                if within_level {
                    if let Some(monster_id) = level.monster_on_pos((x, y)) {
                        player.spend_ap(1);
                        let monster = &mut monsters[monster_id];
                        assert_eq!(monster.id(), monster_id);
                        println!("Player attacks {:?}", monster);
                        kill_monster(monster, level);
                        match monster.kind {
                            monster::Kind::Anxiety => {
                                player.anxiety_counter.add(1);
                                if *player.anxiety_counter == 10 {
                                    player.will.add(1);
                                    player.anxiety_counter.set(0);
                                }
                            }
                            _ => {}
                        }
                    } else if level.walkable((x, y), level::Walkability::BlockingMonsters) {
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
                let newpos_opt = if point::tile_distance(pos, destination) <= 1 {
                    Some(destination)
                } else {
                    let (w, h) = level.size();
                    {   // Find path && walk one step:
                        let mut path = tcod::AStarPath::new_from_callback(
                            w, h,
                            |&mut: _from: (i32, i32), to: (i32, i32)| -> f32 {
                                if level.walkable(to, level::Walkability::BlockingMonsters) {
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
                            println!("{:?} cannot move so it waits.", monster);
                        } else {
                            unreachable!();
                        }
                    }
                    None => {
                        println!("{:?} can't find a path so it waits.", monster);
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
                              *player.state_of_mind,
                              *player.will,
                              player.inventory.len());
    display.write_text(attribute_line.as_slice(), 0, h-1,
                       color::Color{r: 255, g: 255, b: 255},
                       color::Color{r: 0, g: 0, b: 0});

    let mut status_line = String::new();
    if player.alive() {
        if *player.stun > 0 {
            status_line.push_str(format!("Stunned({})", *player.stun).as_slice());
        }
        if *player.panic > 0 {
            if status_line.len() > 0 {
                status_line.push_str(",  ");
            }
            status_line.push_str(format!("Panicking({})", *player.panic).as_slice())
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
    if let Some(key) = engine.keys.pop_front() {
        if key.key == Special(KeyCode::Enter) && (key.left_alt || key.right_alt) {
            // TODO: add fullscreen support to tcod-rs
            unsafe {
                let fullscreen = tcod_ffi::TCOD_console_is_fullscreen() != 0;
                tcod_ffi::TCOD_console_set_fullscreen(!fullscreen as u8);
            }
        } else {
            engine.keys.push_front(key);
        }
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

    let previous_intoxication_state = player::IntoxicationState::from_int(
        *state.player.state_of_mind);
    let player_was_alive = state.player.alive();


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
                state.level.explore(state.player.pos, exploration_radius(*state.player.state_of_mind));
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


    let som = *state.player.state_of_mind;
    let current_intoxication_state = player::IntoxicationState::from_int(som);

    // Rendering & related code here:
    if state.player.alive() {
        use player::IntoxicationState::*;

        if previous_intoxication_state != current_intoxication_state {
            let was_high = match previous_intoxication_state {
                High | VeryHigh => true,
                _ => false,
            };
            let is_high = match current_intoxication_state {
                High | VeryHigh => true,
                _ => false,
            };

            if !was_high && is_high {
                // Set animation on each level's tile:
                for ((x, y), cell) in state.level.iter_mut() {
                    let dur_ms = 700 + (((x * y) % 100) as i64) * 5;
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
        match current_intoxication_state {
            DeliriumTremens | Withdrawal => {
                // NOTE: SoM is <0, 100>, this turns it into percentage <0, 100>
                let som_percent = (som as f32) / 100.0;
                let mut fade = som_percent * 0.025 + 0.25;
                if fade < 0.25 {
                    fade = 0.25;
                }
                engine.display.fade(fade , color::Color{r: 0, g: 0, b: 0});
            }
            Exhausted | Sober | Overdosed | High | VeryHigh => {
                // NOTE: Not withdrawn, don't fade
            }
        }

        // NOTE: Update the animation state of each tile:
        for (_, cell) in state.level.iter_mut() {
            cell.tile.update(dt);
        }
    } else if player_was_alive {  // NOTE: Player just died
        // Make sure we're not showing the High gfx effect when dead
        if current_intoxication_state != previous_intoxication_state {
            for (_pos, cell) in state.level.iter_mut() {
                cell.tile.set_animation(graphics::Animation::None);
            }
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
    let radius = exploration_radius(*state.player.state_of_mind);

    // Render the level and items:
    for ((x, y), cell) in state.level.iter() {
        let in_fov = point::distance((x, y), state.player.pos) < (radius as f32);

        // Render the tile
        if in_fov {
            graphics::draw(&mut engine.display, dt, (x, y), &cell.tile);
        } else if cell.explored || bonus == player::Bonus::UncoverMap {
            // TODO: need to supply the dark bg here?
            graphics::draw(&mut engine.display, dt, (x, y), &cell.tile);
            for item in cell.items.iter() {
                graphics::draw(&mut engine.display, dt, (x, y), item);
            }
            engine.display.set_background(x, y, color::dim_background);
        }

        // Render the items
        if in_fov || cell.explored || bonus == player::Bonus::SeeMonstersAndItems || bonus == player::Bonus::UncoverMap {
            for item in cell.items.iter() {
                graphics::draw(&mut engine.display, dt, (x, y), item);
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
            for (x, y) in point::points_within_radius(center, r) {
                if state.level.within_bounds((x, y)) {
                    engine.display.set_background(x, y, c);
                }
            }
        } else {
            state.explosion_animation = None;
        }

    }

    // TODO: assert no monster is on the same coords as the player
    // assert!(pos != self.player().coordinates(), "Monster can't be on the same cell as player.");
    for monster in state.monsters.iter().filter(|m| !m.dead) {
        let visible = point::distance(monster.position, state.player.pos) < (radius as f32);
        if visible || bonus == player::Bonus::UncoverMap || bonus == player::Bonus::SeeMonstersAndItems {
            graphics::draw(&mut engine.display, dt, monster.position, monster);
        }
    }
    graphics::draw(&mut engine.display, dt, state.player.pos, &state.player);
    render_gui(&mut engine.display, &state.player);
    Some(state)
}



fn main() {
    let (width, height) = (80, 50);
    let title = "Dose Response";
    let font_path = Path::new("./fonts/dejavu16x16_gs_tc.png");

    let game_state = match os::args().len() {
        1 => {  // Run the game with a new seed, create the replay log
            // TODO: directory creation is unix-specific because permissions.
            // This should probably be taken out of GameState and moved here or
            // to some platform-specific layer.
            GameState::new_game(width, height)
        },
        2 => {  // Replay the game from the entered log
            GameState::replay_game(width, height)
        },
        _ => panic!("You must pass either pass zero or one arguments."),
    };

    let (screen_width, screen_height) = tcod::system::get_current_resolution();
    println!("Current resolution: ({}, {})", screen_width, screen_height);
    // TODO: check the screen_width/screen_height values against known
    // (supported?) monitor resolutions. Only force fullscreen res if it's
    // one of the known ones.
    tcod::system::force_fullscreen_resolution(screen_width, screen_height);

    let mut engine = Engine::new(width, height, title, font_path.clone());
    tcod::RootConsole.set_default_background(color::background);
    engine.main_loop(game_state, update);
}
