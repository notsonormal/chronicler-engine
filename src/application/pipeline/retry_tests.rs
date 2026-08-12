//! Unit tests for retry and retrigger paths.

use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::adapters::driving::http::AppState;
use crate::application::ports::llm_provider::LlmCallResult;
use crate::application::errors::ApplicationError;
use crate::application::errors::ProcessActionResult;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;
use crate::application::agents::registry::AgentRegistry;
use crate::test_support::fixtures::TestGameState;
use crate::test_support::make_test_pipeline_with_backends;
use crate::test_support::make_test_pipeline_with_mock_quantifier;
use crate::test_support::make_test_recorder;
use crate::test_support::TestAppBuilder;
use crate::test_support::TestDataBuilder;
use crate::test_support::make_test_app_without_snapshot;

fn make_test_state() -> GameState {
    TestGameState::in_room("start")
}

fn make_service() -> crate::application::pipeline::pipeline::ActionPipeline {
    let storage = Arc::new(Storage::new_in_memory());
    crate::test_support::make_test_pipeline_with_mock_quantifier(
        storage,
        make_test_recorder(Arc::new(MockBackend::default())),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    )
}

fn insert_message_with_swipe(
    _app: &AppState,
    storage: &crate::adapters::driven::storage::Storage,
    msg: &crate::domain::model::message::Message,
) {
    let id = storage.insert_message(msg).unwrap();
    if let Some(swipe) = msg.swipes.first() {
        let mut swipe = swipe.clone();
        swipe.text = msg.text().to_string();
        swipe.snapshot_id = msg.snapshot_id();
        swipe.location_header = msg.location_header().map(|s| s.to_string());
        swipe.event_header = msg.event_header().map(|s| s.to_string());
        let _ = storage.insert_swipe(id, &swipe, 0);
    }
}

fn add_input_and_save(
    app: &AppState,
    storage: &crate::adapters::driven::storage::Storage,
    text: &str,
) -> u64 {
    let mut state = app.message_service.load_or_fresh();
    let player_name = "Player".to_string();
    state.add_message(text.to_string(), Some(player_name), MessageType::Input);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(id));
        insert_message_with_swipe(app, storage, last);
    }
    id
}

fn add_narration_and_save(
    app: &AppState,
    storage: &crate::adapters::driven::storage::Storage,
    text: &str,
) -> u64 {
    let mut state = app.message_service.load_or_fresh();
    state.add_message(text.to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(id));
        insert_message_with_swipe(app, storage, last);
    }
    id
}

fn save_pre_main(app: &AppState, storage: &crate::adapters::driven::storage::Storage) -> u64 {
    let state = app.message_service.load_or_fresh();
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    storage.save_snapshot(&snapshot).unwrap()
}

fn save_pre_event(app: &AppState, storage: &crate::adapters::driven::storage::Storage) -> u64 {
    let state = app.message_service.load_or_fresh();
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    storage.save_snapshot(&snapshot).unwrap()
}

fn setup_event_flow(app: &AppState, storage: &crate::adapters::driven::storage::Storage) {
    let _ = add_input_and_save(app, storage, "test input");
    let _ = save_pre_main(app, storage);

    let mut pre_event_state = app.message_service.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(app, storage, last);
    }

    let mut final_state = pre_event_state;
    final_state.add_message("Event narration".to_string(), None, MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &final_state,
        );
    let final_id = storage.save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(app, storage, last);
    }
}

fn setup_event_flow_without_trigger(
    app: &AppState,
    storage: &crate::adapters::driven::storage::Storage,
) {
    let _ = add_input_and_save(app, storage, "test input");
    let _ = save_pre_main(app, storage);

    let mut pre_event_state = app.message_service.load_or_fresh();
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(app, storage, last);
    }

    let mut final_state = pre_event_state;
    final_state.add_message("Event narration".to_string(), None, MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &final_state,
        );
    let final_id = storage.save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(app, storage, last);
    }
}

#[tokio::test]
async fn test_retry_no_snapshot() {
    let state = make_test_state();
    let wired = make_test_app_without_snapshot(state).unwrap();
    let _storage = Arc::clone(&wired.storage);
    let app = AppState::from_wired(wired);
    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Error(ref msg) if msg.contains("Retry failed: no anchor message")),
        "Should record retry error when no anchor message exists, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_load_messages_error() {
    let _state = make_test_state();
    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        "load_message_rows",
        TestOverride::internal("simulated load_message_rows failure"),
    );

    let app = AppState::from_wired(
        crate::test_support::build_test_wired_app(failing, make_service())
            .expect("build_test_wired_app: build_app_graph_for_tests should succeed"),
    );
    app.pipeline.retry_last_response();
}

#[tokio::test]
async fn test_retry_no_input() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let sys = crate::domain::model::message::Message::new(
        None,
        "System boot",
        MessageType::System,
        None,
        None,
    );
    let nar = crate::domain::model::message::Message::new(
        None,
        "You see a room.",
        MessageType::Narration,
        None,
        None,
    );
    insert_message_with_swipe(&app, &storage, &sys);
    insert_message_with_swipe(&app, &storage, &nar);

    let before = app.message_service.load_messages().unwrap().len();

    app.pipeline.retry_last_response();

    let after = app.message_service.load_messages().unwrap().len();
    assert_eq!(
        after, before,
        "Retry with no input should be a noop on history (no new messages)"
    );
}

#[tokio::test]
async fn test_retry_event_with_no_pre_event_fallback_to_main() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");

    let mut state = app.message_service.load_or_fresh();
    state.add_message("Event narration".to_string(), None, MessageType::Narration);
    state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _ = storage.save_snapshot(&snapshot);

    app.pipeline.retry_last_response();
}

#[tokio::test]
async fn test_retry_event_with_no_pre_event_and_no_input() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let mut state = app.message_service.load_or_fresh();
    state.add_message("Event only".to_string(), None, MessageType::Narration);
    state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _ = storage.save_snapshot(&snapshot);

    app.pipeline.retry_last_response();
}

#[tokio::test]
async fn test_retry_event_storage_error_on_pre_event() {
    let state = make_test_state();
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    let storage = Arc::new(storage);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _ = storage.save_snapshot(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = storage.insert_message(&msg) {
            if let Some(swipe) = msg.swipes.first() {
                let _ = storage.insert_swipe(id, swipe, 0);
            }
        }
    }

    let wired = crate::test_support::build_test_wired_app(Arc::clone(&storage), make_service())
        .expect("build_test_wired_app: build_app_graph_for_tests should succeed");
    let app = AppState::from_wired(wired);

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_event_id = save_pre_event(&app, &storage);

    handle.set(
        "load_snapshot_by_id",
        TestOverride::internal("simulated load_by_id failure"),
    );

    app.pipeline.retry_last_response();

    handle.clear("load_snapshot_by_id");

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("simulated load_by_id failure"),
        ),
        "Should set error status on storage failure, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_event_missing_trigger_context() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    setup_event_flow_without_trigger(&app, &storage);

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on TriggerMissing, got {other:?}"),
    };
    assert!(
        msg.contains("missing trigger context"),
        "expected canonical 'missing trigger context' in error message, got: {msg}"
    );
}

#[tokio::test]
async fn test_retry_event_continuation_cancels_before_llm() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);

    let mut pre_event_state = app.message_service.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, &storage, last);
    }

    app.shutdown_token.cancel();

    let _ = app.pipeline.retry_event_continuation(&mut pre_event_state);

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Idle),
        "Cancelled retry should reset status to Idle, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_event_trigger_narration_fails() {
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(Arc::new(
            MockBackend::default().with_trigger_narration_fail(),
        )),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("Trigger narration failed"),
        ),
        "Should set error status when trigger narration fails, got {:?}",
        state.narrative.input_buffer.status
    );

    // The System message logged by the failed trigger continuation must
    // survive in history, not be lost to a fresh state load on the error path.
    let has_system_msg = state.narrative.history.iter().any(|m| {
        m.message_type == MessageType::System && m.text().contains("Trigger narration failed")
    });
    assert!(
        has_system_msg,
        "Should persist a System message mentioning 'Trigger narration failed', got history: {:?}",
        state
            .narrative
            .history
            .iter()
            .map(|m| (m.message_type.clone(), m.text().to_string()))
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_retry_event_empty_continuation_text() {
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(Arc::new(MockBackend::new())),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_event_id = save_pre_event(&app, &storage);

    let mut state = app.message_service.load_or_fresh();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::standard());
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _pre_event_with_trigger_id = storage.save_snapshot(&snapshot).unwrap();

    let mut final_state = state;
    final_state.add_message("Event narration".to_string(), None, MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &final_state,
        );
    let _ = storage.save_snapshot(&final_snapshot);

    app.pipeline.retry_last_response();
}

#[tokio::test]
async fn test_retry_main_no_pre_main_snapshot() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let mut state = app.message_service.load_or_fresh();
    let player_name = "Player".to_string();
    state.add_message(
        "test input".to_string(),
        Some(player_name),
        MessageType::Input,
    );
    if let Some(last) = state.narrative.history.last_mut() {
        insert_message_with_swipe(&app, &storage, last);
    }

    let mut state = app.message_service.load_or_fresh();
    state.add_message("Narration text".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _ = storage.save_snapshot(&snapshot);
    if let Some(last) = state.narrative.history.last_mut() {
        insert_message_with_swipe(&app, &storage, last);
    }

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status when anchor message has no snapshot_id"
    );
}

#[tokio::test]
async fn test_retry_event_continuation_happy_path() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);

    let mut pre_event_state = app.message_service.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, &storage, last);
    }

    let mut final_state = pre_event_state;
    final_state.add_message("Event narration".to_string(), None, MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &final_state,
        );
    let _ = storage.save_snapshot(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_snapshot.db_id.unwrap_or(0)));
        insert_message_with_swipe(&app, &storage, last);
    }

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Idle),
        "Should finish with Idle status, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_main_narration_happy_path() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);
    let _final_id = add_narration_and_save(&app, &storage, "Narration text");

    let state = app.message_service.load_or_fresh();
    let _ = app
        .pipeline
        .retry_main_narration(state, "test input".to_string());

    // retry_main_narration is sync (pub(crate) fn) — state is final on return.
    let final_state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Idle
        ),
        "Should finish with Idle status, got {:?}",
        final_state.narrative.input_buffer.status
    );
    let narrations: Vec<_> = final_state
        .narrative
        .history
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert!(
        !narrations.is_empty(),
        "Retry should generate at least one narration (completion), got history: {:?}",
        final_state
            .narrative
            .history
            .iter()
            .map(|m| (m.message_type.clone(), m.text().to_string()))
            .collect::<Vec<_>>()
    );
}

// execute_action + retry_last_response are sync (no spawn, no gate) — no wait seam needed.
// Input is pre-seeded: execute_action runs the pipeline on existing state without adding the input itself.
#[tokio::test]
async fn test_retry_recovers_after_llm_failure() {
    let narrator = Arc::new(MockBackend::default().with_fail_first_n(1));
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(Arc::clone(&narrator) as Arc<_>),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "look");

    app.pipeline.execute_action("look".to_string());
    let after_fail = app.message_service.load_or_fresh();
    assert!(
        after_fail
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "First action should fail, got {:?}",
        after_fail.narrative.input_buffer.status
    );

    app.pipeline.retry_last_response();

    let after_retry = app.message_service.load_or_fresh();
    assert!(
        !after_retry.narrative.input_buffer.status.is_generating(),
        "Retry should complete, got {:?}",
        after_retry.narrative.input_buffer.status
    );
    let narration_count = app
        .message_service
        .load_messages()
        .unwrap()
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narration_count, 1,
        "Retry should produce exactly one narration, got {narration_count}"
    );
}

#[tokio::test]
async fn test_retry_room_not_found_sets_error() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let mut state = app.message_service.load_or_fresh();
    state.add_message(
        "look".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    state.movement.current_room_id = "non_existent_room".to_string();
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(id));
        insert_message_with_swipe(&app, &storage, last);
    }

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("Room not found")),
        "Expected 'Room not found' error, got {:?}",
        state.narrative.input_buffer.status
    );
}

// Main-retry entry path (no event) with a failing narrator → Error. Distinct from
// test_retry_event_trigger_narration_fails, which covers the event-retry trigger path.
#[tokio::test]
async fn test_retry_llm_error_sets_error() {
    let narrator = Arc::new(MockBackend::default().with_fail());
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(Arc::clone(&narrator) as Arc<_>),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);
    let _final_id = add_narration_and_save(&app, &storage, "Narration text");

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Expected Error status after LLM failure, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_empty_narration_sets_error() {
    let narrator = Arc::new(MockBackend::default().with_empty_response());
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(Arc::clone(&narrator) as Arc<_>),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);
    let _final_id = add_narration_and_save(&app, &storage, "Narration text");

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("empty")),
        "Expected 'empty' in error message, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_main_storage_error_on_pre_main() {
    let state = make_test_state();
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    let storage = Arc::new(storage);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _ = storage.save_snapshot(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = storage.insert_message(&msg) {
            if let Some(swipe) = msg.swipes.first() {
                let _ = storage.insert_swipe(id, swipe, 0);
            }
        }
    }

    let wired = crate::test_support::build_test_wired_app(Arc::clone(&storage), make_service())
        .expect("build_test_wired_app: build_app_graph_for_tests should succeed");
    let app = AppState::from_wired(wired);

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);
    let _final_id = add_narration_and_save(&app, &storage, "Narration text");

    handle.set(
        "load_snapshot_by_id",
        TestOverride::internal("simulated load_by_id failure"),
    );

    app.pipeline.retry_last_response();

    handle.clear("load_snapshot_by_id");

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status on storage failure"
    );
}

struct EmptyTriggerBackend;

impl crate::application::ports::llm_provider::LlmProvider for EmptyTriggerBackend {
    fn model(&self) -> &str {
        "mock"
    }

    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(LlmCallResult {
            text: String::new(),
            raw_request_json: String::new(),
            raw_response_json: String::new(),
            backend_name: "EmptyTrigger".to_string(),
            model_name: "mock".to_string(),
            agent_name: String::new(),
        })
    }

    fn name(&self) -> &str {
        "EmptyTrigger"
    }
}

#[tokio::test]
async fn test_retry_event_empty_continuation_triggers_error() {
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(Arc::new(EmptyTriggerBackend)),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);

    let mut pre_event_state = app.message_service.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, &storage, last);
    }

    let mut final_state = pre_event_state;
    final_state.add_message("Event narration".to_string(), None, MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &final_state,
        );
    let final_id = storage.save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(&app, &storage, last);
    }
    app.pipeline.retry_last_response();
    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Error(ref msg) if msg.contains("empty response")),
        "Should set error status when continuation text is empty, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_appends_swipe_to_same_message() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);
    let _narration_id = add_narration_and_save(&app, &storage, "Narration text");

    let msgs = app.message_service.load_messages().unwrap();
    let narration_msg = msgs
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .unwrap();
    let original_id = narration_msg.id;
    let extra_swipe = crate::domain::model::message::Swipe {
        text: "Alt narration".to_string(),
        snapshot_id: Some(_pre_main_id),
        location_header: None,
        event_header: None,
    };
    storage
        .insert_swipe(narration_msg.id, &extra_swipe, 1)
        .unwrap();

    app.pipeline.retry_last_response();

    let msgs = app.message_service.load_messages().unwrap();
    let narration = msgs
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .expect("Should have a narration after retry");
    assert_eq!(
        narration.id, original_id,
        "Retry should keep the same message ID"
    );
    assert_eq!(
        narration.swipes.len(),
        3,
        "Retry should append a new swipe to the existing message, got {} swipes",
        narration.swipes.len()
    );
    assert_eq!(
        narration.active_swipe_index, 2,
        "Active swipe should be the newly appended one"
    );
}

#[tokio::test]
async fn test_retrigger_event_cancels_cleanly() {
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);

    let mut pre_event_state = app.message_service.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, &storage, last);
    }

    let mut final_state = pre_event_state;
    final_state.add_message("Event narration".to_string(), None, MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &final_state,
        );
    let final_id = storage.save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(&app, &storage, last);
    }

    app.shutdown_token.cancel();

    app.pipeline.retrigger_event();

    let state = app.message_service.load_or_fresh();
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancelled retrigger should reset status to Idle, got {:?}",
        state.narrative.input_buffer.status
    );
    assert_eq!(
        state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Cancelled retrigger should reset phase to default"
    );
}

#[tokio::test]
async fn test_retry_event_continuation_returns_ok_on_world_fetch_failure() {
    let data = TestDataBuilder::default_test().build();
    let (storage, handle) = {
        let base = Storage::new_in_memory();
        data.seed_into(&base);
        base.with_test_failures()
    };
    handle.set(
        "get_world",
        TestOverride::internal("simulated get_world failure"),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .pipeline(service)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on world fetch failure, got {other:?}"),
    };
    assert!(
        msg.contains("simulated get_world failure"),
        "expected the injected storage error to flow through PhaseError::FetchFailed -> finalize_phase_error, got: {msg}"
    );
}

#[tokio::test]
async fn test_retry_event_continuation_returns_ok_on_persona_fetch_failure() {
    let data = TestDataBuilder::default_test().build();
    let (storage, handle) = {
        let base = Storage::new_in_memory();
        data.seed_into(&base);
        base.with_test_failures()
    };
    handle.set(
        "get_persona",
        TestOverride::internal("simulated get_persona failure"),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .pipeline(service)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on persona fetch failure, got {other:?}"),
    };
    assert!(
        msg.contains("simulated get_persona failure"),
        "expected the injected storage error to flow through PhaseError::FetchFailed -> finalize_phase_error, got: {msg}"
    );
}

#[tokio::test]
async fn retry_records_canonical_game_not_found_when_game_missing() {
    let storage = {
        let base = Storage::new_in_memory();
        let data = TestDataBuilder::default_test().build();
        data.seed_into(&base);
        base.set_game_id(999);
        base
    };
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::with_data(TestDataBuilder::default_test().build())
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .pipeline(service)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error, got {other:?}"),
    };
    assert!(
        msg.contains("Game not found: 999"),
        "expected canonical 'Game not found: 999' in error message, got: {msg}"
    );
}

#[tokio::test]
async fn test_retry_last_response_cancelled_at_phase_boundary() {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::thread;

    let mock_backend = Arc::new(MockBackend::default().with_trigger_delay(200));
    let narrator_recorder = make_test_recorder(Arc::clone(&mock_backend) as Arc<_>);
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    let initial_game_id = app.game_catalogue.current_game_id();
    let storage_for_thread = Arc::clone(&storage);
    let backend_for_thread = Arc::clone(&mock_backend);

    let flipper = thread::spawn(move || {
        while !backend_for_thread.trigger_started.load(Ordering::SeqCst) {
            std::hint::spin_loop();
        }
        storage_for_thread.set_game_id(initial_game_id.wrapping_add(1));
    });

    app.pipeline.retry_last_response();

    flipper
        .join()
        .expect("game-id flipper thread should complete");

    let state = app.message_service.load_or_fresh();
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancelled retry must reset status to Idle via the orchestrator's \
         Cancelled match arm, got {:?}",
        state.narrative.input_buffer.status
    );
    assert_eq!(
        state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Cancelled retry must reset phase to default, got {:?}",
        state.narrative.input_buffer.phase
    );
}

#[tokio::test]
async fn test_retrigger_event_cancelled_at_phase_boundary() {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::thread;

    let mock_backend = Arc::new(MockBackend::default().with_trigger_delay(200));
    let narrator_recorder = make_test_recorder(Arc::clone(&mock_backend) as Arc<_>);
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    let initial_game_id = app.game_catalogue.current_game_id();
    let storage_for_thread = Arc::clone(&storage);
    let backend_for_thread = Arc::clone(&mock_backend);

    let flipper = thread::spawn(move || {
        while !backend_for_thread.trigger_started.load(Ordering::SeqCst) {
            std::hint::spin_loop();
        }
        storage_for_thread.set_game_id(initial_game_id.wrapping_add(1));
    });

    app.pipeline.retrigger_event();

    flipper
        .join()
        .expect("game-id flipper thread should complete");

    let state = app.message_service.load_or_fresh();
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancelled retrigger must reset status to Idle via the \
         orchestrator's Cancelled match arm, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retrigger_event_emits_error_on_world_fetch_failure() {
    let data = TestDataBuilder::default_test().build();
    let (storage, handle) = {
        let base = Storage::new_in_memory();
        data.seed_into(&base);
        base.with_test_failures()
    };
    handle.set(
        "get_world",
        TestOverride::internal("simulated get_world failure"),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .pipeline(service)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    app.pipeline.retrigger_event();

    let state = app.message_service.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => {
            panic!("expected GenerationStatus::Error on retrigger fetch failure, got {other:?}")
        }
    };
    assert!(
        msg.contains("simulated get_world failure"),
        "expected FetchFailed(msg) to flow through finalize_phase_error, got: {msg}"
    );
}

#[tokio::test]
async fn test_retry_event_continuation_handles_state_without_input_message() {
    let (app, _storage) = TestAppBuilder::default_test().build_service_with_storage();

    let mut state = app.message_service.load_or_fresh();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::standard());
    state.add_message(
        "Narration without prior input".to_string(),
        None,
        MessageType::Narration,
    );

    let _ = app.pipeline.retry_event_continuation(&mut state);
}

#[tokio::test]
async fn test_retry_records_missing_snapshot_id() {
    const MISSING_ID: u64 = 99_999;

    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let mut state = app.message_service.load_or_fresh();
    let player_name = "Player".to_string();
    state.add_message(
        "test input".to_string(),
        Some(player_name),
        MessageType::Input,
    );
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(MISSING_ID));
        insert_message_with_swipe(&app, &storage, last);
    }

    app.pipeline.retry_last_response();

    let state = app.message_service.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on missing snapshot, got {other:?}"),
    };
    assert!(
        msg.contains(&format!("no snapshot found for id {MISSING_ID}")),
        "expected typed 'no snapshot found for id {MISSING_ID}' error, got: {msg}"
    );
}

#[tokio::test]
async fn test_retry_returns_internal_error_when_anchor_has_no_snapshot_id() {
    // anchor message without snapshot_id is a data-integrity
    // violation → 500 from retry(), with the reason persisted on status.
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let mut state = app.message_service.load_or_fresh();
    state.add_message(
        "test input".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let last = state.narrative.history.last().unwrap();
    insert_message_with_swipe(&app, &storage, last);

    let result = app.pipeline.retry(&app.generation_gate);

    assert!(
        matches!(
            result,
            Err(ApplicationError::Engine(EngineError::Internal(_)))
        ),
        "retry() should return ApplicationError::internal (500) when anchor has no snapshot_id, got {result:?}"
    );

    let state = app.message_service.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("no snapshot_id")),
        "Should persist Error status indicating the snapshot is missing, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_returns_internal_error_when_snapshot_row_missing() {
    const MISSING_ID: u64 = 99_999;
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();

    let mut state = app.message_service.load_or_fresh();
    state.add_message(
        "test input".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(MISSING_ID));
        insert_message_with_swipe(&app, &storage, last);
    }

    let result = app.pipeline.retry(&app.generation_gate);

    assert!(
        matches!(
            result,
            Err(ApplicationError::Engine(EngineError::Internal(_)))
        ),
        "retry() should return ApplicationError::internal (500) when snapshot row missing, got {result:?}"
    );

    let state = app.message_service.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on missing snapshot, got {other:?}"),
    };
    assert!(
        msg.contains(&format!("no snapshot found for id {MISSING_ID}")),
        "expected 'no snapshot found for id {MISSING_ID}' error, got: {msg}"
    );
}

#[tokio::test]
async fn test_retry_returns_concurrent_generation_when_gate_busy() {
    // retry() must reject concurrent generation the same way
    // `process_action` does — Ok(ConcurrentGeneration), no task spawned.
    let (app, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let _input_id = add_input_and_save(&app, &storage, "test input");

    let game_id = storage.current_game_id();
    let mut state = app.message_service.load_or_fresh();
    let (_, _, claim) = app
        .generation_gate
        .try_claim(game_id, &mut state, app.message_service.as_ref())
        .expect("pre-claim should succeed");
    assert!(matches!(claim, ProcessActionResult::Started));

    let result = app.pipeline.retry(&app.generation_gate);

    assert!(
        matches!(result, Ok(ProcessActionResult::ConcurrentGeneration)),
        "retry() should return Ok(ConcurrentGeneration) when gate is busy, got {result:?}"
    );
}

#[tokio::test]
async fn test_retrigger_returns_concurrent_generation_when_gate_busy() {
    // retrigger() must reject concurrent generation the same way
    // `process_action` does — Ok(ConcurrentGeneration), no task spawned.
    let (app, storage) = TestAppBuilder::default_test()
        .last_trigger(crate::test_support::TestStoredTriggerContext::standard())
        .log("Main narration", None, MessageType::Narration)
        .build_service_with_storage();

    let game_id = storage.current_game_id();
    let mut state = app.message_service.load_or_fresh();
    let (_, _, claim) = app
        .generation_gate
        .try_claim(game_id, &mut state, app.message_service.as_ref())
        .expect("pre-claim should succeed");
    assert!(matches!(claim, ProcessActionResult::Started));

    let result = app.pipeline.retrigger(&app.generation_gate);

    assert!(
        matches!(result, Ok(ProcessActionResult::ConcurrentGeneration)),
        "retrigger() should return Ok(ConcurrentGeneration) when gate is busy, got {result:?}"
    );
}

// The is_shutting_down() guard runs before any state load, so no seeding is needed.
#[tokio::test]
async fn test_retry_returns_shutting_down_when_token_cancelled() {
    let (app, _storage) = TestAppBuilder::default_test().build_service_with_storage();
    app.shutdown_token.cancel();

    let result = app.pipeline.retry(&app.generation_gate);
    assert!(
        matches!(result, Ok(ProcessActionResult::ShuttingDown)),
        "retry() should return Ok(ShuttingDown) when token is cancelled, got {result:?}"
    );
}

#[tokio::test]
async fn test_retrigger_returns_shutting_down_when_token_cancelled() {
    let (app, _storage) = TestAppBuilder::default_test().build_service_with_storage();
    app.shutdown_token.cancel();

    let result = app.pipeline.retrigger(&app.generation_gate);
    assert!(
        matches!(result, Ok(ProcessActionResult::ShuttingDown)),
        "retrigger() should return Ok(ShuttingDown) when token is cancelled, got {result:?}"
    );
}
