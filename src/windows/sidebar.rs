use color;
use engine::{Draw, TextMetrics};
use graphics;
use game;
use item;
use player::Mind;
use point::Point;
use rect::Rectangle;
use state::State;
use util;

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
    main_menu_button: Draw,
    help_button: Draw,
    action_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
}

pub struct Window;

impl Window {
    fn layout(&self, state: &State, metrics: &TextMetrics) -> Layout {
        let x = state.map_size.x;
        let fg = color::gui_text;
        let bg = color::dim_background;
        let mut bottom = state.display_size.y - 2;

        let main_menu_button = Draw::Text(
            Point::new(x + 1, bottom),
            "[Esc] Main Menu".into(),
            fg,
            Default::default(),
        );

        bottom -= 2;

        let help_button = Draw::Text(
            Point::new(x + 1, bottom),
            "[?] Help".into(),
            fg,
            Default::default(),
        );
        bottom -= 1;

        let mut action_under_mouse = None;
        let mut rect_under_mouse = None;

        let main_menu_rect = metrics.text_rect(&main_menu_button);
        if main_menu_rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::MainMenu);
            rect_under_mouse = Some(main_menu_rect);
        }

        let help_rect = metrics.text_rect(&help_button);
        if help_rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::Help);
            rect_under_mouse = Some(help_rect);
        }

        Layout {
            x,
            fg,
            bg,
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
        drawcalls: &mut Vec<Draw>,
    ) {
        let layout = self.layout(state, metrics);
        let x = layout.x;
        let fg = layout.fg;
        let bg = layout.bg;
        let width = state.panel_width;

        let height = state.display_size.y;
        drawcalls.push(Draw::Rectangle(
            Rectangle::from_point_and_size(Point::new(x, 0), Point::new(width, height)),
            bg,
        ));

        let player = &state.player;

        let (mind_str, mind_val_percent) = match player.mind {
            Mind::Withdrawal(val) => ("Withdrawal", val.percent()),
            Mind::Sober(val) => ("Sober", val.percent()),
            Mind::High(val) => ("High", val.percent()),
        };

        let mut lines: Vec<Cow<'static, str>> = vec![
            mind_str.into(),
            "".into(), // NOTE: placeholder for the Mind state percentage bar
            "".into(),
            format!("Will: {}", player.will.to_int()).into(),
        ];

        if player.inventory.len() > 0 {
            lines.push("".into());
            lines.push("Inventory:".into());

            let mut item_counts = HashMap::new();
            for item in player.inventory.iter() {
                let count = item_counts.entry(item.kind).or_insert(0);
                *count += 1;
            }

            for kind in item::Kind::iter() {
                if let Some(count) = item_counts.get(&kind) {
                    lines.push(
                        format!("[{}] {:?}: {}", game::inventory_key(kind), kind, count).into(),
                    );
                }
            }
        }

        lines.push("".into());

        if player.will.is_max() {
            lines.push(format!("Sobriety: {}", player.sobriety_counter.percent()).into());
        }

        if !player.bonuses.is_empty() {
            lines.push("Bonus:".into());
            for bonus in &player.bonuses {
                lines.push(format!("* {:?}", bonus).into());
            }
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
                lines.push(format!("Mouse: {}", state.mouse.tile_pos).into())
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
            drawcalls.push(Draw::Text(
                Point {
                    x: x + 1,
                    y: y as i32,
                },
                line.into(),
                fg,
                Default::default(),
            ));
        }

        let max_val = match player.mind {
            Mind::Withdrawal(val) => val.max(),
            Mind::Sober(val) => val.max(),
            Mind::High(val) => val.max(),
        };
        let mut bar_width = width - 2;
        if max_val < bar_width {
            bar_width = max_val;
        }

        graphics::progress_bar(
            drawcalls,
            mind_val_percent,
            (x + 1, 1).into(),
            bar_width,
            color::gui_progress_bar_fg,
            color::gui_progress_bar_bg,
        );

        if let Some(highlighted) = layout.rect_under_mouse {
            drawcalls.push(Draw::Rectangle(highlighted, color::menu_highlight));
        }

        drawcalls.push(layout.main_menu_button);
        drawcalls.push(layout.help_button);

        if state.cheating {
            drawcalls.push(Draw::Text(
                Point {
                    x: x + 1,
                    y: layout.bottom - 1,
                },
                format!("dt: {}ms", util::num_milliseconds(dt)).into(),
                fg,
                Default::default(),
            ));
            drawcalls.push(Draw::Text(
                Point {
                    x: x + 1,
                    y: layout.bottom,
                },
                format!("FPS: {}", fps).into(),
                fg,
                Default::default(),
            ));
        }
    }
}
