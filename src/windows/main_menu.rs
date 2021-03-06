use crate::{
    audio::{Audio, Effect},
    engine::{Display, TextMetrics},
    game,
    game::RunningState,
    keys::KeyCode,
    settings::Settings,
    state::State,
    ui,
    window::{self, Window},
};

use egui::{
    self,
    paint::{Shape, Stroke},
    Rect, Ui,
};

use std::time::Duration;

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
    settings: &Settings,
    _metrics: &dyn TextMetrics,
    display: &mut Display,
    audio: &mut Audio,
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

    ui.painter().add(Shape::Rect {
        rect: Rect {
            min: [0.0, 0.0].into(),
            max: window_size_px.into(),
        },
        corner_radius: 0.0,
        fill: state.palette.gui_window_background.into(),
        stroke: Stroke {
            width: border_width_px,
            color: state.palette.gui_window_edge.into(),
        },
    });

    ui.painter().text(
        Into::<egui::Pos2>::into(window_size_px) - egui::Vec2::splat(70.0),
        egui::Align2([egui::Align::Max, egui::Align::Max]),
        format!(
            "Version: {}.{}",
            crate::metadata::VERSION_MAJOR,
            crate::metadata::VERSION_MINOR
        ),
        egui::TextStyle::Body,
        state.palette.gui_text.into(),
    );

    let mut action = None;

    // NOTE: this centers the UI area. Without it, we start in the top-left corner.
    //let mut ui = ui.centered_column(ui.available_size().y.min(480.0));
    // NOTE: This makes the buttons left-aligned but full-width
    //ui.set_layout(egui::Layout::justified(egui::Direction::Vertical));

    // This makes the buttons centered but only as wide as the text inside:
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        // NOTE: hack to add some top padding to the buttons and labels:
        ui.label("\n");

        ui.label("Dose Response");
        ui.label("By Tomas Sedovic");
        ui.label("");

        let game_in_progress = !state.game_ended && state.world.initialised();

        if game_in_progress
            && ui
                .add(ui::button("[R]esume", active, &state.palette))
                .clicked()
        {
            action = Some(MenuItem::Resume);
        }

        if ui
            .add(ui::button("[N]ew Game", active, &state.palette))
            .clicked()
        {
            action = Some(MenuItem::NewGame);
        }

        if ui
            .add(ui::button("[H]elp", active, &state.palette))
            .clicked()
        {
            action = Some(MenuItem::Help);
        }

        if ui
            .add(ui::button("S[e]ttings", active, &state.palette))
            .clicked()
        {
            action = Some(MenuItem::Settings);
        }

        if game_in_progress
            && ui
                .add(ui::button("[S]ave and Quit", active, &state.palette))
                .clicked()
        {
            action = Some(MenuItem::SaveAndQuit);
        }

        if ui
            .add(ui::button("[L]oad game", active, &state.palette))
            .clicked()
        {
            action = Some(MenuItem::Load);
        }

        let first_startup = !state.world.initialised() && !state.game_ended;
        let game_over = state.world.initialised() && state.game_ended;
        let quit_label = if first_startup {
            "[Q]uit without playing"
        } else if game_over {
            "[Q]uit"
        } else {
            // the game is in progress
            "[Q]uit without saving"
        };

        if ui
            .add(ui::button(quit_label, active, &state.palette))
            .clicked()
        {
            action = Some(MenuItem::Quit);
        };

        ui.label("");
        ui.label("\"You cannot lose if you do not play.\"\n-- Marla Daniels");
    });

    if action.is_none() && active {
        if state.keys.matches_code(KeyCode::E) {
            action = Some(MenuItem::Settings);
        } else if state.keys.matches_code(KeyCode::H)
            || state.keys.matches_code(KeyCode::QuestionMark)
        {
            action = Some(MenuItem::Help);
        } else if state.keys.matches_code(KeyCode::L) {
            action = Some(MenuItem::Load);
        } else if state.keys.matches_code(KeyCode::N) {
            action = Some(MenuItem::NewGame);
        } else if state.keys.matches_code(KeyCode::Q) {
            action = Some(MenuItem::Quit);
        } else if state.keys.matches_code(KeyCode::R)
            || state.keys.matches_code(KeyCode::Esc)
            || state.mouse.right_clicked
        {
            action = Some(MenuItem::Resume);
        } else if state.keys.matches_code(KeyCode::S) {
            action = Some(MenuItem::SaveAndQuit);
        }
    }

    if let Some(action) = action {
        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
        match action {
            MenuItem::Resume => {
                if state.world.initialised() {
                    state.window_stack.pop();
                }
                return RunningState::Running;
            }

            MenuItem::NewGame => {
                return RunningState::NewGame(Box::new(game::create_new_game_state(
                    state,
                    settings.challenge(),
                )));
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
