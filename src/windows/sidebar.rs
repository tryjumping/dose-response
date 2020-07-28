use crate::{color, engine::Display, game, item, player::Mind, point::Point, state::State, ui};

use egui::{self, paint::PaintCmd, Rect, Ui};

use std::{collections::HashMap, time::Duration};

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

pub fn process(
    state: &mut State,
    ui: &mut Ui,
    dt: Duration,
    fps: i32,
    display: &Display,
    active: bool,
) -> (Option<Action>, Option<Point>) {
    let mut action = None;

    let width_px = 250.0;
    let bottom_left = [
        (display.screen_size_px.x - 1) as f32,
        (display.screen_size_px.y - 1) as f32,
    ];
    let top_left = [bottom_left[0] - width_px, 0.0];
    let ui_rect = Rect::from_min_max(top_left.into(), bottom_left.into());

    let padding = 20.0;
    let full_rect = Rect::from_min_max(
        [ui_rect.left() - padding, ui_rect.top()].into(),
        ui_rect.right_bottom(),
    );

    let mut ui = ui.child_ui(ui_rect);
    ui.set_clip_rect(full_rect);

    let mut style = ui.style().clone();
    style.text_color = color::gui_text.into();
    ui.set_style(style);

    ui.add_paint_cmd(PaintCmd::Rect {
        rect: full_rect,
        corner_radius: 0.0,
        outline: None,
        // TODO: use `color::dim_background` this for background
        fill: Some(color::RED.into()),
    });

    let player = &state.player;

    let (mind_str, mind_val_percent) = match (player.alive(), player.mind) {
        (true, Mind::Withdrawal(val)) => ("Withdrawal", val.percent()),
        (true, Mind::Sober(val)) => ("Sober", val.percent()),
        (true, Mind::High(val)) => ("High", val.percent()),
        (false, _) => ("Lost", 0.0),
    };

    let paint_list_pos = ui.paint_list_len();
    let mindstate_rect = ui.label(mind_str).rect;

    ui::progress_bar(
        &mut ui,
        paint_list_pos,
        mindstate_rect.left_top(),
        ui_rect.width() - padding,
        mindstate_rect.height(),
        mind_val_percent,
        color::gui_progress_bar_bg,
        color::gui_progress_bar_fg,
    );

    let paint_list_pos = ui.paint_list_len();
    let anxiety_counter_rect = ui.label(format!("Will: {}", player.will.to_int())).rect;

    // Show the anxiety counter as a progress bar next to the `Will` number
    if state.show_anxiety_counter {
        let top_left: egui::Pos2 = [
            anxiety_counter_rect.right() + padding,
            anxiety_counter_rect.top(),
        ]
        .into();

        ui::progress_bar(
            &mut ui,
            paint_list_pos,
            top_left,
            ui_rect.right() - padding - top_left.x,
            anxiety_counter_rect.height(),
            player.anxiety_counter.percent(),
            color::anxiety_progress_bar_bg,
            color::anxiety_progress_bar_fg,
        );
    }

    let mut inventory = HashMap::new();
    for item in &player.inventory {
        let count = inventory.entry(item.kind).or_insert(0);
        *count += 1;
    }

    if !inventory.is_empty() {
        ui.label("Inventory:");
        for kind in item::Kind::iter() {
            if let Some(count) = inventory.get(&kind) {
                let button_action = match kind {
                    item::Kind::Food => Action::UseFood,
                    item::Kind::Dose => Action::UseDose,
                    item::Kind::CardinalDose => Action::UseCardinalDose,
                    item::Kind::DiagonalDose => Action::UseDiagonalDose,
                    item::Kind::StrongDose => Action::UseStrongDose,
                };
                let precision = state.panel_width as usize;
                let button_label = format!(
                    "[{}] {:.pr$}: {}",
                    game::inventory_key(kind),
                    kind,
                    count,
                    pr = precision - 7
                );
                if ui.add(ui::button(&button_label, active)).clicked {
                    action = Some(button_action);
                };
            }
        }
    }

    if let Some(vnpc_id) = state.victory_npc_id {
        if let Some(vnpc_pos) = state.world.monster(vnpc_id).map(|m| m.position) {
            let distance = {
                let dx = (player.pos.x - vnpc_pos.x) as f32;
                let dy = (player.pos.y - vnpc_pos.y) as f32;
                dx.abs().max(dy.abs()) as i32
            };
            ui.label(format!("Distance to Victory NPC: {}", distance));
        }
    }

    if !player.bonuses.is_empty() {
        ui.label("Active bonus:");
        for bonus in &player.bonuses {
            ui.label(format!("{}", bonus));
        }
    }

    if player.alive() {
        if player.stun.to_int() > 0 {
            ui.label(format!("Stunned({})", player.stun.to_int()));
        }
        if player.panic.to_int() > 0 {
            ui.label(format!("Panicking({})", player.panic.to_int()));
        }
    }

    // NOTE: `Layout::reverse()` builds it up from the bottom:
    ui.inner_layout(egui::Layout::vertical(egui::Align::Min).reverse(), |ui| {
        if ui.add(ui::button("[Esc] Main Menu", active)).clicked {
            action = Some(Action::MainMenu);
        }
        if ui.add(ui::button("[?] Help", active)).clicked {
            action = Some(Action::Help);
        }
    });

    if state.cheating {
        ui.label("CHEATING");

        if state.mouse.tile_pos >= (0, 0) && state.mouse.tile_pos < display.size_without_padding() {
            ui.label(format!("Mouse px: {}", state.mouse.screen_pos));
            ui.label(format!("Mouse: {}", state.mouse.tile_pos));
        }

        ui.label(format!("dt: {}ms", dt.as_millis()));
        ui.label(format!("FPS: {}", fps));

        // // NOTE: commenting this out for now, we're not using the stats now
        // ui.label("Time stats:");
        // for frame_stat in state.stats.last_frames(25) {
        //     ui.label(format!(
        //         "upd: {}, dc: {}",
        //         frame_stat.update.as_millis(),
        //         frame_stat.drawcalls.as_millis()
        //     ));
        // }

        ui.label(format!(
            "longest upd: {}",
            state.stats.longest_update().as_millis()
        ));

        ui.label(format!(
            "longest dc: {}",
            state.stats.longest_drawcalls().as_millis()
        ));
    }

    let mut highlighted_tile = None;

    {
        // TODO: make sure this hardcoded position works even for
        // bigger fonts and smaller windows. I think we should be able
        // to calculate a good boundary by using some of the rects
        // calculated for other UI elements.
        let controls_pos_from_bottom = 300.0;
        let mut ui = ui.child_ui(Rect::from_min_max(
            [ui_rect.left(), ui_rect.bottom() - controls_pos_from_bottom].into(),
            ui_rect.right_bottom(),
        ));

        let mut highlighted_tile_offset_from_player_pos = None;

        ui.label("Numpad Controls:\n");
        ui.columns(3, |c| {
            let mut style = c[0].style().clone();
            style.button_padding = [20.0, 15.0].into();
            for index in 0..=2 {
                c[index].set_style(style.clone());
            }

            let btn = c[0].add(ui::button("7", active));
            if btn.clicked {
                action = Some(Action::MoveNW);
            };
            if btn.hovered {
                highlighted_tile_offset_from_player_pos = Some((-1, -1));
            }

            let btn = c[1].add(ui::button("8", active));
            if btn.clicked {
                action = Some(Action::MoveN);
            };
            if btn.hovered {
                highlighted_tile_offset_from_player_pos = Some((0, -1));
            }

            let btn = c[2].add(ui::button("9", active));
            if btn.clicked {
                action = Some(Action::MoveNE);
            };
            if btn.hovered {
                highlighted_tile_offset_from_player_pos = Some((1, -1));
            }

            let btn = c[0].add(ui::button("4", active));
            if btn.clicked {
                action = Some(Action::MoveW);
            };
            if btn.hovered {
                highlighted_tile_offset_from_player_pos = Some((-1, 0));
            }

            c[1].add(egui::Button::new("@").enabled(false));

            let btn = c[2].add(ui::button("6", active));
            if btn.clicked {
                action = Some(Action::MoveE);
            };
            if btn.hovered {
                highlighted_tile_offset_from_player_pos = Some((1, 0));
            }

            let btn = c[0].add(ui::button("1", active));
            if btn.clicked {
                action = Some(Action::MoveSW);
            };
            if btn.hovered {
                highlighted_tile_offset_from_player_pos = Some((-1, 1));
            }

            let btn = c[1].add(ui::button("2", active));
            if btn.clicked {
                action = Some(Action::MoveS);
            };
            if btn.hovered {
                highlighted_tile_offset_from_player_pos = Some((0, 1));
            }

            let btn = c[2].add(ui::button("3", active));
            if btn.clicked {
                action = Some(Action::MoveSE);
            };
            if btn.hovered {
                highlighted_tile_offset_from_player_pos = Some((1, 1));
            }
        });

        // Highlight the target tile the player would walk to if clicked in the sidebar numpad:
        if let Some(offset) = highlighted_tile_offset_from_player_pos {
            let screen_left_top_corner = state.screen_position_in_world - (state.map_size / 2);
            let player_screen_pos = state.player.pos - screen_left_top_corner;
            highlighted_tile = Some(player_screen_pos + offset);
        }
    }

    if active {
        (action, highlighted_tile)
    } else {
        (None, None)
    }
}
