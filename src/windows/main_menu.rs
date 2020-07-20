use crate::{
    color,
    engine::{Display, TextMetrics},
    game,
    game::RunningState,
    keys::KeyCode,
    point::Point,
    rect::Rectangle,
    state::State,
    window::{self, Window},
};

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
    state: &mut State,
    ui: &mut Ui,
    _metrics: &dyn TextMetrics,
    display: &mut Display,
    active: bool,
) -> RunningState {
    // TODO: Any chance we could just replace all this with an egui window?
    let window_pos = Point::new(0, 0);
    let window_size = display.size_without_padding();
    let window_rect = Rectangle::from_point_and_size(window_pos, window_size);

    let inner_window_rect = Rectangle::new(
        window_rect.top_left() + (1, 1),
        window_rect.bottom_right() - (1, 1),
    );
    display.draw_rectangle(window_rect, color::window_edge);
    display.draw_rectangle(inner_window_rect, color::window_background);

    let mut action = None;

    // TODO: check all the UI padding & layouting work on mobile.
    // We're ignoring all that here, but a lot of work went into
    // doing that in the previous version of the UI.
    // Check if we need to do that here too.

    // NOTE: this centers the UI area. Without it, we start in the top-left corner.

    ui.floating_text(
        ui.available().translate([-70.0, -70.0].into()).max,
        format!(
            "Version: {}.{}",
            crate::metadata::VERSION_MAJOR,
            crate::metadata::VERSION_MINOR
        ),
        egui::TextStyle::Body,
        (egui::Align::Max, egui::Align::Max),
        None,
    );

    let mut ui = ui.centered_column(ui.available().width().min(480.0));

    // This makes the buttons centered but only as wide as the text inside:
    ui.set_layout(egui::Layout::vertical(egui::Align::Center));
    // NOTE: This makes the buttons left-aligned but full-width
    //ui.set_layout(egui::Layout::justified(egui::Direction::Vertical));

    // NOTE: hack to add some top padding to the buttons and labels:
    ui.label("\n");

    ui.label("Dose Response");
    ui.label("By Tomas Sedovic");
    ui.label("");

    if !state.game_ended && !state.first_game_already_generated {
        if ui.button("[R]esume").clicked {
            action = Some(MenuItem::Resume);
        }
    }

    if ui.add(Button::new("[N]ew Game").enabled(active)).clicked {
        action = Some(MenuItem::NewGame);
    }

    if ui.add(Button::new("[H]elp").enabled(active)).clicked {
        action = Some(MenuItem::Help);
    }

    if ui.add(Button::new("S[e]ttings").enabled(active)).clicked {
        action = Some(MenuItem::Settings);
    }

    if ui
        .add(Button::new("[S]ave and Quit").enabled(active))
        .clicked
    {
        action = Some(MenuItem::SaveAndQuit);
    }

    if ui.add(Button::new("[L]oad game").enabled(active)).clicked {
        action = Some(MenuItem::Load);
    }

    if ui
        .add(Button::new("[Q]uit without saving").enabled(active))
        .clicked
    {
        log::info!("Clicked!");
        action = Some(MenuItem::Quit);
    };

    ui.label("");
    ui.label("\"You cannot lose if you do not play.\"\n-- Marla Daniels");

    if action.is_none() {
        if state.keys.matches_code(KeyCode::Esc)
            || state.keys.matches_code(KeyCode::R)
            || state.mouse.right_clicked
        {
            action = Some(MenuItem::Resume);
        } else if state.keys.matches_code(KeyCode::N) {
            action = Some(MenuItem::NewGame);
        } else if state.keys.matches_code(KeyCode::QuestionMark)
            || state.keys.matches_code(KeyCode::H)
        {
            action = Some(MenuItem::Help);
        } else if state.keys.matches_code(KeyCode::E) {
            action = Some(MenuItem::Settings);
        } else if state.keys.matches_code(KeyCode::S) {
            action = Some(MenuItem::SaveAndQuit);
        } else if state.keys.matches_code(KeyCode::Q) {
            action = Some(MenuItem::Quit);
        } else if state.keys.matches_code(KeyCode::L) {
            action = Some(MenuItem::Load);
        }
    }

    if let Some(action) = action {
        match action {
            MenuItem::Resume => {
                state.window_stack.pop();
                return RunningState::Running;
            }

            MenuItem::NewGame => {
                // NOTE: When this is the first run, we resume the
                // game that's already loaded in the background.
                if state.first_game_already_generated {
                    state.window_stack.pop();
                    state.first_game_already_generated = false;
                    return RunningState::Running;
                } else {
                    return RunningState::NewGame(Box::new(game::create_new_game_state(state)));
                }
            }

            MenuItem::Help => {
                state.window_stack.push(Window::Help);
                return RunningState::Running;
            }

            MenuItem::Settings => {
                state.window_stack.push(Window::Settings);
                return RunningState::Running;
            }

            MenuItem::SaveAndQuit => {
                if !state.game_ended {
                    match state.save_to_file() {
                        Ok(()) => return RunningState::Stopped,
                        Err(error) => {
                            // NOTE: we couldn't save the game so we'll keep going
                            log::error!("Error saving the game: {:?}", error);
                            state
                                .window_stack
                                .push(window::message_box("Error: could not save the game."));
                        }
                    }
                }
                return RunningState::Running;
            }

            MenuItem::Load => match State::load_from_file() {
                Ok(new_state) => {
                    *state = new_state;
                    if state.window_stack.top() == Window::MainMenu {
                        state.window_stack.pop();
                    }
                    return RunningState::Running;
                }
                Err(error) => {
                    log::error!("Error loading the game: {:?}", error);
                    state
                        .window_stack
                        .push(window::message_box("Error: could not load the game."));
                    return RunningState::Running;
                }
            },

            MenuItem::Quit => {
                return RunningState::Stopped;
            }
        }
    }

    RunningState::Running
}
