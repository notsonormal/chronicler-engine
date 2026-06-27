use chronicler_engine::bootstrap::test_api::ArrivalTaskContext;
use chronicler_engine::model::character::NpcCard;
use chronicler_engine::model::state::MessageType;
use chronicler_engine::test_support::make_test_context_with_sqlite;

use crate::pipeline_helpers::{create_test_state_with_map, latest_state};

#[test]
fn test_arrival_narration_survives_reload() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let connection = {
        let settings_guard = ctx.settings.read().unwrap();
        settings_guard.narration_connection()
    };

    let preset_storage = ctx.preset_storage.clone();
    let arrival_preset = preset_storage
        .get_preset("system_default")
        .ok()
        .flatten()
        .expect("Should have system_default preset");

    let nearby_npcs: Vec<NpcCard> = vec![];
    let all_npcs: Vec<NpcCard> = vec![];

    let task_ctx = ArrivalTaskContext::new_for_test(
        ctx.clone(),
        "room1".to_string(),
        nearby_npcs,
        all_npcs,
        Some(arrival_preset),
        "short".to_string(),
        1024,
        None,
        connection,
    );

    task_ctx.run_sync();

    let messages = ctx.storage.list_latest_llm_messages(50).unwrap();
    let narration_msgs: Vec<_> = messages
        .iter()
        .filter(|m| m.agent_name == "narrator")
        .collect();

    assert!(
        !narration_msgs.is_empty(),
        "Should have at least one narration message in messages table after ArrivalTaskContext::run"
    );

    let first_narration = narration_msgs.first().unwrap();
    assert_ne!(
        first_narration.id, 0,
        "Narration message should have non-zero id (persisted to messages table)"
    );

    let guard = latest_state(&ctx);
    let history_narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();

    assert!(
        !history_narrations.is_empty(),
        "GameState should contain narration after ArrivalTaskContext::run"
    );

    let history_narration_text = history_narrations.first().unwrap().text.clone();

    let reloaded_messages =
        chronicler_engine::application::context::load_messages_with_swipes(&ctx.storage).unwrap();

    assert!(
        !reloaded_messages.is_empty(),
        "load_messages_with_swipes should return the persisted narration message"
    );

    let reloaded_narration = reloaded_messages
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .expect("Should find narration message after reload");

    assert_eq!(
        reloaded_narration.text(),
        history_narration_text,
        "Reloaded narration should match the one produced by ArrivalTaskContext::run"
    );
}
