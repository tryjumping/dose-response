use crate::{
    engine::Display,
    game::RunningState,
    keys::KeyCode,
    settings::{Settings, Store as SettingsStore},
    state::State,
    ui,
};

use egui::{self, Ui};

pub enum Action {
    Fullscreen,
    Window,
    TileSize(i32),
    TextSize(i32),
    Back,
    Apply,
}

pub fn process(
    state: &mut State,
    ui: &mut Ui,
    settings: &mut Settings,
    display: &mut Display,
    settings_store: &mut dyn SettingsStore,
) -> RunningState {
    let mut visible = true;
    let mut action = None;

    let display_size_px = display.size_without_padding() * display.tile_size;
    let window_size_px = [
        (display_size_px.x - 150) as f32,
        (display_size_px.y - 150) as f32,
    ];
    let window_pos_px = [
        (display_size_px.x as f32 - window_size_px[0]) / 2.0,
        (display_size_px.y as f32 - window_size_px[1]) / 2.0,
    ];

    egui::Window::new("Settings")
        .open(&mut visible)
        .collapsable(false)
        .fixed_pos(window_pos_px)
        .fixed_size(window_size_px)
        .show(ui.ctx(), |ui| {
            ui.columns(3, |c| {
                // NOTE: the tooltips don't have a window/screen
                // boundary checks and they just overflow. So I've put
                // the checkboxes with tooltips to the leftmost column
                // -- to make sure they're always visible.
                //
                // TODO: file a bug in egui for this.

                // TODO: implement these
                c[0].label("Challenge:");
                c[0].checkbox("Fast Depression", &mut true).tooltip_text(
                    "\
Checked: Depression moves two tiles per turn.
Unchecked: Depression moves one tile per turn.",
                );
                // NOTE: this how do we handle persistentcases like
                // exhaustion, overdose, loss of will, etc.? I think
                // we'll prolly want to drop thisone.
                c[0].checkbox("Player respawn", &mut false)
                    .tooltip_text("Does the player respawn instead of losing the game?");
                c[0].checkbox("Overdose", &mut true).tooltip_text(
                    "\
Checked: game over on overdose.
Unchecked: the game continues
 even on overdose.",
                );
                c[0].checkbox("Show all tiles", &mut false).tooltip_text(
                    "\
Checked: the entire map is uncovered.
Unchecked: only previously seen tiles
are visible.",
                );

                let mut available_key_shortcut = 1;

                c[1].label("Tile Size:");
                for &tile_size in crate::engine::AVAILABLE_TILE_SIZES.iter().rev() {
                    let selected = tile_size == settings.tile_size;
                    if c[1]
                        .radio(
                            format!("[{}] {}px", available_key_shortcut, tile_size),
                            selected,
                        )
                        .clicked
                    {
                        action = Some(Action::TileSize(tile_size));
                    };
                    available_key_shortcut += 1;
                }

                c[1].label("");
                c[1].label("Text Size:");
                for &text_size in crate::engine::AVAILABLE_TEXT_SIZES.iter().rev() {
                    let selected = text_size == settings.text_size;
                    if c[1]
                        .radio(
                            format!("[{}] {}px", available_key_shortcut, text_size),
                            selected,
                        )
                        .clicked
                    {
                        action = Some(Action::TextSize(text_size));
                    };
                    available_key_shortcut += 1;
                }

                c[2].label("Display:");
                if c[2].radio("[F]ullscreen", settings.fullscreen).clicked {
                    action = Some(Action::Fullscreen);
                }
                if c[2].radio("[W]indowed", !settings.fullscreen).clicked {
                    action = Some(Action::Window)
                }

                c[2].label("");
                c[2].label("Tiles:");
                c[2].radio("[G]raphical", true);
                c[2].radio("[T]extual (ASCII)", false);

                c[2].label("");
                c[2].label("Colour:");
                c[2].radio("[S]tandard", true);
                c[2].radio("[C]olour-blind", false);
                c[2].radio("C[u]stom", false);
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.add(ui::button("[A]pply Changes", true)).clicked {
                    action = Some(Action::Apply);
                }

                if ui.add(ui::button("[B]ack", true)).clicked {
                    action = Some(Action::Back);
                }
            });
        });

    if !visible {
        action = Some(Action::Back);
    }

    if state.keys.matches_code(KeyCode::Esc) || state.mouse.right_clicked {
        state.window_stack.pop();
        return RunningState::Running;
    }

    if action.is_none() {
        if state.keys.matches_code(KeyCode::F) {
            action = Some(Action::Fullscreen);
        } else if state.keys.matches_code(KeyCode::W) {
            action = Some(Action::Window);
        } else if state.keys.matches_code(KeyCode::A) {
            action = Some(Action::Apply);
        }
    }

    if action.is_none() {
        for (index, &size) in crate::engine::AVAILABLE_TILE_SIZES.iter().rev().enumerate() {
            let code = match index + 1 {
                1 => Some(KeyCode::D1),
                2 => Some(KeyCode::D2),
                3 => Some(KeyCode::D3),
                4 => Some(KeyCode::D4),
                5 => Some(KeyCode::D5),
                6 => Some(KeyCode::D6),
                7 => Some(KeyCode::D7),
                8 => Some(KeyCode::D8),
                9 => Some(KeyCode::D9),
                _ => None,
            };
            if let Some(code) = code {
                if state.keys.matches_code(code) {
                    action = Some(Action::TileSize(size));
                }
            }
        }
    }

    if action.is_none() {
        for (index, &size) in crate::engine::AVAILABLE_TEXT_SIZES.iter().rev().enumerate() {
            let code = match index + crate::engine::AVAILABLE_TILE_SIZES.len() + 1 {
                1 => Some(KeyCode::D1),
                2 => Some(KeyCode::D2),
                3 => Some(KeyCode::D3),
                4 => Some(KeyCode::D4),
                5 => Some(KeyCode::D5),
                6 => Some(KeyCode::D6),
                7 => Some(KeyCode::D7),
                8 => Some(KeyCode::D8),
                9 => Some(KeyCode::D9),
                _ => None,
            };
            if let Some(code) = code {
                if state.keys.matches_code(code) {
                    action = Some(Action::TextSize(size));
                }
            }
        }
    }

    if let Some(action) = action {
        match action {
            Action::Fullscreen => {
                settings.fullscreen = true;
            }

            Action::Window => {
                settings.fullscreen = false;
            }

            Action::TileSize(tile_size) => {
                settings.tile_size = tile_size;
            }

            Action::TextSize(text_size) => {
                log::info!("Changing text size to: {}", text_size);
                settings.text_size = text_size;
            }

            Action::Back => {
                *settings = settings_store.load();
                state.window_stack.pop();
            }

            Action::Apply => {
                settings_store.save(settings);
            }
        }
    }

    RunningState::Running
}
