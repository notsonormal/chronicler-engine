//! Integration flow tests for arrival narration persistence — confirms the arrival narration survives a state reload, exercising the `ArrivalTaskContext` end-to-end against SQLite storage.

use std::sync::Arc;

use chronicler_engine::application::arrival_service::ArrivalTaskContext;
use chronicler_engine::domain::model::character::NpcCard;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{make_test_context_with_sqlite, make_test_recorder_with_storage};

use crate::pipeline_helpers::{create_test_state_with_map, latest_state};

#[test]
fn test_arrival_narration_survives_reload() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let preset_storage = ctx.preset_storage.clone();
    let arrival_preset = preset_storage
        .get_preset("system_default")
        .ok()
        .flatten()
        .expect("Should have system_default preset");

    let nearby_npcs: Vec<NpcCard> = vec![];
    let all_npcs: Vec<NpcCard> = vec![];

    let recorder = make_test_recorder_with_storage(
        Arc::new(MockBackend::default())
            as Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider>,
        Arc::clone(&ctx.storage),
    );
    let task_ctx = ArrivalTaskContext::new_for_test(
        ctx.clone(),
        "room1".to_string(),
        nearby_npcs,
        all_npcs,
        Some(arrival_preset),
        "short".to_string(),
        1024,
        None,
        recorder,
    );

    task_ctx.run_sync();

    let messages = ctx.storage.list_latest_llm_messages(50).unwrap();
    let narration_msgs: Vec<_> = messages
        .iter()
        .filter(|m| m.agent_name == "narrator")
        .collect();

    assert!(
        !narration_msgs.is_empty(),
        "Should have at least one narration message in llm_messages table after ArrivalTaskContext::run"
    );

    let first_narration = narration_msgs.first().unwrap();
    assert_ne!(
        first_narration.id, 0,
        "Narration message should have non-zero id (persisted to llm_messages table)"
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

    let history_narration = history_narrations.first().unwrap();
    let history_narration_text = history_narration.text.clone();

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
        reloaded_narration.id, history_narration.id,
        "Reloaded narration message id should match the one in snapshot history (survives reload)"
    );

    assert_eq!(
        reloaded_narration.text(),
        history_narration_text,
        "Reloaded narration should match the one produced by ArrivalTaskContext::run"
    );
}
