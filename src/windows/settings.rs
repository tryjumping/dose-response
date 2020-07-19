use crate::{engine::Display, settings::Settings, state::State};

use egui::{self, Ui, Window as GuiWindow};

pub enum Action {
    Fullscreen,
    Window,
    TileSize(i32),
    TextSize(i32),
    Back,
    Apply,
}

pub fn process(
    _state: &State,
    ui: &mut Ui,
    settings: &Settings,
    display: &mut Display,
) -> Option<Action> {
    let mut visible = true;

    // NOTE: this is why I think it probably makes sense to keep
    // the logic and rendering in the same place. We won't have to
    // be returning actions or whatnot to process them later. But
    // IDK might lead to spagetti code and right now, the GUI
    // layout and the code is cleanly separate. IDK.
    let mut action = None;

    // TODO: resizing the game window doesn't resize the settings window properly.

    let display_size_px = display.size_without_padding() * display.tile_size;
    let window_size_px = [
        (display_size_px.x - 150) as f32,
        (display_size_px.y - 150) as f32,
    ];
    let window_pos_px = [
        (display_size_px.x as f32 - window_size_px[0]) / 2.0,
        (display_size_px.y as f32 - window_size_px[1]) / 2.0,
    ];

    GuiWindow::new("Settings")
        .open(&mut visible)
        .default_pos(window_pos_px)
        .fixed_size(window_size_px)
        .show(ui.ctx(), |ui| {
            ui.columns(2, |c| {
                c[0].label("Display:");
                if c[0].radio("[F]ullscreen", settings.fullscreen).clicked {
                    action = Some(Action::Fullscreen);
                }
                if c[0].radio("[W]indowed", !settings.fullscreen).clicked {
                    action = Some(Action::Window)
                }

                let mut available_key_shortcut = 1;

                c[0].label("Tile Size:");
                for &tile_size in crate::engine::AVAILABLE_TILE_SIZES.iter().rev() {
                    let selected = tile_size == settings.tile_size;
                    if c[0]
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

                c[0].label("Text Size:");
                for &text_size in crate::engine::AVAILABLE_TEXT_SIZES.iter().rev() {
                    let selected = text_size == settings.text_size;
                    if c[0]
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

                c[1].label("Accessibility:");
                c[1].label("Tiles:");
                c[1].radio("[G]raphical", true);
                c[1].radio("[T]extual (ASCII)", false);

                c[1].label("Colour:");
                c[1].radio("[S]tandard", true);
                c[1].radio("[C]olour-blind", false);
                c[1].radio("C[u]stom", false);
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("[A]pply Changes").clicked {
                    action = Some(Action::Apply);
                }

                if ui.button("[B]ack").clicked {
                    action = Some(Action::Back);
                }
            });
        });

    if !visible {
        return Some(Action::Back);
    }

    action
}
