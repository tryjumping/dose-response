use crate::{game::RunningState, state::State};

use egui::{self, Ui};

pub fn process(state: &mut State, ui: &mut Ui, title: &str, message: &str) -> RunningState {
    let mut window_open = true;
    egui::Window::new(title)
        .open(&mut window_open)
        .show(ui.ctx(), |ui| {
            ui.label(message);
        });

    let closed = !window_open;

    if closed || state.keys.get().is_some() || state.mouse.right_clicked {
        state.window_stack.pop();
    }

    RunningState::Running
}
