use crate::{
    audio::{Audio, Effect},
    engine::{self, Display, VisualStyle},
    game::RunningState,
    gamepad::Gamepad,
    keys::KeyCode,
    settings::{Palette, Settings, Store as SettingsStore},
    state::State,
    ui,
};

use std::time::Duration;

use egui::{self, Ui};

#[derive(Copy, Clone, Debug)]
pub enum Action {
    FastDepression,
    Permadeath,
    HideUnseenTiles,
    Fullscreen,
    Window,
    VisualStyle(VisualStyle),
    Palette(Palette),
    TileSize(i32),
    TextSize(i32),
    MusicVolume(f32),
    SoundVolume(f32),
    Back,
    Apply,
}

pub fn process(
    state: &mut State,
    ui: &mut Ui,
    gamepad: &Gamepad,
    settings: &mut Settings,
    display: &mut Display,
    audio: &mut Audio,
    settings_store: &mut dyn SettingsStore,
) -> RunningState {
    let mut visible = true;
    let mut action = None;

    let screen_size_px = display.screen_size_px;
    let window_size_px = [
        ((screen_size_px.x - 150).min(1024)) as f32,
        ((screen_size_px.y - 150).min(768)) as f32,
    ];
    let window_pos_px = [
        (screen_size_px.x as f32 - window_size_px[0]) / 2.0,
        ((screen_size_px.y as f32 - window_size_px[1]) / 2.0).min(250.0),
    ];

    const FAST_DEPRESSION: Option<(i32, i32)> = Some((0, 0));
    const PERMADEATH: Option<(i32, i32)> = Some((0, 1));
    const HIDE_UNSEEN_TILES: Option<(i32, i32)> = Some((0, 2));
    const BACKGROUND_VOLUME: Option<(i32, i32)> = Some((1, 6));
    const SOUND_VOLUME: Option<(i32, i32)> = Some((1, 7));
    const FULLSCREEN: Option<(i32, i32)> = Some((2, 0));
    const WINDOWED: Option<(i32, i32)> = Some((2, 1));
    const GRAPHICAL: Option<(i32, i32)> = Some((2, 2));
    const TEXTUAL: Option<(i32, i32)> = Some((2, 3));
    const CLASSIC: Option<(i32, i32)> = Some((2, 4));
    const ACCESSIBLE: Option<(i32, i32)> = Some((2, 5));
    const GREYSCALE: Option<(i32, i32)> = Some((2, 6));

    let max_rows: [i32; 3] = [3, 8, 7];

    // NOTE: these buttons are outside of the `max_rows` table.
    // They'll be treaded specially in the UI.
    const APPLY: Option<(i32, i32)> = Some((0, 3));
    const BACK: Option<(i32, i32)> = Some((1, 8));

    let previous_settings_position = state.selected_settings_position;

    let stick_flicked_up = gamepad.left_stick_flicked && gamepad.left_stick_y > 0.0;
    let stick_flicked_down = gamepad.left_stick_flicked && gamepad.left_stick_y < 0.0;
    let stick_flicked_left = gamepad.left_stick_flicked && gamepad.left_stick_x < 0.0;
    let stick_flicked_right = gamepad.left_stick_flicked && gamepad.left_stick_x > 0.0;

    if state.keys.matches_code(KeyCode::Up) || stick_flicked_up {
        state.selected_settings_position = match state.selected_settings_position {
            current @ Some((column, row)) => {
                if row <= 0 {
                    APPLY
                } else if current == APPLY || current == BACK {
                    Some((0, max_rows[0] - 1))
                } else {
                    Some((column, row - 1))
                }
            }

            None => FAST_DEPRESSION,
        };
    }

    if state.keys.matches_code(KeyCode::Down) || stick_flicked_down {
        state.selected_settings_position = match state.selected_settings_position {
            current @ Some((column, row)) => {
                if row + 1 == max_rows[column as usize] {
                    APPLY
                } else if current == APPLY || current == BACK {
                    FAST_DEPRESSION
                } else {
                    Some((column, (row + 1) % max_rows[column as usize]))
                }
            }
            None => FAST_DEPRESSION,
        };
    }

    if state.keys.matches_code(KeyCode::Left) || stick_flicked_left {
        state.selected_settings_position = match state.selected_settings_position {
            current @ Some((column, _row)) => {
                if current == APPLY {
                    BACK
                } else if current == BACK {
                    APPLY
                } else {
                    let column = if column <= 0 { 2 } else { column - 1 };
                    Some((column, 0))
                }
            }
            None => FAST_DEPRESSION,
        }
    }

    if state.keys.matches_code(KeyCode::Right) || stick_flicked_right {
        state.selected_settings_position = match state.selected_settings_position {
            current @ Some((column, _row)) => {
                if current == APPLY {
                    BACK
                } else if current == BACK {
                    APPLY
                } else {
                    Some(((column + 1) % 3, 0))
                }
            }
            None => FAST_DEPRESSION,
        }
    }

    if previous_settings_position != state.selected_settings_position {
        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
    }

    egui::Window::new("Settings")
        .open(&mut visible)
        .collapsible(false)
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

                c[0].label("Gameplay:");
                let resp = c[0]
                    .checkbox(&mut settings.fast_depression, "Fast D[e]pression")
                    .on_hover_text(
                        "Checked: Depression moves two tiles per turn.
Unchecked: Depression moves one tile per turn.",
                    );
                if state.selected_settings_position == FAST_DEPRESSION {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        settings.fast_depression = !settings.fast_depression;
                        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                    state.selected_settings_position = None;
                }

                // NOTE: this how do we handle persistentcases like
                // exhaustion, overdose, loss of will, etc.? I think
                // we'll probably want to drop this one.
                let resp = c[0]
		    .checkbox(&mut settings.permadeath, "[O]nly one chance")
                    .on_hover_text(
                    "Checked: the game ends when the player loses (via overdose, depression, etc.).
Unchecked: all player effects are removed on losing. The game continues.",
                    );
                if state.selected_settings_position == PERMADEATH {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        settings.permadeath = !settings.permadeath;
                        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                    state.selected_settings_position = None;
                }

                let resp = c[0]
                    .checkbox(&mut settings.hide_unseen_tiles, "[H]ide unseen tiles")
                    .on_hover_text(
                        "Checked: only previously seen tiles are visible.
Unchecked: the entire map is uncovered.",
                    );
                if state.selected_settings_position == HIDE_UNSEEN_TILES {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        settings.hide_unseen_tiles = !settings.hide_unseen_tiles;
                        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                    state.selected_settings_position = None;
                }

                let mut available_key_shortcut = 1;
                let mut c1_row_index = 0;

                let tile_size_labels = ["Small", "Medium", "Large"];

                c[1].label("Tile Size:");
                for (index, &tile_size) in engine::AVAILABLE_TILE_SIZES.iter().rev().enumerate() {
                    let selected = tile_size == settings.tile_size;
                    let resp = c[1].radio(
                        selected,
                        format!("[{}] {}", available_key_shortcut, tile_size_labels[index]),
                    );
                    if state.selected_settings_position == Some((1, c1_row_index)) {
                        resp.request_focus();
                        if state.keys.matches_code(KeyCode::Enter) {
                            action = Some(Action::TileSize(tile_size));
                            audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                        }
                    } else {
                        resp.surrender_focus();
                    }
                    if resp.clicked() {
                        action = Some(Action::TileSize(tile_size));
                    };
                    available_key_shortcut += 1;
                    c1_row_index += 1;
                }

                let text_size_labels = ["Small", "Medium", "Large"];

                c[1].label("");
                c[1].label("Text Size:");
                for (index, &text_size) in engine::AVAILABLE_TEXT_SIZES.iter().rev().enumerate() {
                    let selected = text_size == settings.text_size;
                    let resp = c[1].radio(
                        selected,
                        format!("[{}] {}", available_key_shortcut, text_size_labels[index]),
                    );
                    if state.selected_settings_position == Some((1, c1_row_index)) {
                        resp.request_focus();
                        if state.keys.matches_code(KeyCode::Enter) {
                            action = Some(Action::TextSize(text_size));
                            audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                        }
                    } else {
                        resp.surrender_focus();
                    }
                    if resp.clicked() {
                        action = Some(Action::TextSize(text_size));
                    };
                    available_key_shortcut += 1;
                    c1_row_index += 1;
                }

                c[1].label("");
                c[1].label("Audio:");
                let mut play_music = settings.background_volume != 0.0;
                let resp = c[1].checkbox(&mut play_music, "Play [M]usic");
                if state.selected_settings_position == BACKGROUND_VOLUME {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        let volume = match play_music {
                            true => 0.0,
                            false => 1.0,
                        };
                        action = Some(Action::MusicVolume(volume));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    let volume = match play_music {
                        true => 1.0,
                        false => 0.0,
                    };
                    action = Some(Action::MusicVolume(volume));
                };

                let mut play_sound = settings.sound_volume != 0.0;
                let resp = c[1].checkbox(&mut play_sound, "Play So[u]nd");
                if state.selected_settings_position == SOUND_VOLUME {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        let volume = match play_sound {
                            true => 0.0,
                            false => 1.0,
                        };
                        action = Some(Action::SoundVolume(volume));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    let volume = match play_sound {
                        true => 1.0,
                        false => 0.0,
                    };
                    action = Some(Action::SoundVolume(volume));
                };

                c[2].label("Display:");
                let resp = c[2].radio(settings.fullscreen, "[F]ullscreen");
                if state.selected_settings_position == FULLSCREEN {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::Fullscreen);
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::Fullscreen);
                }

                let resp = c[2].radio(!settings.fullscreen, "[W]indowed");
                if state.selected_settings_position == WINDOWED {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::Window);
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::Window);
                }

                c[2].label("");
                c[2].label("Tile:");
                let resp = c[2].radio(
                    settings.visual_style == VisualStyle::Graphical,
                    "[G]raphical",
                );
                if state.selected_settings_position == GRAPHICAL {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::VisualStyle(VisualStyle::Graphical));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::VisualStyle(VisualStyle::Graphical));
                };

                let resp = c[2].radio(
                    settings.visual_style == VisualStyle::Textual,
                    "[T]extual (ASCII)",
                );
                if state.selected_settings_position == TEXTUAL {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::VisualStyle(VisualStyle::Textual));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::VisualStyle(VisualStyle::Textual));
                };

                c[2].label("");
                c[2].label("Colour:");
                let resp = c[2].radio(settings.palette == Palette::Classic, "Cla[s]sic");
                if state.selected_settings_position == CLASSIC {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::Palette(Palette::Classic));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::Palette(Palette::Classic));
                };

                let resp = c[2].radio(settings.palette == Palette::Accessible, "A[c]cessible");
                if state.selected_settings_position == ACCESSIBLE {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::Palette(Palette::Accessible));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::Palette(Palette::Accessible));
                };

                let resp = c[2].radio(settings.palette == Palette::Greyscale, "G[r]eyscale");
                if state.selected_settings_position == GREYSCALE {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::Palette(Palette::Greyscale));
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::Palette(Palette::Greyscale));
                };
            });

            // NOTE: on linux, the separator is visible but super thin, almost invisible
            // on macos, it's working just fine
            ui.separator();
            ui.horizontal(|ui| {
                let resp = ui::button(ui, "[A]ccept Changes", true, &state.palette);
                if state.selected_settings_position == APPLY {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::Apply);
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::Apply);
                }

                let resp = ui::button(ui, "[D]iscard Changes", true, &state.palette);
                if state.selected_settings_position == BACK {
                    resp.request_focus();
                    if state.keys.matches_code(KeyCode::Enter) {
                        action = Some(Action::Back);
                    }
                } else {
                    resp.surrender_focus();
                }
                if resp.clicked() {
                    action = Some(Action::Back);
                }
            });
        });

    if !visible {
        action = Some(Action::Back);
    }

    if state.keys.matches_code(KeyCode::Esc) || state.mouse.right_clicked {
        action = Some(Action::Back);
    }

    if action.is_none() {
        // NOTE: keep them alfasorted to spot conflicts quickly
        if state.keys.matches_code(KeyCode::A) {
            action = Some(Action::Apply);
        } else if state.keys.matches_code(KeyCode::C) {
            action = Some(Action::Palette(Palette::Accessible));
        } else if state.keys.matches_code(KeyCode::D) {
            action = Some(Action::Back);
        } else if state.keys.matches_code(KeyCode::E) {
            action = Some(Action::FastDepression)
        } else if state.keys.matches_code(KeyCode::F) {
            action = Some(Action::Fullscreen);
        } else if state.keys.matches_code(KeyCode::G) {
            action = Some(Action::VisualStyle(VisualStyle::Graphical));
        } else if state.keys.matches_code(KeyCode::H) {
            action = Some(Action::HideUnseenTiles)
        } else if state.keys.matches_code(KeyCode::M) {
            let volume = match settings.background_volume == 0.0 {
                true => 1.0,
                false => 0.0,
            };
            action = Some(Action::MusicVolume(volume))
        } else if state.keys.matches_code(KeyCode::O) {
            action = Some(Action::Permadeath)
        } else if state.keys.matches_code(KeyCode::R) {
            action = Some(Action::Palette(Palette::Greyscale));
        } else if state.keys.matches_code(KeyCode::S) {
            action = Some(Action::Palette(Palette::Classic));
        } else if state.keys.matches_code(KeyCode::U) {
            let volume = match settings.sound_volume == 0.0 {
                true => 1.0,
                false => 0.0,
            };
            action = Some(Action::SoundVolume(volume));
        } else if state.keys.matches_code(KeyCode::W) {
            action = Some(Action::Window);
        } else if state.keys.matches_code(KeyCode::T) {
            action = Some(Action::VisualStyle(VisualStyle::Textual));
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
        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
        state.selected_settings_position = None;
        match action {
            Action::FastDepression => {
                settings.fast_depression = !settings.fast_depression;
            }

            Action::Permadeath => {
                settings.permadeath = !settings.permadeath;
            }

            Action::HideUnseenTiles => {
                settings.hide_unseen_tiles = !settings.hide_unseen_tiles;
            }

            Action::Fullscreen => {
                settings.fullscreen = true;
            }

            Action::VisualStyle(visual_style) => {
                settings.visual_style = visual_style;
            }

            Action::Palette(palette) => {
                settings.palette = palette;
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

            Action::MusicVolume(volume) => {
                settings.background_volume = volume;
            }

            Action::SoundVolume(volume) => {
                settings.sound_volume = volume;
            }

            Action::Back => {
                *settings = settings_store.load();
                state.window_stack.pop();
            }

            Action::Apply => {
                state.palette = settings.palette();
                settings_store.save(settings);
                state.window_stack.pop();
            }
        }
    }

    RunningState::Running
}
