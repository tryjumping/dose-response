use crate::{
    color,
    engine::{Display, TextMetrics},
    game,
    game::RunningState,
    keys::KeyCode,
    state::State,
    ui,
    window::{self, Window},
};

use egui::{
    self,
    paint::{LineStyle, PaintCmd},
    Rect, Ui,
};

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
    // TODO: check all the UI padding & layouting work on mobile.
    // We're ignoring all that here, but a lot of work went into
    // doing that in the previous version of the UI.
    // Check if we need to do that here too.

    let window_size_px = display.screen_size_px;

    // NOTE: half of the border is inside the rect and half is
    // outside. Since the edge of the rectangle is the edge of the
    // window, we only see half of this. By making the outline twice
    // as wide, we'll see the desired thickness.
    let border_width_px = 30.0 * 2.0;

    ui.painter().add(PaintCmd::Rect {
        rect: Rect {
            min: [0.0, 0.0].into(),
            max: [window_size_px.x as f32, window_size_px.y as f32].into(),
        },
        corner_radius: 0.0,
        fill: Some(color::window_background.into()),
        outline: Some(LineStyle {
            width: border_width_px,
            color: color::window_edge.into(),
        }),
    });

    ui.painter().floating_text(
        ui.available().translate([-70.0, -70.0].into()).max,
        format!(
            "Version: {}.{}",
            crate::metadata::VERSION_MAJOR,
            crate::metadata::VERSION_MINOR
        ),
        egui::TextStyle::Body,
        (egui::Align::Max, egui::Align::Max),
        color::gui_text.into(),
    );

    // NOTE: this centers the UI area. Without it, we start in the top-left corner.
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

    let mut action = None;

    if !state.game_ended && !state.first_game_already_generated {
        if ui.add(ui::button("[R]esume", active)).clicked {
            action = Some(MenuItem::Resume);
        }
    }

    if ui.add(ui::button("[N]ew Game", active)).clicked {
        action = Some(MenuItem::NewGame);
    }

    if ui.add(ui::button("[H]elp", active)).clicked {
        action = Some(MenuItem::Help);
    }

    if ui.add(ui::button("S[e]ttings", active)).clicked {
        action = Some(MenuItem::Settings);
    }

    if ui.add(ui::button("[S]ave and Quit", active)).clicked {
        action = Some(MenuItem::SaveAndQuit);
    }

    if ui.add(ui::button("[L]oad game", active)).clicked {
        action = Some(MenuItem::Load);
    }

    if ui.add(ui::button("[Q]uit without saving", active)).clicked {
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
                            state.window_stack.push(window::message_box(
                                "Save Game",
                                "Error: could not save the game.",
                            ));
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
                    state.window_stack.push(window::message_box(
                        "Load Game",
                        "Error: could not load the game.",
                    ));
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
