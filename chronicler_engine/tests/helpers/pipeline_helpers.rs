//! Shared pipeline helpers used by integration tests across binaries; builds a minimal `GameState` and a few derived fixtures for action-pipeline scenarios.

#![allow(dead_code)]

use chronicler_engine::domain::model::state::game_state::GameState;
use chronicler_engine::domain::model::state::message_types::MessageType;

pub fn create_test_state_with_map() -> GameState {
    GameState::new("room1".to_string())
}

pub fn create_test_state_with_trigger_npc() -> GameState {
    GameState::new("room1".to_string())
}

use chronicler_engine::application::application_service::DefaultApplicationService;

pub fn wait_for_generation_complete(app: &DefaultApplicationService, timeout_ms: u64) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);
    while start.elapsed() < timeout {
        let state = app.load_or_fresh();
        if !state.narrative.input_buffer.status.is_generating() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    false
}

pub fn latest_state(app: &DefaultApplicationService) -> GameState {
    let mut state = app.load_or_fresh();
    app.load_messages_into_state(&mut state);
    state
}

pub fn save_state(app: &DefaultApplicationService, state: &GameState) {
    use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
    let storage = app.storage();
    let snapshot = GameStateSnapshot::from_game_state(state);
    let snapshot_id = storage.save_snapshot(&snapshot).unwrap();
    let existing = app.load_messages().unwrap_or_default();
    for msg in existing {
        let _ = storage.delete_message(msg.id);
    }
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.snapshot_id().is_none() {
            msg.set_snapshot_id(Some(snapshot_id));
        }
        if let Some(swipe) = msg.swipes.first_mut() {
            swipe.snapshot_id = Some(snapshot_id);
        }
        let id = storage.insert_message(&msg).unwrap();
        for (idx, swipe) in msg.swipes.iter().enumerate() {
            let _ = storage.insert_swipe(id, swipe, idx);
        }
    }
}

fn player_name(app: &DefaultApplicationService) -> String {
    app.storage()
        .get_game(app.current_game_id())
        .ok()
        .flatten()
        .and_then(|game| app.storage().get_persona(&game.persona_key).ok().flatten())
        .map(|persona| persona.sheet.name)
        .unwrap_or_else(|| "Player".to_string())
}

pub fn add_input_and_save(app: &DefaultApplicationService, text: &str) {
    let mut state = latest_state(app);
    let player_name = player_name(app);
    state.add_message(text.to_string(), Some(player_name), MessageType::Input);
    save_state(app, &state);
}

pub fn latest_snapshot(
    app: &DefaultApplicationService,
) -> chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot {
    let state = app.load_or_fresh();
    chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
        &state,
    )
}

pub fn wait_for_condition<F>(
    timeout: std::time::Duration,
    poll_interval: std::time::Duration,
    condition: F,
) -> bool
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        std::thread::sleep(poll_interval);
    }
    false
}
