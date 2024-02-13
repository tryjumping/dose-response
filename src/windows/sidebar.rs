use crate::{
    engine::{Display, Texture, VisualStyle},
    formula, game,
    graphic::Graphic,
    item,
    keys::KeyCode,
    player::Mind,
    point::Point,
    settings::Settings,
    state::State,
    ui,
};

use egui::{self, paint::Shape, Pos2, Rect, Ui, Vec2};

use std::{collections::HashMap, time::Duration};

#[derive(Copy, Clone, Debug, PartialEq)]
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
    settings: &Settings,
    dt: Duration,
    fps: i32,
    display: &Display,
    active: bool,
) -> (Option<Action>, Option<Point>) {
    let mut action = None;

    if state.keys.matches_code(KeyCode::I) {
        state.inventory_focused = !state.inventory_focused;
        dbg!(state.inventory_focused);
    }
    if state.keys.matches_code(KeyCode::Enter) {
        dbg!(state.selected_sidebar_action);
        action = state.selected_sidebar_action;
        // We've taken the action, leave the inventory and go back to the game.
        state.inventory_focused = false;
    }
    if state.inventory_focused {
        use Action::*;
        if state.selected_sidebar_action.is_none() {
            state.selected_sidebar_action = Some(UseFood);
        }

        if state.keys.matches_code(KeyCode::Down) {
            let new_selected_action = match state.selected_sidebar_action {
                Some(UseFood) => UseDose,
                Some(UseDose) => UseCardinalDose,
                Some(UseCardinalDose) => UseDiagonalDose,
                Some(UseDiagonalDose) => UseStrongDose,
                Some(UseStrongDose) => UseFood,
                _ => UseFood,
            };
            state.selected_sidebar_action = Some(new_selected_action);
        }
        if state.keys.matches_code(KeyCode::Up) {
            let new_selected_action = match state.selected_sidebar_action {
                Some(UseFood) => UseStrongDose,
                Some(UseDose) => UseFood,
                Some(UseCardinalDose) => UseDose,
                Some(UseDiagonalDose) => UseCardinalDose,
                Some(UseStrongDose) => UseDiagonalDose,
                _ => UseStrongDose,
            };
            state.selected_sidebar_action = Some(new_selected_action);
        }
    } else {
        state.selected_sidebar_action = None;
    }

    let width_px = formula::sidebar_width_px(display.text_size) as f32;
    let bottom_right = Pos2::new(
        (display.screen_size_px.x + 1) as f32,
        (display.screen_size_px.y + 1) as f32,
    );
    let top_left = Pos2::new(bottom_right.x - width_px - 1.0, -1.0);
    let full_rect = Rect::from_min_max(top_left, bottom_right);

    let padding = Vec2::splat(20.0);
    let ui_rect = Rect::from_min_max(top_left + padding, bottom_right - padding);

    let mut ui = ui.child_ui(ui_rect, *ui.layout());
    ui.set_clip_rect(full_rect);

    ui.style_mut().visuals.override_text_color = Some(state.palette.gui_text.into());

    ui.painter().add(Shape::rect_filled(
        full_rect,
        0.0,
        state.palette.gui_sidebar_background,
    ));

    let player = &state.player;

    let (mind_str, mind_val_percent) = match (player.alive(), player.mind) {
        (true, Mind::Withdrawal(val)) => ("Withdrawal", val.percent()),
        (true, Mind::Sober(val)) => ("Sober", val.percent()),
        (true, Mind::High(val)) => ("High", val.percent()),
        (false, _) => ("Lost", 0.0),
    };

    let bg_progress_bar_pos = ui.painter().add(Shape::Noop);
    let fg_progress_bar_pos = ui.painter().add(Shape::Noop);
    let progress_padding = 2.0;
    let mindstate_rect = ui
        .colored_label(state.palette.gui_text, mind_str)
        .rect
        .expand(progress_padding);

    ui::progress_bar(
        &mut ui,
        bg_progress_bar_pos,
        fg_progress_bar_pos,
        mindstate_rect.left_top(),
        ui_rect.width(),
        mindstate_rect.height(),
        mind_val_percent,
        state.palette.gui_mind_progress_bar_bg,
        state.palette.gui_mind_progress_bar_fg,
    );

    let bg_anxiety_paint_pos = ui.painter().add(Shape::Noop);
    let fg_anxiety_paint_pos = ui.painter().add(Shape::Noop);
    let anxiety_counter_rect = ui.label(format!("Will: {}", player.will.to_int())).rect;

    // Show the anxiety counter as a progress bar next to the `Will` number
    if state.show_anxiety_counter {
        let top_left: egui::Pos2 = [
            anxiety_counter_rect.right() + progress_padding,
            anxiety_counter_rect.top(),
        ]
        .into();

        ui::progress_bar(
            &mut ui,
            bg_anxiety_paint_pos,
            fg_anxiety_paint_pos,
            top_left,
            ui_rect.right() - top_left.x - progress_padding,
            anxiety_counter_rect.height(),
            player.anxiety_counter.percent(),
            state.palette.gui_anxiety_progress_bar_bg,
            state.palette.gui_anxiety_progress_bar_fg,
        );
    }

    if player.stun.to_int() > 0 {
        ui.label(format!("Stunned({})", player.stun.to_int()));
    } else {
        ui.label("");
    }

    if player.panic.to_int() > 0 {
        ui.label(format!("Panicking({})", player.panic.to_int()));
    } else {
        ui.label("");
    }

    // NOTE: this ignores if we've got more than one bonus. That's
    // correct as of right now, but if we ever support more than one
    // bonus, we'll need to update this code!
    if let Some(bonus) = player.bonuses.first() {
        ui.label(format!("Bonus: {}", bonus));
    } else {
        ui.label("");
    }

    if player.bonuses.len() > 1 {
        log::warn!(
            "Player has more than one bonus! This is not supported at this time. Bonuses: {:#?}",
            player.bonuses
        );
    }

    if let Some(vnpc_id) = state.victory_npc_id {
        if let Some(vnpc_pos) = state.world.monster(vnpc_id).map(|m| m.position) {
            let distance = {
                let dx = (player.pos.x - vnpc_pos.x) as f32;
                let dy = (player.pos.y - vnpc_pos.y) as f32;
                dx.abs().max(dy.abs()) as i32
            };
            ui.label(format!("Victory Distance: {}", distance));
        } else {
            ui.label("");
        }
    } else {
        ui.label("");
    }

    let mut inventory = HashMap::new();
    for item in &player.inventory {
        let count = inventory.entry(item.kind).or_insert(0);
        *count += 1;
    }

    ui.label("\nInventory:");
    for kind in item::Kind::iter() {
        let count = *inventory.get(&kind).unwrap_or(&0);
        let button_action = match kind {
            item::Kind::Food => Action::UseFood,
            item::Kind::Dose => Action::UseDose,
            item::Kind::CardinalDose => Action::UseCardinalDose,
            item::Kind::DiagonalDose => Action::UseDiagonalDose,
            item::Kind::StrongDose => Action::UseStrongDose,
        };
        let graphic = match kind {
            item::Kind::Food => Graphic::FoodStriped,
            item::Kind::Dose => Graphic::Dose,
            item::Kind::CardinalDose => Graphic::CardinalDose,
            item::Kind::DiagonalDose => Graphic::DiagonalDose,
            item::Kind::StrongDose => Graphic::StrongDose,
        };
        let item_color = match kind {
            item::Kind::Food => state.palette.food,
            item::Kind::Dose => state.palette.dose,
            item::Kind::CardinalDose => state.palette.dose,
            item::Kind::DiagonalDose => state.palette.dose,
            item::Kind::StrongDose => state.palette.dose,
        };

        let tile_offset = match (settings.visual_style, settings.text_size) {
            (VisualStyle::Textual, 16) => Vec2::new(3.0, -2.0),
            (VisualStyle::Textual, 22) => Vec2::new(3.0, -1.0),
            (VisualStyle::Textual, 28) => Vec2::new(3.0, 0.0),
            (VisualStyle::Graphical, 16) => Vec2::new(3.0, -2.0),
            (VisualStyle::Graphical, 22) => Vec2::new(3.0, -1.0),
            (VisualStyle::Graphical, 28) => Vec2::new(3.0, 0.0),
            _ => Vec2::ZERO,
        };

        let panel_width_chars =
            (ui_rect.width() / settings.text_size as f32).abs().floor() as usize;
        let button_label = format!("{:.pr$}: {}", kind, count, pr = panel_width_chars);
        let active = active && count > 0;
        let texture = match settings.visual_style {
            VisualStyle::Graphical => Texture::Tilemap,
            VisualStyle::Textual => Texture::Glyph,
        };
        let button = ui::ImageTextButton::new(texture, button_label)
            .prefix_text(format!("[{}]", game::inventory_key(kind)))
            .tile(graphic)
            .tile_offset_px(tile_offset)
            .image_color(item_color)
            .text_color(state.palette.gui_text)
            .text_disabled_color(state.palette.gui_text_inactive)
            .selected(active)
            .background_color(state.palette.gui_button_background);

        let dbg_btn = button.clone();
        let resp = ui.add(button);
        if state.inventory_focused && Some(button_action) == state.selected_sidebar_action {
            resp.request_focus();
        } else {
            resp.surrender_focus();
        }
        if resp.clicked() {
            log::info!(
                "Button {:?} clicked! Click Action: {:?}",
                dbg_btn,
                button_action
            );
            action = Some(button_action);
        };
    }

    let mut help_rect = Rect::NAN; // Will be filled in later

    // NOTE: `Layout::reverse()` builds it up from the bottom:
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        if ui::button(ui, "[Esc] Main Menu", active, &state.palette).clicked() {
            action = Some(Action::MainMenu);
        }
        let gui_response = ui::button(ui, "[?] Help", active, &state.palette);
        if gui_response.clicked() {
            action = Some(Action::Help);
        }
        help_rect = gui_response.rect;
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
        let bottom_offset = formula::sidebar_numpad_offset_px(settings.text_size);
        let mut ui = ui.child_ui(
            Rect::from_min_max(
                [ui_rect.left(), help_rect.min.y - bottom_offset].into(),
                ui_rect.right_bottom(),
            ),
            *ui.layout(),
        );

        let mut highlighted_tile_offset_from_player_pos = None;

        ui.label("Numpad Controls:");
        ui.columns(3, |c| {
            for column in c.iter_mut() {
                column.style_mut().spacing.button_padding = [0.0, 25.0].into();
            }

            c[0].with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    let btn = ui::button(ui, "7", active, &state.palette);
                    if btn.clicked() {
                        action = Some(Action::MoveNW);
                    };
                    if btn.hovered() {
                        highlighted_tile_offset_from_player_pos = Some((-1, -1));
                    }

                    let btn = ui::button(ui, "4", active, &state.palette);
                    if btn.clicked() {
                        action = Some(Action::MoveW);
                    };
                    if btn.hovered() {
                        highlighted_tile_offset_from_player_pos = Some((-1, 0));
                    }

                    let btn = ui::button(ui, "1", active, &state.palette);
                    if btn.clicked() {
                        action = Some(Action::MoveSW);
                    };
                    if btn.hovered() {
                        highlighted_tile_offset_from_player_pos = Some((-1, 1));
                    }
                },
            );

            c[1].with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    let btn = ui::button(ui, "8", active, &state.palette);
                    if btn.clicked() {
                        action = Some(Action::MoveN);
                    };
                    if btn.hovered() {
                        highlighted_tile_offset_from_player_pos = Some((0, -1));
                    }

                    // Add the player image
                    {
                        let texture = match settings.visual_style {
                            VisualStyle::Graphical => Texture::Tilemap,
                            VisualStyle::Textual => Texture::Glyph,
                        };
                        let text_size = settings.text_size as f32;
                        let (uv, tilesize) =
                            ui::image_uv_tilesize(texture, state.player.graphic, text_size);
                        let image_color = state.palette.player(state.player.color_index);
                        let image =
                            egui::widgets::Image::new(texture.into(), Vec2::splat(tilesize))
                                .uv(uv)
                                .tint(image_color);

                        // Allocate the same UI space as any other button to keep the layout correct
                        let sense = egui::Sense::click();
                        let (rect, _response) = ui.allocate_exact_size(btn.rect.size(), sense);

                        // Calculate the size of the actual rendered image and centre it
                        let image_rect = {
                            // NOTE: this will return a rect with floating point values:
                            let r = Rect::from_center_size(rect.center(), Vec2::splat(text_size));
                            // We need to convert it to integers:
                            Rect {
                                min: r.min.floor(),
                                max: r.max.floor(),
                            }
                        };
                        image.paint_at(ui, image_rect);
                    };

                    let btn = ui::button(ui, "2", active, &state.palette);
                    if btn.clicked() {
                        action = Some(Action::MoveS);
                    };
                    if btn.hovered() {
                        highlighted_tile_offset_from_player_pos = Some((0, 1));
                    }
                },
            );

            c[2].with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    let btn = ui::button(ui, "9", active, &state.palette);
                    if btn.clicked() {
                        action = Some(Action::MoveNE);
                    };
                    if btn.hovered() {
                        highlighted_tile_offset_from_player_pos = Some((1, -1));
                    }

                    let btn = ui::button(ui, "6", active, &state.palette);
                    if btn.clicked() {
                        action = Some(Action::MoveE);
                    };
                    if btn.hovered() {
                        highlighted_tile_offset_from_player_pos = Some((1, 0));
                    }

                    let btn = ui::button(ui, "3", active, &state.palette);
                    if btn.clicked() {
                        action = Some(Action::MoveSE);
                    };
                    if btn.hovered() {
                        highlighted_tile_offset_from_player_pos = Some((1, 1));
                    }
                },
            );
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
