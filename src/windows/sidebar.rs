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
use crate::util;

use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

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
    nw_button: Button,
    n_button: Button,
    ne_button: Button,
    w_button: Button,
    e_button: Button,
    sw_button: Button,
    s_button: Button,
    se_button: Button,
    action_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
}

pub struct Window;

impl Window {
    fn layout(&self, state: &State, metrics: &dyn TextMetrics) -> Layout {
        let x = state.map_size.x;
        let fg = color::gui_text;
        let bg = color::dim_background;

        let mind_pos = Point::new(x + 1, 0);
        let progress_bar_pos = Point::new(x + 1, 1);
        let stats_pos = Point::new(x + 1, 3);
        let inventory_pos = Point::new(x + 1, 5);

        let mut action_under_mouse = None;
        let mut rect_under_mouse = None;

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

        let mut bottom = state.display_size.y - 2;

        let main_menu_button = Button::new(Point::new(x + 1, bottom), "[Esc] Main Menu").color(fg);

        bottom -= 2;

        let help_button = Button::new(Point::new(x + 1, bottom), "[?] Help").color(fg);

        // Position of the movement/numpad buttons
        bottom -= 10;

        // NOTE: since text width and tile width don't really match, the number of spaces
        // here was determined empirically and will not hold for different fonts.
        // TODO: These aren't really buttons more like rects so we should just draw those.
        let mut nw_button = Button::new(Point::new(x + 1, bottom), "     ").color(fg);
        nw_button.text_options.height = 3;

        let mut n_button = Button::new(Point::new(x + 4, bottom), "     ").color(fg);
        n_button.text_options.height = 3;

        let mut ne_button = Button::new(Point::new(x + 7, bottom), "     ").color(fg);
        ne_button.text_options.height = 3;

        let mut w_button = Button::new(Point::new(x + 1, bottom + 3), "     ").color(fg);
        w_button.text_options.height = 3;

        let mut e_button = Button::new(Point::new(x + 7, bottom + 3), "     ").color(fg);
        e_button.text_options.height = 3;

        let mut sw_button = Button::new(Point::new(x + 1, bottom + 6), "     ").color(fg);
        sw_button.text_options.height = 3;

        let mut s_button = Button::new(Point::new(x + 4, bottom + 6), "     ").color(fg);
        s_button.text_options.height = 3;

        let mut se_button = Button::new(Point::new(x + 7, bottom + 6), "     ").color(fg);
        se_button.text_options.height = 3;

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

        let rect = metrics.button_rect(&nw_button);
        if rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MoveNW);
            rect_under_mouse = Some(rect);
        }

        let rect = metrics.button_rect(&n_button);
        if rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MoveN);
            rect_under_mouse = Some(rect);
        }

        let rect = metrics.button_rect(&ne_button);
        if rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MoveNE);
            rect_under_mouse = Some(rect);
        }

        let rect = metrics.button_rect(&w_button);
        if rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MoveW);
            rect_under_mouse = Some(rect);
        }

        let rect = metrics.button_rect(&e_button);
        if rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MoveE);
            rect_under_mouse = Some(rect);
        }

        let rect = metrics.button_rect(&sw_button);
        if rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MoveSW);
            rect_under_mouse = Some(rect);
        }

        let rect = metrics.button_rect(&s_button);
        if rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MoveS);
            rect_under_mouse = Some(rect);
        }

        let rect = metrics.button_rect(&se_button);
        if rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MoveSE);
            rect_under_mouse = Some(rect);
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

    pub fn hovered(&self, state: &State, metrics: &dyn TextMetrics) -> Option<Action> {
        self.layout(state, metrics).action_under_mouse
    }

    pub fn render(
        &self,
        state: &State,
        metrics: &dyn TextMetrics,
        dt: Duration,
        fps: i32,
        display: &mut Display,
    ) {
        let layout = self.layout(state, metrics);
        let x = layout.x;
        let fg = layout.fg;
        let bg = layout.bg;
        let width = state.panel_width;

        let height = state.display_size.y;
        display.draw_rectangle(
            Rectangle::from_point_and_size(Point::new(x, 0), Point::new(width, height)),
            bg,
        );

        if let Some(highlighted) = layout.rect_under_mouse {
            display.draw_rectangle(highlighted, color::menu_highlight);
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

        display.draw_button(&Button::new(layout.mind_pos, &mind_str).color(fg));

        graphics::progress_bar(
            display,
            mind_val_percent,
            layout.progress_bar_pos,
            bar_width,
            color::gui_progress_bar_fg,
            color::gui_progress_bar_bg,
        );

        let will_text = format!("Will: {}", player.will.to_int());
        let will_text_options = Default::default();
        display.draw_text(layout.stats_pos, &will_text, fg, will_text_options);

        // Show the anxiety counter as a progress bar next to the `Will` number
        if state.show_anxiety_counter {
            graphics::progress_bar(
                display,
                state.player.anxiety_counter.percent(),
                layout.stats_pos + (metrics.get_text_width(&will_text, will_text_options), 0),
                state.player.anxiety_counter.max(),
                color::anxiety_progress_bar_fg,
                color::anxiety_progress_bar_bg,
            );
        }

        let mut lines: Vec<Cow<'static, str>> = vec![];

        if !layout.inventory.is_empty() {
            display.draw_button(&Button::new(layout.inventory_pos, "Inventory:").color(fg));

            for kind in item::Kind::iter() {
                if let Some(count) = layout.inventory.get(&kind) {
                    lines.push(
                        format!("[{}] {:?}: {}", game::inventory_key(kind), kind, count).into(),
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
                lines.push(format!("Distance to Victory NPC: {}", distance).into());
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

            if state.mouse.tile_pos >= (0, 0) && state.mouse.tile_pos < state.display_size {
                lines.push(format!("Mouse px: {}", state.mouse.screen_pos).into());
                lines.push(format!("Mouse: {}", state.mouse.tile_pos).into());
            }

            lines.push("Time stats:".into());
            for frame_stat in state.stats.last_frames(25) {
                lines.push(
                    format!(
                        "upd: {}, dc: {}",
                        util::num_milliseconds(frame_stat.update),
                        util::num_milliseconds(frame_stat.drawcalls)
                    )
                    .into(),
                );
            }
            lines.push(
                format!(
                    "longest upd: {}",
                    util::num_milliseconds(state.stats.longest_update())
                )
                .into(),
            );
            lines.push(
                format!(
                    "longest dc: {}",
                    util::num_milliseconds(state.stats.longest_drawcalls())
                )
                .into(),
            );
        }

        for (y, line) in lines.into_iter().enumerate() {
            display.draw_text(
                Point {
                    x: x + 1,
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
            Point::new(x + 1, layout.n_button.pos.y - 1),
            "Movement controls (numpad):",
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
                &format!("dt: {}ms", util::num_milliseconds(dt)),
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
