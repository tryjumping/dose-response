use crate::{engine::Display, game::RunningState, state::State};

use egui::{self, Ui};

pub fn process(
    state: &mut State,
    ui: &mut Ui,
    title: &str,
    message: &str,
    display: &Display,
) -> RunningState {
    let display_size_px = display.screen_size_px;
    let window_size_px = [400.0, 300.0];
    let window_pos_px = [
        (display_size_px.x as f32 - window_size_px[0]) / 2.0,
        (display_size_px.y as f32 - window_size_px[1]) / 2.0,
    ];

    let mut window_open = true;
    egui::Window::new(title)
        .open(&mut window_open)
        .default_pos(window_pos_px)
        .default_size(window_size_px)
        .show(ui.ctx(), |ui| {
            ui.label(message);
        });

    let closed = !window_open;

    if closed || state.keys.get().is_some() || state.mouse.right_clicked {
        state.window_stack.pop();
    }

    RunningState::Running
}
