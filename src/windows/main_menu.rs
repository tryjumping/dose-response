use crate::{
    audio::{Audio, Effect},
    engine::{Display, TextMetrics},
    game,
    game::RunningState,
    gamepad::Gamepad,
    keys::KeyCode,
    settings::Settings,
    state::{GameSession, State},
    ui,
    window::{self, Window},
};

use egui::{
    self,
    epaint::{Shape, Stroke},
    Rect, Ui,
};

use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
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
    gamepad: &Gamepad,
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

    ui.painter().add(Shape::Rect(egui::epaint::RectShape::new(
        Rect {
            min: [0.0, 0.0].into(),
            max: window_size_px.into(),
        },
        0.0,
        state.palette.gui_window_background,
        Stroke {
            width: border_width_px,
            color: state.palette.gui_window_edge.into(),
        },
    )));

    ui.painter().text(
        Into::<egui::Pos2>::into(window_size_px) - egui::Vec2::splat(70.0),
        egui::Align2([egui::Align::Max, egui::Align::Max]),
        format!(
            "Version: {}.{}",
            crate::metadata::VERSION_MAJOR,
            crate::metadata::VERSION_MINOR
        ),
        egui::FontId::monospace(settings.text_size as f32),
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

        let game_in_progress = state.game_session == GameSession::InProgress;

        if state.selected_menu_action.is_none() {
            state.selected_menu_action = if game_in_progress {
                Some(MenuItem::Resume)
            } else {
                Some(MenuItem::NewGame)
            };
        }

        let stick_flicked_up = active && gamepad.left_stick_flicked && gamepad.left_stick_y > 0.0;
        let stick_flicked_down = active && gamepad.left_stick_flicked && gamepad.left_stick_y < 0.0;

        if active && (state.keys.matches_code(KeyCode::Down) || stick_flicked_down) {
            use MenuItem::*;
            // TODO: this is ignoring any disabled items, we need to handle those!
            let new_selected_action = match state.selected_menu_action {
                Some(Resume) => NewGame,
                Some(NewGame) => Help,
                Some(Help) => Settings,
                Some(Settings) => {
                    if game_in_progress {
                        SaveAndQuit
                    } else {
                        Load
                    }
                }
                Some(SaveAndQuit) => Load,
                Some(Load) => Quit,
                Some(Quit) => {
                    if game_in_progress {
                        Resume
                    } else {
                        NewGame
                    }
                }
                None => {
                    if game_in_progress {
                        Resume
                    } else {
                        NewGame
                    }
                }
            };
            log::info!("Selecting action: {:?}", new_selected_action);
            state.selected_menu_action = Some(new_selected_action);
            audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
        }

        if active && (state.keys.matches_code(KeyCode::Up) || stick_flicked_up) {
            use MenuItem::*;
            // TODO: this is ignoring any disabled items, we need to handle those!
            let new_selected_action = match state.selected_menu_action {
                Some(Resume) => Quit,
                Some(NewGame) => {
                    if game_in_progress {
                        Resume
                    } else {
                        Quit
                    }
                }
                Some(Help) => NewGame,
                Some(Settings) => Help,
                Some(SaveAndQuit) => Settings,
                Some(Load) => {
                    if game_in_progress {
                        SaveAndQuit
                    } else {
                        Settings
                    }
                }
                Some(Quit) => Load,
                None => {
                    if game_in_progress {
                        Resume
                    } else {
                        NewGame
                    }
                }
            };
            log::info!("Selecting action: {:?}", new_selected_action);
            state.selected_menu_action = Some(new_selected_action);
            audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
        }

        // TODO: handle the "gamepad up" button too!

        if game_in_progress {
            let resp = ui::button(ui, "[R]esume", active, &state.palette);
            if active && state.selected_menu_action == Some(MenuItem::Resume) {
                resp.request_focus();
            }
            if resp.clicked() {
                action = Some(MenuItem::Resume);
            }
        }

        {
            let resp = ui::button(ui, "[N]ew Game", active, &state.palette);
            if active && state.selected_menu_action == Some(MenuItem::NewGame) {
                resp.request_focus();
            }
            if resp.clicked() {
                action = Some(MenuItem::NewGame);
            }
        }

        {
            let resp = ui::button(ui, "[H]elp", active, &state.palette);
            if active && state.selected_menu_action == Some(MenuItem::Help) {
                resp.request_focus();
            }
            if resp.clicked() {
                action = Some(MenuItem::Help);
            }
        }

        {
            let resp = ui::button(ui, "S[e]ttings", active, &state.palette);
            if state.selected_menu_action == Some(MenuItem::Settings) {
                // TODO: we need to do this `if active` check everywhere!
                // Otherwise it'll silent the Settings (or any other dialog) options
                if active {
                    resp.request_focus();
                }
            }
            if resp.clicked() {
                action = Some(MenuItem::Settings);
            }
        }

        if game_in_progress {
            let resp = ui::button(ui, "[S]ave and Quit", active, &state.palette);
            if active && state.selected_menu_action == Some(MenuItem::SaveAndQuit) {
                resp.request_focus();
            }
            if resp.clicked() {
                action = Some(MenuItem::SaveAndQuit);
            }
        }

        {
            let resp = ui::button(ui, "[L]oad game", active, &state.palette);
            if active && state.selected_menu_action == Some(MenuItem::Load) {
                resp.request_focus();
            }
            if resp.clicked() {
                action = Some(MenuItem::Load);
            }
        }

        let quit_label = match state.game_session {
            GameSession::NotStarted => "[Q]uit without playing",
            GameSession::InProgress => "[Q]uit without saving",
            GameSession::Ended => "[Q]uit",
        };

        {
            let resp = ui::button(ui, quit_label, active, &state.palette);
            if active && state.selected_menu_action == Some(MenuItem::Quit) {
                resp.request_focus();
            }
            if resp.clicked() {
                action = Some(MenuItem::Quit);
            };
        }

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
        } else if state.keys.matches_code(KeyCode::Enter) {
            action = state.selected_menu_action;
        }
    }

    if let Some(action) = action {
        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
        match action {
            MenuItem::Resume => {
                if state.game_session.started() {
                    state.window_stack.pop();
                }
                return RunningState::Running;
            }

            MenuItem::NewGame => {
                // NOTE: don't create a new state if we've just
                // started the game. It's got a seend and world
                // already generated so there's no need to throw it
                // away and start a new one.
                if state.game_session == GameSession::NotStarted {
                    state.window_stack.pop();
                    state.game_session = GameSession::InProgress;
                    return RunningState::Running;
                }
                let mut new_state =
                    Box::new(game::create_new_game_state(state, settings.challenge()));
                new_state.game_session = GameSession::InProgress;
                return RunningState::NewGame(new_state);
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
                if state.game_session == GameSession::InProgress {
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
