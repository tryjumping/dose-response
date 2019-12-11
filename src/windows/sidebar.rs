use crate::color;
use crate::engine::{Display, TextMetrics};
use crate::game;
use crate::graphics;
use crate::item;
use crate::player::Mind;
use crate::point::Point;
use crate::rect::Rectangle;
use crate::state::State;
use crate::ui::Button;

use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Copy, Clone)]
pub enum Action {
    MainMenu,
    Help,
    UseFood,
    UseDose,
    UseCardinalDose,
    UseDiagonalDose,
    UseStrongDose,

    MoveN,
    MoveS,
    MoveW,
    MoveE,

    MoveNW,
    MoveNE,
    MoveSW,
    MoveSE,
}

struct Layout {
    x: i32,
    bottom: i32,
    fg: color::Color,
    bg: color::Color,
    mind_pos: Point,
    progress_bar_pos: Point,
    stats_pos: Point,
    inventory_pos: Point,
    inventory: HashMap<item::Kind, i32>,
    main_menu_button: Button,
    help_button: Button,

    n_button: Button,
    s_button: Button,
    w_button: Button,
    e_button: Button,

    nw_button: Button,
    ne_button: Button,
    sw_button: Button,
    se_button: Button,

    action_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
    rect2_under_mouse: Option<Rectangle>,
}

pub struct Window;

impl Window {
    fn layout(&self, state: &State, metrics: &dyn TextMetrics, display: &Display) -> Layout {
        let wide = state.panel_width > 16;
        let tall = display.size_without_padding().y > 31;
        let x = state.map_size.x;
        let fg = color::gui_text;
        let bg = color::dim_background;

        let left_padding = if wide { 1 } else { 0 };
        let mind_pos = Point::new(x + left_padding, 0);
        let progress_bar_pos = {
            let top = if tall { 1 } else { 0 };
            Point::new(x + left_padding, top)
        };

        let stats_pos = {
            let top = if tall { 3 } else { 1 };
            Point::new(x + left_padding, top)
        };
        let inventory_pos = {
            let top = if tall { 5 } else { 3 };
            Point::new(x + left_padding, top)
        };

        let mut action_under_mouse = None;
        let mut rect_under_mouse = None;
        let mut rect2_under_mouse = None;

        let mut inventory = HashMap::new();
        for item in &state.player.inventory {
            let count = inventory.entry(item.kind).or_insert(0);
            *count += 1;
        }

        let mut item_y_offset = 0;
        for kind in item::Kind::iter() {
            if inventory.get(&kind).is_some() {
                let rect = Rectangle::from_point_and_size(
                    inventory_pos + Point::new(-1, item_y_offset + 1),
                    Point::new(state.panel_width, 1),
                );
                if rect.contains(state.mouse.tile_pos) {
                    rect_under_mouse = Some(rect);
                    action_under_mouse = Some(match kind {
                        item::Kind::Food => Action::UseFood,
                        item::Kind::Dose => Action::UseDose,
                        item::Kind::CardinalDose => Action::UseCardinalDose,
                        item::Kind::DiagonalDose => Action::UseDiagonalDose,
                        item::Kind::StrongDose => Action::UseStrongDose,
                    });
                }
                item_y_offset += 1;
            }
        }

        let mut bottom = display.size_without_padding().y - if tall { 2 } else { 1 };

        let main_menu_button = {
            let text = if wide {
                "[Esc] Main Menu"
            } else {
                "[Esc] Menu"
            };
            Button::new(Point::new(x + left_padding, bottom), text).color(fg)
        };

        bottom -= if tall { 2 } else { 1 };
        let help_button = Button::new(Point::new(x + left_padding, bottom), "[?] Help").color(fg);

        // Position of the movement/numpad buttons
        bottom -= if tall { 10 } else { 9 };

        // NOTE: each button takes 3 tiles and there are 3 buttons in each row:
        let controls_width_tiles = 9;

        let left_padding = (state.panel_width - controls_width_tiles) / 2;

        // NOTE: since text width and tile width don't really match, the number of spaces
        // here was determined empirically and will not hold for different fonts.
        // TODO: These aren't really buttons more like rects so we should just draw those.
        let mut nw_button =
            Button::new(Point::new(x + left_padding + 0, bottom + 0), "     ").color(fg);
        nw_button.text_options.height = 3;
        let nw_button_small =
            Button::new(Point::new(x + left_padding + 3, bottom + 3), " ").color(fg);

        let mut n_button = Button::new(Point::new(x + left_padding + 3, bottom), "     ").color(fg);
        n_button.text_options.height = 3;
        let n_button_small =
            Button::new(Point::new(x + left_padding + 4, bottom + 3), " ").color(fg);

        let mut ne_button =
            Button::new(Point::new(x + left_padding + 6, bottom + 0), "     ").color(fg);
        ne_button.text_options.height = 3;
        let ne_button_small =
            Button::new(Point::new(x + left_padding + 5, bottom + 3), " ").color(fg);

        let mut w_button =
            Button::new(Point::new(x + left_padding + 0, bottom + 3), "     ").color(fg);
        w_button.text_options.height = 3;
        let w_button_small =
            Button::new(Point::new(x + left_padding + 3, bottom + 4), " ").color(fg);

        let mut e_button =
            Button::new(Point::new(x + left_padding + 6, bottom + 3), "     ").color(fg);
        e_button.text_options.height = 3;
        let e_button_small =
            Button::new(Point::new(x + left_padding + 5, bottom + 4), " ").color(fg);

        let mut sw_button =
            Button::new(Point::new(x + left_padding + 0, bottom + 6), "     ").color(fg);
        sw_button.text_options.height = 3;
        let sw_button_small =
            Button::new(Point::new(x + left_padding + 3, bottom + 5), " ").color(fg);

        let mut s_button =
            Button::new(Point::new(x + left_padding + 3, bottom + 6), "     ").color(fg);
        s_button.text_options.height = 3;
        let s_button_small =
            Button::new(Point::new(x + left_padding + 4, bottom + 5), " ").color(fg);

        let mut se_button =
            Button::new(Point::new(x + left_padding + 6, bottom + 6), "     ").color(fg);
        se_button.text_options.height = 3;
        let se_button_small =
            Button::new(Point::new(x + left_padding + 5, bottom + 5), " ").color(fg);

        let main_menu_rect = metrics.button_rect(&main_menu_button);
        if main_menu_rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MainMenu);
            rect_under_mouse = Some(main_menu_rect);
        }

        let help_rect = metrics.button_rect(&help_button);
        if help_rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::Help);
            rect_under_mouse = Some(help_rect);
        }

        let buttons = [
            (&n_button, n_button_small, Action::MoveN),
            (&s_button, s_button_small, Action::MoveS),
            (&w_button, w_button_small, Action::MoveW),
            (&e_button, e_button_small, Action::MoveE),
            (&nw_button, nw_button_small, Action::MoveNW),
            (&ne_button, ne_button_small, Action::MoveNE),
            (&sw_button, sw_button_small, Action::MoveSW),
            (&se_button, se_button_small, Action::MoveSE),
        ];

        for (button_big, button_small, action) in &buttons {
            let rect_big = metrics.button_rect(button_big);
            let rect_small = metrics.button_rect(button_small);

            if rect_big.contains(state.mouse.tile_pos) || rect_small.contains(state.mouse.tile_pos)
            {
                action_under_mouse = Some(*action);
                rect_under_mouse = Some(rect_big);
                rect2_under_mouse = Some(rect_small);
            }
        }

        Layout {
            x,
            fg,
            bg,
            mind_pos,
            progress_bar_pos,
            stats_pos,
            inventory_pos,
            inventory,
            action_under_mouse,
            rect_under_mouse,
            rect2_under_mouse,
            main_menu_button,
            help_button,
            nw_button,
            n_button,
            ne_button,
            w_button,
            e_button,
            sw_button,
            s_button,
            se_button,
            bottom,
        }
    }

    pub fn hovered(
        &self,
        state: &State,
        metrics: &dyn TextMetrics,
        display: &Display,
    ) -> Option<Action> {
        self.layout(state, metrics, display).action_under_mouse
    }

    pub fn render(
        &self,
        state: &State,
        metrics: &dyn TextMetrics,
        dt: Duration,
        fps: i32,
        display: &mut Display,
    ) {
        let wide = state.panel_width > 16;
        let tall = display.size_without_padding().y > 31;
        let left_padding = if wide { 1 } else { 0 };

        let layout = self.layout(state, metrics, display);
        let x = layout.x;
        let fg = layout.fg;
        let bg = layout.bg;
        let width = state.panel_width;
        let precision = width as usize;

        let height = display.size_without_padding().y;
        display.draw_rectangle(
            Rectangle::from_point_and_size(Point::new(x, 0), Point::new(width, height)),
            bg,
        );

        if let Some(highlighted) = layout.rect_under_mouse {
            display.draw_rectangle(highlighted, color::menu_highlight);
        }

        if let Some(highlighted) = layout.rect2_under_mouse {
            display.draw_rectangle(highlighted, color::menu_highlight);

            // Calculate player offset a move action would cause:
            let offset = match layout.action_under_mouse {
                Some(Action::MoveN) => Some((0, -1)),
                Some(Action::MoveS) => Some((0, 1)),
                Some(Action::MoveW) => Some((-1, 0)),
                Some(Action::MoveE) => Some((1, 0)),

                Some(Action::MoveNW) => Some((-1, -1)),
                Some(Action::MoveNE) => Some((1, -1)),
                Some(Action::MoveSW) => Some((-1, 1)),
                Some(Action::MoveSE) => Some((1, 1)),

                _ => None,
            };

            // Highlight the target tile the player would walk to if clicked in the sidebar numpad:
            if let Some(offset) = offset {
                let screen_left_top_corner = state.screen_position_in_world - (state.map_size / 2);
                let player_screen_pos = state.player.pos - screen_left_top_corner;
                // Only highlight when we're not re-centering the
                // screen (because that looks weird)
                if state.pos_timer.finished() {
                    display.set_background(player_screen_pos + offset, color::player);
                }
            }
        }

        let player = &state.player;

        let max_val = match player.mind {
            Mind::Withdrawal(val) => val.max(),
            Mind::Sober(val) => val.max(),
            Mind::High(val) => val.max(),
        };
        let mut bar_width = width - 2;
        if max_val < bar_width {
            bar_width = max_val;
        }

        let (mind_str, mind_val_percent) = match (player.alive(), player.mind) {
            (true, Mind::Withdrawal(val)) => ("Withdrawal", val.percent()),
            (true, Mind::Sober(val)) => ("Sober", val.percent()),
            (true, Mind::High(val)) => ("High", val.percent()),
            (false, _) => ("Lost", 0.0),
        };

        graphics::progress_bar(
            display,
            mind_val_percent,
            layout.progress_bar_pos,
            bar_width,
            color::gui_progress_bar_fg,
            color::gui_progress_bar_bg,
        );

        display.draw_button(&Button::new(layout.mind_pos, &mind_str).color(fg));

        let will_text = format!("Will: {}", player.will.to_int());
        let will_text_options = Default::default();

        // Show the anxiety counter as a progress bar next to the `Will` number
        if state.show_anxiety_counter {
            let will_bar_padding = if wide {
                metrics.get_text_width(&will_text, will_text_options)
            } else {
                0
            };
            graphics::progress_bar(
                display,
                state.player.anxiety_counter.percent(),
                layout.stats_pos + (will_bar_padding, 0),
                state.player.anxiety_counter.max(),
                color::anxiety_progress_bar_fg,
                color::anxiety_progress_bar_bg,
            );
        }
        display.draw_text(layout.stats_pos, &will_text, fg, will_text_options);

        let mut lines: Vec<Cow<'static, str>> = vec![];

        if !layout.inventory.is_empty() {
            display.draw_button(&Button::new(layout.inventory_pos, "Inventory:").color(fg));

            for kind in item::Kind::iter() {
                if let Some(count) = layout.inventory.get(&kind) {
                    lines.push(
                        format!(
                            "[{}] {:.pr$}: {}",
                            game::inventory_key(kind),
                            kind,
                            count,
                            pr = precision - 7
                        )
                        .into(),
                    );
                }
            }
        }

        lines.push("".into());

        if let Some(vnpc_id) = state.victory_npc_id {
            if let Some(vnpc_pos) = state.world.monster(vnpc_id).map(|m| m.position) {
                let distance = {
                    let dx = (player.pos.x - vnpc_pos.x) as f32;
                    let dy = (player.pos.y - vnpc_pos.y) as f32;
                    dx.abs().max(dy.abs()) as i32
                };
                if wide {
                    lines.push(format!("Distance to Victory NPC: {}", distance).into());
                } else {
                    lines.push(format!("Victory {} tiles", distance).into());
                }
                lines.push("".into());
            }
        }

        if !player.bonuses.is_empty() {
            lines.push("Active bonus:".into());
            for bonus in &player.bonuses {
                lines.push(format!("{}", bonus).into());
            }
            lines.push("".into());
        }

        if player.alive() {
            if player.stun.to_int() > 0 {
                lines.push(format!("Stunned({})", player.stun.to_int()).into());
            }
            if player.panic.to_int() > 0 {
                lines.push(format!("Panicking({})", player.panic.to_int()).into());
            }
        }

        if state.cheating {
            lines.push("CHEATING".into());
            lines.push("".into());

            if state.mouse.tile_pos >= (0, 0)
                && state.mouse.tile_pos < display.size_without_padding()
            {
                lines.push(format!("Mouse px: {}", state.mouse.screen_pos).into());
                lines.push(format!("Mouse: {}", state.mouse.tile_pos).into());
            }

            lines.push("Time stats:".into());
            for frame_stat in state.stats.last_frames(25) {
                lines.push(
                    format!(
                        "upd: {}, dc: {}",
                        frame_stat.update.as_millis(),
                        frame_stat.drawcalls.as_millis()
                    )
                    .into(),
                );
            }
            lines.push(format!("longest upd: {}", state.stats.longest_update().as_millis()).into());
            lines.push(
                format!(
                    "longest dc: {}",
                    state.stats.longest_drawcalls().as_millis()
                )
                .into(),
            );
        }

        for (y, line) in lines.into_iter().enumerate() {
            display.draw_text(
                Point {
                    x: x + left_padding,
                    y: y as i32 + layout.inventory_pos.y + 1,
                },
                &line,
                fg,
                Default::default(),
            );
        }

        display.draw_button(&layout.main_menu_button);
        display.draw_button(&layout.help_button);

        // Draw the clickable controls help
        display.draw_text(
            Point::new(x, layout.n_button.pos.y - 1),
            "Numpad Controls:",
            layout.fg,
            crate::engine::TextOptions::align_left(),
        );

        let numpad_buttons = [
            (&layout.nw_button, '7', (1, 1)),
            (&layout.n_button, '8', (0, 1)),
            (&layout.ne_button, '9', (-1, 1)),
            (&layout.w_button, '4', (1, 0)),
            (&layout.e_button, '6', (-1, 0)),
            (&layout.sw_button, '1', (1, -1)),
            (&layout.s_button, '2', (0, -1)),
            (&layout.se_button, '3', (-1, -1)),
        ];

        let tilesize = metrics.tile_width_px();
        for &(ref button, glyph, tile_offset) in &numpad_buttons {
            display.draw_button(button);

            // Offset to center the glyph. The font width is different from tilesize so we need
            // sub-tile (pixel-precise) positioning here:
            let x_offset_px = (tilesize - metrics.advance_width_px(glyph)) / 2;

            let tilepos_px = (button.pos + (1, 1) + tile_offset) * tilesize;
            display.draw_glyph_abs_px(
                tilepos_px.x + x_offset_px,
                tilepos_px.y,
                glyph,
                button.color,
            );
        }

        // Draw the `@` character in the middle of the controls diagram:
        // glyphs and their tile offset from centre
        let offset_glyphs = [
            ('@', (0, 0)),
            ('-', (-1, 0)),
            ('-', (1, 0)),
            ('|', (0, -1)),
            ('|', (0, 1)),
            ('\\', (-1, -1)),
            ('\\', (1, 1)),
            ('/', (1, -1)),
            ('/', (-1, 1)),
        ];

        // The centre tile doesn't have its own button but we can
        // calculate it from the surrounding tiles:
        let centre = Point::new(layout.n_button.pos.x, layout.w_button.pos.y) + (1, 1);
        for &(glyph, offset) in &offset_glyphs {
            let x_offset_px = (tilesize - metrics.advance_width_px(glyph)) / 2;
            let tilepos_px = (centre + offset) * tilesize;
            display.draw_glyph_abs_px(
                tilepos_px.x + x_offset_px,
                tilepos_px.y,
                glyph,
                layout.n_button.color,
            );
        }

        if state.cheating {
            display.draw_text(
                Point {
                    x: x + 1,
                    y: layout.bottom - 1,
                },
                &format!("dt: {}ms", dt.as_millis()),
                fg,
                Default::default(),
            );
            display.draw_text(
                Point {
                    x: x + 1,
                    y: layout.bottom,
                },
                &format!("FPS: {}", fps),
                fg,
                Default::default(),
            );
        }
    }
}
