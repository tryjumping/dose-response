#![allow(unused_variables, dead_code)]
use crate::{engine::Display, game::RunningState, keys::KeyCode, state::State};

use egui::{self, Ui};

/// This window appears only during a gameplay recording/replay.
///
/// It is not interactive and should not be visible during normal
/// playtime.
pub fn process(state: &mut State, ui: &mut Ui, display: &Display) -> RunningState {
    let expected_window_width: f32 = 700.0;
    let expected_window_height: f32 = 400.0;
    let padding = 50.0;
    let max_size = [
        display.screen_size_px.x as f32 - padding,
        display.screen_size_px.y as f32 - padding,
    ];
    let window_size = [
        expected_window_width.min(max_size[0]),
        expected_window_height.min(max_size[1]),
    ];
    let window_pos_px = [
        (display.screen_size_px.x as f32 - window_size[0]) / 2.0,
        (display.screen_size_px.y as f32 - window_size[1]) / 2.0,
    ];

    egui::Window::new("Dose Response")
        .collapsible(false)
        .fixed_pos(window_pos_px)
        .fixed_size(window_size)
        .show(ui.ctx(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label("By: Try Jumping");
                ui.label("");
                ui.label("Visit:");
                //ui.label("https://tryjumping.com");
                ui.label("https://store.steampowered.com/app/1750910/Dose_Response/");
                ui.label("")
            });
        });

    if state.keys.matches_code(KeyCode::Esc) || state.mouse.right_clicked {
        state.window_stack.pop();
    }

    RunningState::Running
}
