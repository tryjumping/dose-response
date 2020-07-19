use crate::{game::RunningState, state::State};

use egui::{self, Ui};

pub fn process(state: &mut State, ui: &mut Ui, text: &str) -> RunningState {
    let mut window_open = true;
    let mut close_button_clicked = false;
    egui::Window::new(text)
        .open(&mut window_open)
        .show(ui.ctx(), |ui| {
            ui.label(text);
            close_button_clicked = ui.button("Close").clicked;
        });

    let closed = !window_open || close_button_clicked;

    if closed || state.keys.get().is_some() || state.mouse.right_clicked {
        state.window_stack.pop();
    }

    RunningState::Running
}
