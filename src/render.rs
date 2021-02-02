use crate::{
    color,
    engine::{Display, TextMetrics},
    formula, graphics, monster,
    player::Bonus,
    point::{Point, SquareArea},
    rect::Rectangle,
    settings::Settings,
    state::State,
    world::Chunk,
};

pub fn render_game(
    state: &State,
    settings: &Settings,
    metrics: &dyn TextMetrics,
    display: &mut Display,
    highlighted_tile: Option<Point>,
) {
    let mut offset_px = state.offset_px;

    // NOTE: Transfer `offset_px` into the screen position in the
    // world, and only keep the sub-tile remainder for smooth
    // scrolling.
    //
    // We do this to select the display area correctly. Even if we
    // scroll far away from the player, we still want to show the map
    // if it exists there.
    let mut screen_position_in_world = state.screen_position_in_world;
    screen_position_in_world.x -= offset_px.x / display.tile_size;
    screen_position_in_world.y -= offset_px.y / display.tile_size;
    offset_px.x = offset_px.x % display.tile_size;
    offset_px.y = offset_px.y % display.tile_size;

    if let Some(ref animation) = state.screen_fading {
        use crate::animation::ScreenFadePhase;
        let fade = match animation.phase {
            ScreenFadePhase::FadeOut => animation.timer.percentage_remaining(),
            ScreenFadePhase::Wait => 0.0,
            ScreenFadePhase::FadeIn => animation.timer.percentage_elapsed(),
            ScreenFadePhase::Done => 1.0,
        };
        display.set_fade(animation.color, fade);
    }

    let uncovered_map = (cfg!(feature = "cheating") && state.cheating)
        || state.player.bonus == Bonus::UncoverMap  // player bonus
        || state.uncovered_map  // map uncovered after the endgame fade
        || !settings.hide_unseen_tiles; // challenge Settings option

    let radius = formula::exploration_radius(state.player.mind);

    let player_pos = state.player.pos;
    let in_fov = |pos| player_pos.distance(pos) < (radius as f32);
    let screen_left_top_corner = screen_position_in_world - (state.map_size / 2);
    let display_area = Rectangle::center(screen_position_in_world, state.map_size);
    let screen_coords_from_world = |pos| pos - screen_left_top_corner;

    let total_time_ms = state.clock.as_millis() as i64;
    let world_size = state.world_size;

    let player_will = state.player.will.to_int();
    // NOTE: this is here to appease the borrow checker. If we
    // borrowed the state here as immutable, we wouln't need it.
    let show_intoxication_effect = state.player.alive() && state.player.mind.is_high();

    if state.player.alive() {
        let fade = formula::mind_fade_value(state.player.mind);
        display.set_fade(color::BLACK, fade);
    }

    // TODO: I THIN THIS FIXED IT \O/
    // UNCOMMENT ALL THE OTHER STUFF AND VERIFY
    display.offset_px = offset_px;

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

        if show_intoxication_effect && rendered_tile.kind != crate::level::TileKind::Empty {
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

        if in_fov(world_pos) || cell.always_visible || state.uncovered_map {
            display.set(
                display_pos,
                rendered_tile.graphic,
                rendered_tile.fg_color,
                color::explored_background,
            );
        } else if cell.explored || uncovered_map {
            display.set(
                display_pos,
                rendered_tile.graphic,
                rendered_tile.fg_color,
                color::dim_background,
            );
        } else {
            // It's not visible. Do nothing.
        }

        // Render the items
        if in_fov(world_pos)
            || cell.explored
            || cell.always_visible
            || state.player.bonus == Bonus::SeeMonstersAndItems
            || uncovered_map
        {
            for item in &cell.items {
                display.set_graphic(display_pos, item.graphic(), item.color());
            }
        }
    }

    for (world_pos, cell) in state
        .world
        .chunks(display_area)
        .flat_map(Chunk::cells)
        .filter(|&(pos, _)| display_area.contains(pos))
    {
        // Render the irresistible background of a dose
        for item in &cell.items {
            if item.is_dose() {
                let resist_radius = formula::player_resist_radius(item.irresistible, player_will);
                for point in SquareArea::new(world_pos, resist_radius) {
                    let cell_visible = state
                        .world
                        .cell(point)
                        .map_or(false, |cell| cell.always_visible);
                    if in_fov(point) || cell_visible || (state.game_ended && state.uncovered_map) {
                        let screen_coords = screen_coords_from_world(point);
                        display.set_background(screen_coords, color::dose_irresistible_background);
                    }
                }
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
        let monster_visible = monster.position.distance(state.player.pos) < (radius as f32);
        let cell_visible = state
            .world
            .cell(monster.position)
            .map_or(false, |cell| cell.always_visible);
        if monster_visible
            || monster.accompanying_player
            || cell_visible
            || uncovered_map
            || state.player.bonus == Bonus::SeeMonstersAndItems
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

            let color = if monster.kind == monster::Kind::Npc && state.player.mind.is_high() {
                color::npc_dim
            } else {
                monster.color
            };
            display.set_graphic(display_pos, monster.graphic(), color);
        }
    }

    // NOTE: render the player
    {
        let display_pos = screen_coords_from_world(state.player.pos);
        display.set_graphic(display_pos, state.player.graphic(), state.player.color());
    }

    // Highlight the target tile the player would walk to if clicked in the sidebar numpad:
    if let Some(pos) = highlighted_tile {
        // Only highlight when we're not re-centering the
        // screen (because that looks weird)
        if state.pos_timer.finished() {
            display.set_background(pos, state.player.color);
        }
    }

    if state.show_keboard_movement_hints && !state.game_ended {
        render_controls_help(state.map_size, metrics, display);
    }

    let mouse_inside_map = state.mouse.tile_pos >= (0, 0) && state.mouse.tile_pos < state.map_size;
    if mouse_inside_map && state.mouse.right_is_down {
        render_monster_info(state, display);
    }
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
            color::window_background,
        );
        for (index, line) in debug_text.lines().enumerate() {
            display.draw_text_in_tile_coordinates(
                Point {
                    x: 0,
                    y: index as i32,
                },
                line,
                color::gui_text,
                Default::default(),
                display.tile_size,
            );
        }
    }
}

fn render_controls_help(map_size: Point, metrics: &dyn TextMetrics, display: &mut Display) {
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
            display.draw_text_in_tile_coordinates(
                start + Point::new(0, index as i32),
                line,
                color::gui_text,
                Default::default(),
                display.tile_size,
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
