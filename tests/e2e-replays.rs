#![allow(clippy::expect_used)]
use dose_response::settings::Store;

use std::path::Path;

fn test_replay(replay_path: &Path) {
    assert!(replay_path.exists());

    let cheating = false;
    let invincible = false;
    let replay_full_speed = false;
    let exit_after = true;
    let debug = false;

    let settings_store = dose_response::settings::FileSystemStore::new();
    let settings = settings_store.load();
    let challenge = settings.challenge();
    let palette = settings.palette();

    let state = dose_response::state::State::replay_game(
        dose_response::WORLD_SIZE,
        dose_response::point::Point::from_i32(dose_response::DISPLAYED_MAP_SIZE),
        dose_response::PANEL_WIDTH,
        replay_path,
        cheating,
        invincible,
        replay_full_speed,
        exit_after,
        debug,
        challenge,
        palette,
    )
    .expect("state created");

    let result = dose_response::engine::headless::main_loop(settings_store, Box::new(state));
    assert!(matches!(result, Ok(())));
}

#[test]
fn test_almost_replay() {
    let replay_path = &Path::new("e2e-tests/almost-2024-09-27.gz");
    test_replay(replay_path);
}

#[test]
fn test_depression_replay() {
    let replay_path = &Path::new("e2e-tests/depression-2024-09-25.gz");
    test_replay(replay_path);
}

#[test]
fn test_exhaustion_replay() {
    let replay_path = &Path::new("e2e-tests/exhaustion-2024-09-25.gz");
    test_replay(replay_path);
}

#[test]
fn test_greedy_replay() {
    let replay_path = &Path::new("e2e-tests/greedy-2024-09-26.gz");
    test_replay(replay_path);
}

#[test]
fn test_overdose_replay() {
    let replay_path = &Path::new("e2e-tests/overdose-2024-09-25.gz");
    test_replay(replay_path);
}

#[test]
fn test_stunned_replay() {
    let replay_path = &Path::new("e2e-tests/stunned-2024-09-25.gz");
    test_replay(replay_path);
}

#[test]
fn test_victory_replay() {
    let replay_path = &Path::new("e2e-tests/victory-2024-10-01.gz");
    test_replay(replay_path);
}
