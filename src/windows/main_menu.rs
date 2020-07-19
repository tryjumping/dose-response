use crate::engine::{Display, TextMetrics};
use crate::state::State;

use egui::{self, Button, Ui};

#[derive(Debug)]
pub enum MenuItem {
    Resume,
    NewGame,
    Help,
    Settings,
    SaveAndQuit,
    Load,
    Quit,
}

pub fn process(
    state: &State,
    ui: &mut Ui,
    _metrics: &dyn TextMetrics,
    _display: &mut Display,
    visible: bool,
    active: bool,
) -> Option<MenuItem> {
    if !visible {
        return None;
    }
    // TODO: check all the UI padding & layouting work on mobile.
    // We're ignoring all that here, but a lot of work went into
    // doing that in the previous version of the UI.
    // Check if we need to do that here too.

    // NOTE: this centers the UI area. Without it, we start in the top-left corner.
    let mut ui = ui.centered_column(ui.available().width().min(480.0));
    //ui.set_layout(egui::Layout::vertical(Align::Min));
    ui.set_layout(egui::Layout::justified(egui::Direction::Vertical));
    // TODO: center the text here
    ui.label("Dose Response");
    ui.label("By Tomas Sedovic");

    if !state.game_ended && !state.first_game_already_generated {
        if ui.button("[R]esume").clicked {
            return Some(MenuItem::Resume);
        }
    }

    if ui.add(Button::new("[N]ew Game").enabled(active)).clicked {
        return Some(MenuItem::NewGame);
    }

    if ui.add(Button::new("[H]elp").enabled(active)).clicked {
        return Some(MenuItem::Help);
    }

    if ui.add(Button::new("S[e]ttings").enabled(active)).clicked {
        return Some(MenuItem::Settings);
    }

    if ui
        .add(Button::new("[S]ave and Quit").enabled(active))
        .clicked
    {
        return Some(MenuItem::SaveAndQuit);
    }

    if ui.add(Button::new("[L]oad game").enabled(active)).clicked {
        return Some(MenuItem::Load);
    }

    if ui
        .add(Button::new("[Q]uit without saving").enabled(active))
        .clicked
    {
        log::info!("Clicked!");
        return Some(MenuItem::Quit);
    };

    // TODO: move this to the bottom-left corner
    ui.label(" \"You cannot lose if you do not play.\"\n -- Marla Daniels");

    // TODO: move this to the bottom-right corner
    ui.label(format!(
        "Version: {}.{}",
        crate::metadata::VERSION_MAJOR,
        crate::metadata::VERSION_MINOR
    ));

    None
}
