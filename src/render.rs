use crate::{
    animation::{self, MoveState},
    color,
    engine::{Display, OffsetTile, TextMetrics},
    formula, graphics, monster,
    player::Bonus,
    point::{Point, SquareArea},
    rect::Rectangle,
    state::{GameSession, State},
    world::Chunk,
};

pub fn render_move_animation(
    animation: &animation::Move,
    display_pos: Point,
    display: &mut Display,
) {
    let cell_offset = animation.current_offset_px();

    // NOTE: yep, this is pretty fucking ugly.
    //
    // Since the actual player movement is immediate but the
    // animation takes time, there are times when the offset e.g.
    // refers to the player's original position but the
    // `player.pos` value is now the animation's destination.
    //
    // Since the offset is relative, this can cause weird jumps
    // and rendering the player in a wrong location.
    //
    // This `fixup` fudges the offset to do the right thing
    // visually.
    //
    // TODO: can we get rid of this hack and do it more cleanly?
    // If the animation had an absolute value instead of a
    // relative, this wouldn't be necessary.
    let fixup = match (animation.state, animation.bounce) {
        (MoveState::There, true) => Point::zero(),
        (MoveState::Back, true) => animation.source - animation.destination,
        (MoveState::Finished, true) => animation.source - animation.destination,
        (_, false) => animation.source - animation.destination,
    };
    if !animation.finished() {
        display.set_offset(display_pos, cell_offset + fixup);
    }
}

pub fn render_game(
    state: &State,
    _metrics: &dyn TextMetrics,
    display: &mut Display,
    highlighted_tiles: Vec<Point>,
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
    offset_px.x %= display.tile_size;
    offset_px.y %= display.tile_size;

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
        || !state.challenge.hide_unseen_tiles; // challenge Settings option

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
    // borrowed the state here as immutable, we wouldn't need it.
    let show_intoxication_effect = state.player.alive() && state.player.mind.is_high();

    if state.player.alive() && state.screen_fading.is_none() {
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

        let mut fg_color = cell.tile.color(&state.palette);

        if show_intoxication_effect && cell.tile.kind != crate::level::TileKind::Empty {
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

            fg_color = if forwards {
                graphics::fade_color(state.palette.high, state.palette.high_to, progress)
            } else {
                graphics::fade_color(state.palette.high_to, state.palette.high, progress)
            };
        }

        if in_fov(world_pos) || cell.always_visible || state.uncovered_map {
            display.set_cell(
                display_pos,
                cell.tile.graphic,
                fg_color,
                state.palette.explored_background,
            );
        } else if cell.explored || uncovered_map {
            display.set_cell(
                display_pos,
                cell.tile.graphic,
                fg_color,
                state.palette.dim_background,
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
                display.set_foreground_graphic(
                    display_pos,
                    item.graphic(),
                    item.color(&state.palette),
                );
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
                    if in_fov(point)
                        || cell_visible
                        || (state.game_session == GameSession::Ended && state.uncovered_map)
                    {
                        let screen_coords = screen_coords_from_world(point);
                        display.set_empty_color(
                            screen_coords,
                            state.palette.dose_irresistible_background,
                        );
                    }
                }
            }
        }
    }

    if let Some(ref animation) = state.explosion_animation {
        for (world_pos, color, _) in animation.tiles() {
            display.set_empty_color(screen_coords_from_world(world_pos), color);
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
                state.palette.npc_dim
            } else {
                monster.color(&state.palette)
            };
            display.push_fg_to_bg(display_pos);
            display.set_foreground_graphic(display_pos, monster.graphic(), color);

            render_move_animation(&monster.motion_animation, display_pos, display);
        }
    }

    // NOTE: Render the extra animations
    {
        for animation in &state.extra_animations {
            let tile = OffsetTile {
                pos: screen_coords_from_world(animation.pos),
                graphic: animation.graphic,
                color: animation.color,
                offset_px: animation.animation.current_offset_px(),
            };
            display.offset_tiles.push(tile);
        }
    }

    // NOTE: render the player
    {
        let display_pos = screen_coords_from_world(state.player.pos);
        display.push_fg_to_bg(display_pos);
        display.set_foreground_graphic(
            display_pos,
            state.player.graphic(),
            state.player.color(&state.palette),
        );

        render_move_animation(&state.player.motion_animation, display_pos, display);
    }

    // Highlight the tiles the player would walk to if clicked in the
    // sidebar numpad or followed the pathfinding suggestion:
    for pos in highlighted_tiles {
        // Only highlight when we're not re-centering the
        // screen (because that looks weird)
        if state.pos_timer.finished() {
            display.set_empty_color(pos, state.player.color(&state.palette));
        }
    }
}
