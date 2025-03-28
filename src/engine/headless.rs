use crate::{
    audio::Audio,
    engine::{loop_state::Metrics, Display, Mouse},
    game::RunningState,
    gamepad::Gamepad,
    point::Point,
    settings::Store as SettingsStore,
    state::State,
};

use std::time::Duration;

use egui::Context;

pub fn main_loop<S>(
    mut settings_store: S,
    initial_state: Box<State>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SettingsStore + 'static,
{
    let mut settings = settings_store.load();

    let egui_context = Context::default();
    egui_context.begin_pass(Default::default());

    let mouse = Mouse::new();
    let keys = vec![];
    let mut gamepad = Gamepad::new();

    let window_size_px = Point::new(settings.window_width as i32, settings.window_height as i32);
    let mut display = Display::new(window_size_px, settings.tile_size, settings.text_size);
    let mut audio = Audio::new(None);
    let mut game_state = initial_state;

    let tile_width_px = settings.tile_size;
    let text_width_px = settings.text_size;
    let metrics = &Metrics {
        tile_width_px,
        text_width_px,
    };

    let dt = Duration::from_millis(16);
    let fps = 60;

    loop {
        let mut update_result = crate::game::update(
            &mut game_state,
            &egui_context,
            dt,
            fps,
            &keys,
            mouse,
            &mut gamepad,
            &mut settings,
            metrics,
            &mut settings_store,
            &mut display,
            &mut audio,
        );

        let skipping = std::matches!(update_result, RunningState::Skip);
        if skipping {
            log::debug!("Skipping no-op frames...");
        }
        while std::matches!(update_result, RunningState::Skip) {
            update_result = crate::game::update(
                &mut game_state,
                &egui_context,
                dt,
                fps,
                &[],
                Mouse::new(),
                &mut gamepad,
                &mut settings,
                metrics,
                &mut settings_store,
                &mut display,
                &mut audio,
            );
        }
        if skipping {
            log::debug!("Finished the frame skip");
        }

        match update_result {
            RunningState::Running => {}
            RunningState::NewGame(_new_state) => unreachable!(),
            RunningState::Stopped => break,
            RunningState::Skip => unreachable!(),
        }
    }

    Ok(())
}
