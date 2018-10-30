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
    action_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
}

pub struct Window;

impl Window {
    fn layout(&self, state: &State, metrics: &TextMetrics) -> Layout {
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

        bottom -= 1;

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
            bottom,
        }
    }

    pub fn hovered(&self, state: &State, metrics: &TextMetrics) -> Option<Action> {
        self.layout(state, metrics).action_under_mouse
    }

    pub fn render(
        &self,
        state: &State,
        metrics: &TextMetrics,
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

        let (mind_str, mind_val_percent) = match player.mind {
            Mind::Withdrawal(val) => ("Withdrawal", val.percent()),
            Mind::Sober(val) => ("Sober", val.percent()),
            Mind::High(val) => ("High", val.percent()),
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

        display.draw_text(
            layout.stats_pos,
            &format!("Will: {}", player.will.to_int()),
            fg,
            Default::default(),
        );

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

        if player.will.is_max() {
            lines.push(format!("Sobriety: {}", player.sobriety_counter.percent()).into());
            lines.push("".into());
        }

        if !player.bonuses.is_empty() {
            lines.push("Active bonus:".into());
            for bonus in &player.bonuses {
                lines.push(format!("{}", bonus).into());
            }
            lines.push("".into());
        }

        if state.cheating {
            lines.push("CHEATING".into());
            lines.push("".into());
        }

        if player.alive() {
            if player.stun.to_int() > 0 {
                lines.push(format!("Stunned({})", player.stun.to_int()).into());
            }
            if player.panic.to_int() > 0 {
                lines.push(format!("Panicking({})", player.panic.to_int()).into());
            }
        } else {
            lines.push("Dead".into());
        }

        if state.cheating {
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
                    ).into(),
                );
            }
            lines.push(
                format!(
                    "longest upd: {}",
                    util::num_milliseconds(state.stats.longest_update())
                ).into(),
            );
            lines.push(
                format!(
                    "longest dc: {}",
                    util::num_milliseconds(state.stats.longest_drawcalls())
                ).into(),
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
