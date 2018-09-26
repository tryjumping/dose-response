use color;
use engine::{Display, TextMetrics, TextOptions};
use formula;
use graphics;
use monster;
use player::Bonus;
use point::{Point, SquareArea};
use rect::Rectangle;
use state::{State, Window};
use util;
use windows::{endgame, help, main_menu, sidebar};
use world::Chunk;

use std::time::Duration;

pub fn render(state: &State, dt: Duration, fps: i32, metrics: &TextMetrics, display: &mut Display) {
    // NOTE: Clear the screen
    display.clear(color::background);

    // TODO: This might be inefficient for windows fully covering
    // other windows.
    for window in state.window_stack.windows() {
        match window {
            Window::MainMenu => {
                render_main_menu(state, &main_menu::Window, metrics, display);
            }
            Window::Game => {
                render_game(state, &sidebar::Window, metrics, dt, fps, display);
            }
            Window::Help => {
                render_help_screen(state, &help::Window, metrics, display);
            }
            Window::Endgame => {
                render_endgame_screen(state, &endgame::Window, metrics, display);
            }
            Window::Message(ref text) => {
                render_message(state, text, metrics, display);
            }
        }
    }

    // NOTE: This renders the game's icon. Change the tilesize to an
    // appropriate value.
    //
    // let origin = Point::new(15, 15);
    // drawcalls.push(Draw::Char(origin, 'D', color::depression));
    // drawcalls.push(Draw::Char(origin + (1, 0), 'r', color::anxiety));
    // drawcalls.push(Draw::Char(origin + (0, 1), '@', color::player));
    // drawcalls.push(Draw::Char(origin + (1, 1), 'i', color::dose));
    // drawcalls.push(Draw::Fade(1.0, color::BLACK));

    // Show the tile under mouse pointer:
    // drawcalls.push(Draw::Rectangle(::rect::Rectangle::from_point_and_size(state.mouse.tile_pos, Point::from_i32(1)), color::gui_text));
}

pub fn render_game(
    state: &State,
    sidebar_window: &sidebar::Window,
    metrics: &TextMetrics,
    dt: Duration,
    fps: i32,
    display: &mut Display,
) {
    let offset_px = state.offset_px;

    if let Some(ref animation) = state.screen_fading {
        use animation::ScreenFadePhase;
        let fade = match animation.phase {
            ScreenFadePhase::FadeOut => animation.timer.percentage_remaining(),
            ScreenFadePhase::Wait => 0.0,
            ScreenFadePhase::FadeIn => animation.timer.percentage_elapsed(),
            ScreenFadePhase::Done => 1.0,
        };
        display.set_fade(animation.color, fade);
    }

    let bonus = if cfg!(feature = "cheating") && state.cheating {
        // TODO: setting this as a bonus is a hack. Pass it to all renderers
        // directly instead.
        Bonus::UncoverMap
    } else {
        state.player.bonus
    };
    let radius = formula::exploration_radius(state.player.mind);

    let player_pos = state.player.pos;
    let in_fov = |pos| player_pos.distance(pos) < (radius as f32);
    let screen_left_top_corner = state.screen_position_in_world - (state.map_size / 2);
    let display_area = Rectangle::center(state.screen_position_in_world, state.map_size);
    let screen_coords_from_world = |pos| pos - screen_left_top_corner;

    let total_time_ms = util::num_milliseconds(state.clock) as i64;
    let world_size = state.world_size;

    let player_will = state.player.will.to_int();
    // NOTE: this is here to appease the borrow checker. If we
    // borrowed the state here as immutable, we wouln't need it.
    let show_intoxication_effect = state.player.alive() && state.player.mind.is_high();

    if state.player.alive() {
        let fade = formula::mind_fade_value(state.player.mind);
        display.set_fade(color::BLACK, fade);
    }

    // NOTE: render the cells on the display. That means world geometry and items.
    for (world_pos, cell) in state
        .world
        .chunks(display_area)
        .flat_map(Chunk::cells)
        .filter(|&(pos, _)| display_area.contains(pos))
    {
        let display_pos = screen_coords_from_world(world_pos);

        // Render the tile
        let mut rendered_tile = cell.tile;

        if show_intoxication_effect {
            // TODO: try to move this calculation of this loop and see
            // what it does to our speed.
            let pos_x: i64 = i64::from(world_pos.x + world_size.x);
            let pos_y: i64 = i64::from(world_pos.y + world_size.y);
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

        if in_fov(world_pos) || state.uncovered_map {
            display.set_glyph(
                display_pos,
                rendered_tile.glyph(),
                rendered_tile.fg_color,
                offset_px,
            );
        } else if cell.explored || bonus == Bonus::UncoverMap {
            display.set(
                display_pos,
                rendered_tile.glyph(),
                rendered_tile.fg_color,
                color::dim_background,
                offset_px,
            );
        } else {
            // It's not visible. Do nothing.
        }

        // Render the irresistible background of a dose
        for item in &cell.items {
            if item.is_dose() {
                let resist_radius = formula::player_resist_radius(item.irresistible, player_will);
                for point in SquareArea::new(world_pos, resist_radius) {
                    if in_fov(point) || (state.game_ended && state.uncovered_map) {
                        let screen_coords = screen_coords_from_world(point);
                        display.set_background(screen_coords, color::dose_irresistible_background);
                    }
                }
            }
        }

        // Render the items
        if in_fov(world_pos)
            || cell.explored
            || bonus == Bonus::SeeMonstersAndItems
            || bonus == Bonus::UncoverMap
            || state.uncovered_map
        {
            for item in &cell.items {
                display.set_glyph(display_pos, item.glyph(), item.color(), offset_px);
            }
        }
    }

    if let Some(ref animation) = state.explosion_animation {
        for (world_pos, color, _) in animation.tiles() {
            display.set_background(screen_coords_from_world(world_pos), color);
        }
    }

    // NOTE: render monsters
    for monster in state.world.monsters(display_area) {
        let visible = monster.position.distance(state.player.pos) < (radius as f32);
        if visible
            || bonus == Bonus::UncoverMap
            || bonus == Bonus::SeeMonstersAndItems
            || state.uncovered_map
        {
            let display_pos = screen_coords_from_world(monster.position);
            // NOTE: this is the monster trail. It's looking bad and
            // really confusing, so we turned it off.
            // if let Some(trail_pos) = monster.trail {
            //     if cfg!(feature = "cheating") && state.cheating {
            //         let trail_pos = screen_coords_from_world(trail_pos);
            //         let glyph = monster.glyph();
            //         let color = monster.color;
            //         // TODO: show a fading animation of the trail colour
            //         let color = color::Color {
            //             r: color.r.saturating_sub(55),
            //             g: color.g.saturating_sub(55),
            //             b: color.b.saturating_sub(55),
            //         };
            //         drawcalls.push(Draw::Char(trail_pos, glyph, color, offset_px));
            //     }
            // }

            // if cfg!(feature = "cheating") && state.cheating {
            //     for &point in &monster.path {
            //         let path_pos = screen_coords_from_world(point);
            //         let color = monster.color;
            //         drawcalls.push(Draw::Background(path_pos, color));
            //     }
            // }

            let glyph = monster.glyph();
            let mut color = if monster.kind == monster::Kind::Npc && state.player.mind.is_high() {
                color::npc_dim
            } else {
                monster.color
            };
            display.set_glyph(display_pos, glyph, color, offset_px);
        }
    }

    // NOTE: render the player
    {
        let display_pos = screen_coords_from_world(state.player.pos);
        display.set_glyph(
            display_pos,
            state.player.glyph(),
            state.player.color(),
            offset_px,
        );
    }

    sidebar_window.render(state, metrics, dt, fps, display);
    if state.show_keboard_movement_hints && !state.game_ended {
        render_controls_help(state.map_size, metrics, display);
    }

    let mouse_inside_map = state.mouse.tile_pos >= (0, 0) && state.mouse.tile_pos < state.map_size;
    if mouse_inside_map && state.mouse.right_is_down {
        render_monster_info(state, display);
    }
}

fn render_main_menu(
    state: &State,
    window: &main_menu::Window,
    metrics: &TextMetrics,
    display: &mut Display,
) {
    window.render(state, metrics, display);

    // Clear any fade set by the gameplay rendering
    display.fade = color::invisible;
}

fn render_help_screen(
    state: &State,
    window: &help::Window,
    metrics: &TextMetrics,
    display: &mut Display,
) {
    window.render(state, metrics, display);

    // Clear any fade set by the gameplay rendering
    display.fade = color::invisible;
}

fn render_endgame_screen(
    state: &State,
    window: &endgame::Window,
    metrics: &TextMetrics,
    display: &mut Display,
) {
    window.render(state, metrics, display);

    // Clear any fade set by the gameplay rendering
    display.fade = color::invisible;
}

fn render_message(state: &State, text: &str, _metrics: &TextMetrics, display: &mut Display) {
    let window_size = Point::new(40, 10);
    let window_pos = ((state.display_size - window_size) / 2) - (0, 10);
    let window_rect = Rectangle::from_point_and_size(window_pos, window_size);

    let padding = Point::new(2, 3);
    let rect = Rectangle::new(
        window_rect.top_left() + padding,
        window_rect.bottom_right() - padding,
    );

    display.draw_rectangle(window_rect, color::window_edge);

    display.draw_rectangle(
        Rectangle::new(
            window_rect.top_left() + (1, 1),
            window_rect.bottom_right() - (1, 1),
        ),
        color::background,
    );

    display.draw_text(
        rect.top_left(),
        text,
        color::gui_text,
        TextOptions::align_center(rect.width()),
    );
}

fn render_monster_info(state: &State, display: &mut Display) {
    let screen_left_top_corner = state.screen_position_in_world - (state.map_size / 2);
    let mouse_world_pos = screen_left_top_corner + state.mouse.tile_pos;
    // TODO: world.monster_on_pos is mutable, let's add an immutable version
    let monster_area = Rectangle::from_point_and_size(mouse_world_pos, (1, 1).into());
    let mut debug_text = None;
    for monster in state.world.monsters(monster_area) {
        if monster.position == mouse_world_pos {
            debug_text = Some(format!("{:#?}", monster));
        }
    }
    if mouse_world_pos == state.player.pos {
        debug_text = Some(format!("{:#?}", state.player));
    }

    if let Some(debug_text) = debug_text {
        let height = debug_text.lines().count();
        let width = debug_text.lines().map(|s| s.chars().count()).max().unwrap();
        display.draw_rectangle(
            Rectangle::from_point_and_size(
                Point::from_i32(0),
                Point::new(width as i32, height as i32),
            ),
            color::background,
        );
        for (index, line) in debug_text.lines().enumerate() {
            display.draw_text(
                Point {
                    x: 0,
                    y: index as i32,
                },
                line,
                color::gui_text,
                Default::default(),
            );
        }
    }
}

fn render_controls_help(map_size: Point, metrics: &TextMetrics, display: &mut Display) {
    let rect_dim = |lines: &[&str]| {
        let longest_line = lines
            .iter()
            .map(|l| metrics.get_text_width(l, Default::default()))
            .max()
            .unwrap();
        (longest_line, lines.len() as i32)
    };

    fn draw_rect(lines: &[&'static str], start: Point, w: i32, h: i32, display: &mut Display) {
        display.draw_rectangle(
            Rectangle::from_point_and_size(start, Point::new(w, h)),
            color::dim_background,
        );
        for (index, &line) in lines.iter().enumerate() {
            display.draw_text(
                start + Point::new(0, index as i32),
                line,
                color::gui_text,
                Default::default(),
            );
        }
    };

    let padding = 1;

    let lines = &["Up", "Num 8", "or: K"];
    let (width, height) = rect_dim(lines);
    let start = Point {
        x: (map_size.x - width) / 2,
        y: padding,
    };
    draw_rect(lines, start, width, height, display);

    let lines = &["Down", "Num 2", "or: J"];
    let (width, height) = rect_dim(lines);
    let start = Point {
        x: (map_size.x - width) / 2,
        y: map_size.y - height - padding,
    };
    draw_rect(lines, start, width, height, display);

    let lines = &["Left", "Num 4", "or: H"];
    let (width, height) = rect_dim(lines);
    let start = Point {
        x: padding,
        y: (map_size.y - height) / 2,
    };
    draw_rect(lines, start, width, height, display);

    let lines = &["Right", "Num 6", "or: L"];
    let (width, height) = rect_dim(lines);
    let start = Point {
        x: map_size.x - width - padding,
        y: (map_size.y - height) / 2,
    };
    draw_rect(lines, start, width, height, display);

    let lines = &["Shift+Left", "Num 7", "or: Y"];
    let (width, height) = rect_dim(lines);
    let start = Point {
        x: padding,
        y: padding,
    };
    draw_rect(lines, start, width, height, display);

    let lines = &["Shift+Right", "Num 9", "or: U"];
    let (width, height) = rect_dim(lines);
    let start = Point {
        x: map_size.x - width - padding,
        y: padding,
    };
    draw_rect(lines, start, width, height, display);

    let lines = &["Ctrl+Left", "Num 1", "or: B"];
    let (width, height) = rect_dim(lines);
    let start = Point {
        x: padding,
        y: map_size.y - height - padding,
    };
    draw_rect(lines, start, width, height, display);

    let lines = &["Ctrl+Right", "Num 3", "or: N"];
    let (width, height) = rect_dim(lines);
    let start = Point {
        x: map_size.x - width - padding,
        y: map_size.y - height - padding,
    };
    draw_rect(lines, start, width, height, display);
}
