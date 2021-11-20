use crate::{
    audio::{Audio, Effect},
    engine::{self, Display, VisualStyle},
    game::RunningState,
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
                if c[0]
                    .checkbox(&mut settings.fast_depression, "Fast D[e]pression")
                    .on_hover_text(
                        "Checked: Depression moves two tiles per turn.
Unchecked: Depression moves one tile per turn.",
                    )
                    .clicked()
                {
                    audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                }

                // NOTE: this how do we handle persistentcases like
                // exhaustion, overdose, loss of will, etc.? I think
                // we'll probably want to drop this one.
                if c[0].checkbox(&mut settings.permadeath, "[O]nly one chance")
                    .on_hover_text(
                    "Checked: the game ends when the player loses (via overdose, depression, etc.).
Unchecked: all player effects are removed on losing. The game continues.",
                    ).clicked()
                {
                    audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                }

                if c[0]
                    .checkbox(&mut settings.hide_unseen_tiles, "[H]ide unseen tiles")
                    .on_hover_text(
                        "Checked: only previously seen tiles are visible.
Unchecked: the entire map is uncovered.",
                    )
                    .clicked()
                {
                    audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
                }

                let mut available_key_shortcut = 1;

                let tile_size_labels = ["Small", "Medium", "Large"];

                c[1].label("Tile Size:");
                for (index, &tile_size) in engine::AVAILABLE_TILE_SIZES.iter().rev().enumerate() {
                    let selected = tile_size == settings.tile_size;
                    if c[1]
                        .radio(
                            selected,
                            format!("[{}] {}", available_key_shortcut, tile_size_labels[index]),
                        )
                        .clicked()
                    {
                        action = Some(Action::TileSize(tile_size));
                    };
                    available_key_shortcut += 1;
                }

                let text_size_labels = ["Small", "Medium", "Large"];

                c[1].label("");
                c[1].label("Text Size:");
                for (index, &text_size) in engine::AVAILABLE_TEXT_SIZES.iter().rev().enumerate() {
                    let selected = text_size == settings.text_size;
                    if c[1]
                        .radio(
                            selected,
                            format!("[{}] {}", available_key_shortcut, text_size_labels[index]),
                        )
                        .clicked()
                    {
                        action = Some(Action::TextSize(text_size));
                    };
                    available_key_shortcut += 1;
                }

                c[1].label("");
                c[1].label("Audio:");
                let mut play_music = settings.background_volume != 0.0;
                if c[1].checkbox(&mut play_music, "Play [M]usic").clicked() {
                    let volume = match play_music {
                        true => 1.0,
                        false => 0.0,
                    };
                    action = Some(Action::MusicVolume(volume));
                };

                let mut play_sound = settings.sound_volume != 0.0;
                if c[1].checkbox(&mut play_sound, "Play So[u]nd").clicked() {
                    let volume = match play_sound {
                        true => 1.0,
                        false => 0.0,
                    };
                    action = Some(Action::SoundVolume(volume));
                };

                c[2].label("Display:");
                if c[2].radio(settings.fullscreen, "[F]ullscreen").clicked() {
                    action = Some(Action::Fullscreen);
                }
                if c[2].radio(!settings.fullscreen, "[W]indowed").clicked() {
                    action = Some(Action::Window)
                }

                c[2].label("");
                c[2].label("Tile:");
                if c[2]
                    .radio(
                        settings.visual_style == VisualStyle::Graphical,
                        "[G]raphical",
                    )
                    .clicked()
                {
                    action = Some(Action::VisualStyle(VisualStyle::Graphical));
                };
                if c[2]
                    .radio(
                        settings.visual_style == VisualStyle::Textual,
                        "[T]extual (ASCII)",
                    )
                    .clicked()
                {
                    action = Some(Action::VisualStyle(VisualStyle::Textual))
                };

                c[2].label("");
                c[2].label("Colour:");
                if c[2]
                    .radio(settings.palette == Palette::Classic, "Cla[s]sic")
                    .clicked()
                {
                    action = Some(Action::Palette(Palette::Classic));
                };
                if c[2]
                    .radio(settings.palette == Palette::Accessible, "A[c]cessible")
                    .clicked()
                {
                    action = Some(Action::Palette(Palette::Accessible));
                };
                if c[2]
                    .radio(settings.palette == Palette::Greyscale, "G[r]eyscale")
                    .clicked()
                {
                    action = Some(Action::Palette(Palette::Greyscale));
                };
            });

            // NOTE: on linux, the separator is visible but super thin, almost invisible
            // on macos, it's working just fine
            ui.separator();
            ui.horizontal(|ui| {
                if ui::button(ui, "[A]ccept Changes", true, &state.palette).clicked() {
                    action = Some(Action::Apply);
                }

                if ui::button(ui, "[D]iscard Changes", true, &state.palette).clicked() {
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
