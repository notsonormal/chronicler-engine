use std::sync::Arc;

use crate::application::action_pipeline::retry::{retry_event_continuation, retry_main_narration};

#[allow(unused_imports)]
use crate::application::game_service::GameService;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::application::application_service::DefaultApplicationService;
use crate::application::ports::llm_provider::LlmCallResult;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;
use crate::application::agents::registry::AgentRegistry;
use crate::test_support::fixtures::TestGameState;
use crate::test_support::make_test_recorder;
use crate::test_support::TestAppBuilder;
use crate::test_support::TestDataBuilder;
use crate::test_support::make_test_app_without_snapshot;

fn make_test_state() -> GameState {
    TestGameState::in_room("start")
}

fn make_service() -> GameService {
    GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(MockBackend::default())),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    )
}

fn insert_message_with_swipe(
    app: &DefaultApplicationService,
    msg: &crate::domain::model::message::Message,
) {
    let id = app.storage().insert_message(msg).unwrap();
    if let Some(swipe) = msg.swipes.first() {
        let mut swipe = swipe.clone();
        swipe.text = msg.text().to_string();
        swipe.snapshot_id = msg.snapshot_id();
        swipe.location_header = msg.location_header().map(|s| s.to_string());
        swipe.event_header = msg.event_header().map(|s| s.to_string());
        let _ = app.storage().insert_swipe(id, &swipe, 0);
    }
}

fn add_input_and_save(app: &DefaultApplicationService, text: &str) -> u64 {
    let mut state = app.load_or_fresh();
    let player_name = "Player".to_string();
    state.add_message(text.to_string(), Some(player_name), MessageType::Input);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let id = app.storage().save_snapshot(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(id));
        insert_message_with_swipe(app, last);
    }
    id
}

fn add_narration_and_save(app: &DefaultApplicationService, text: &str) -> u64 {
    let mut state = app.load_or_fresh();
    state.add_message(text.to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let id = app.storage().save_snapshot(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(id));
        insert_message_with_swipe(app, last);
    }
    id
}

fn save_pre_main(app: &DefaultApplicationService) -> u64 {
    let state = app.load_or_fresh();
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    app.storage().save_snapshot(&snapshot).unwrap()
}

fn save_pre_event(app: &DefaultApplicationService) -> u64 {
    let state = app.load_or_fresh();
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    app.storage().save_snapshot(&snapshot).unwrap()
}

/// Build event-flow state in storage so `find_retry_anchor` + `load_messages`
/// both succeed; tests then drive the failure mode they care about (fetch
/// failure, trigger missing, etc.) via `handle.set(...)` or omitting the
/// `last_trigger` setup.
fn setup_event_flow(app: &DefaultApplicationService) {
    let _ = add_input_and_save(app, "test input");
    let _ = save_pre_main(app);

    let mut pre_event_state = app.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = app.storage().save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(app, last);
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
    let final_id = app.storage().save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(app, last);
    }
}

/// Event-flow state machine with NO `last_trigger` set on the pre-event
/// snapshot — drives the `PhaseError::TriggerMissing` return in
/// `retry_event_continuation`. The pre-event snapshot is built from the
/// state right after `add_input_and_save`, which leaves `last_trigger = None`.
fn setup_event_flow_without_trigger(app: &DefaultApplicationService) {
    let _ = add_input_and_save(app, "test input");
    let _ = save_pre_main(app);

    let mut pre_event_state = app.load_or_fresh();
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = app.storage().save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(app, last);
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
    let final_id = app.storage().save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(app, last);
    }
}

#[test]
fn test_retry_no_snapshot() {
    let state = make_test_state();
    let wired = make_test_app_without_snapshot(state).unwrap();
    let app = &wired.application_service;
    app.retry_last_response();

    let state = app.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Error(ref msg) if msg.contains("Retry failed: no anchor message")),
        "Should record retry error when no anchor message exists, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_load_messages_error() {
    let _state = make_test_state();
    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        "load_message_rows",
        TestOverride::internal("simulated load_message_rows failure"),
    );

    let app = Arc::new(DefaultApplicationService::new(
        failing,
        Arc::new(crate::adapters::driven::storage::Storage::new_in_memory()),
        Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        tokio_util::sync::CancellationToken::new(),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::new(make_service()),
    ));
    app.retry_last_response();
}

#[test]
fn test_retry_no_input() {
    let app = TestAppBuilder::default_test().build_service();
    app.retry_last_response();
}

#[test]
fn test_retry_event_with_no_pre_event_fallback_to_main() {
    let app = TestAppBuilder::default_test().build_service();

    let _input_id = add_input_and_save(&app, "test input");

    let mut state = app.load_or_fresh();
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
    let _ = app.storage().save_snapshot(&snapshot);

    app.retry_last_response();
}

#[test]
fn test_retry_event_with_no_pre_event_and_no_input() {
    let app = TestAppBuilder::default_test().build_service();

    let mut state = app.load_or_fresh();
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
    let _ = app.storage().save_snapshot(&snapshot);

    app.retry_last_response();
}

#[test]
fn test_retry_event_storage_error_on_pre_event() {
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

    let app = Arc::new(DefaultApplicationService::new(
        Arc::clone(&storage),
        Arc::new(crate::adapters::driven::storage::Storage::new_in_memory()),
        Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        tokio_util::sync::CancellationToken::new(),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::new(make_service()),
    ));

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_event_id = save_pre_event(&app);

    handle.set(
        "load_snapshot_by_id",
        TestOverride::internal("simulated load_by_id failure"),
    );

    app.retry_last_response();

    handle.clear("load_snapshot_by_id");

    let state = app.load_or_fresh();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("simulated load_by_id failure"),
        ),
        "Should set error status on storage failure, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_event_missing_trigger_context() {
    let app = TestAppBuilder::default_test().build_service();

    setup_event_flow_without_trigger(&app);

    app.retry_last_response();

    let state = app.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on TriggerMissing, got {other:?}"),
    };
    assert!(
        msg.contains("missing trigger context"),
        "expected canonical 'missing trigger context' in error message, got: {msg}"
    );
}

#[test]
fn test_retry_event_continuation_cancels_before_llm() {
    let app = TestAppBuilder::default_test().build_service();

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_main_id = save_pre_main(&app);

    let mut pre_event_state = app.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = app.storage().save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, last);
    }

    app.cancel_token().cancel();

    let _ = retry_event_continuation(&app, pre_event_state);

    let state = app.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Idle),
        "Cancelled retry should reset status to Idle, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_event_trigger_narration_fails() {
    let game_service = Arc::new(GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(
            MockBackend::default().with_trigger_narration_fail(),
        )),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    ));
    let app = TestAppBuilder::default_test()
        .game_service(game_service)
        .build_service();

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_event_id = save_pre_event(&app);

    let mut state = app.load_or_fresh();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::standard());
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _pre_event_with_trigger_id = app.storage().save_snapshot(&snapshot).unwrap();

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
    let final_id = app.storage().save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(&app, last);
    }

    app.retry_last_response();

    let state = app.load_or_fresh();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status when trigger narration fails, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_event_empty_continuation_text() {
    let game_service = Arc::new(GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(MockBackend::new())),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    ));
    let app = TestAppBuilder::default_test()
        .game_service(game_service)
        .build_service();

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_event_id = save_pre_event(&app);

    let mut state = app.load_or_fresh();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::standard());
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _pre_event_with_trigger_id = app.storage().save_snapshot(&snapshot).unwrap();

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
    let _ = app.storage().save_snapshot(&final_snapshot);

    app.retry_last_response();
}

#[test]
fn test_retry_main_no_pre_main_snapshot() {
    let app = TestAppBuilder::default_test().build_service();

    let mut state = app.load_or_fresh();
    let player_name = "Player".to_string();
    state.add_message(
        "test input".to_string(),
        Some(player_name),
        MessageType::Input,
    );
    if let Some(last) = state.narrative.history.last_mut() {
        insert_message_with_swipe(&app, last);
    }

    let mut state = app.load_or_fresh();
    state.add_message("Narration text".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _ = app.storage().save_snapshot(&snapshot);
    if let Some(last) = state.narrative.history.last_mut() {
        insert_message_with_swipe(&app, last);
    }

    app.retry_last_response();

    let state = app.load_or_fresh();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status when anchor message has no snapshot_id"
    );
}

#[test]
fn test_retry_event_continuation_happy_path() {
    let app = TestAppBuilder::default_test().build_service();

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_main_id = save_pre_main(&app);

    let mut pre_event_state = app.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = app.storage().save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, last);
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
    let _ = app.storage().save_snapshot(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_snapshot.db_id.unwrap_or(0)));
        insert_message_with_swipe(&app, last);
    }

    app.retry_last_response();

    let state = app.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Idle),
        "Should finish with Idle status, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_main_narration_happy_path() {
    let app = TestAppBuilder::default_test().build_service();

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_main_id = save_pre_main(&app);
    let _final_id = add_narration_and_save(&app, "Narration text");

    let state = app.load_or_fresh();
    let _ = retry_main_narration(&app, state, "test input".to_string());
}

#[test]
fn test_retry_main_storage_error_on_pre_main() {
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

    let app = Arc::new(DefaultApplicationService::new(
        Arc::clone(&storage),
        Arc::new(crate::adapters::driven::storage::Storage::new_in_memory()),
        Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        tokio_util::sync::CancellationToken::new(),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::new(make_service()),
    ));

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_main_id = save_pre_main(&app);
    let _final_id = add_narration_and_save(&app, "Narration text");

    handle.set(
        "load_snapshot_by_id",
        TestOverride::internal("simulated load_by_id failure"),
    );

    app.retry_last_response();

    handle.clear("load_snapshot_by_id");

    let state = app.load_or_fresh();
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

#[test]
fn test_retry_event_empty_continuation_triggers_error() {
    let game_service = Arc::new(GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(EmptyTriggerBackend)),
        Arc::new(MockBackend::default())
            as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    ));
    let app = TestAppBuilder::default_test()
        .game_service(game_service)
        .build_service();

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_main_id = save_pre_main(&app);

    let mut pre_event_state = app.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = app.storage().save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, last);
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
    let final_id = app.storage().save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(&app, last);
    }
    app.retry_last_response();
    let state = app.load_or_fresh();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Error(ref msg) if msg.contains("empty response")),
        "Should set error status when continuation text is empty, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_appends_swipe_to_same_message() {
    let app = TestAppBuilder::default_test().build_service();

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_main_id = save_pre_main(&app);
    let _narration_id = add_narration_and_save(&app, "Narration text");

    let msgs = app.load_messages().unwrap();
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
    app.storage()
        .insert_swipe(narration_msg.id, &extra_swipe, 1)
        .unwrap();

    app.retry_last_response();

    let msgs = app.load_messages().unwrap();
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

#[test]
fn test_retrigger_event_cancels_cleanly() {
    let app = TestAppBuilder::default_test().build_service();

    let _input_id = add_input_and_save(&app, "test input");
    let _pre_main_id = save_pre_main(&app);

    let mut pre_event_state = app.load_or_fresh();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = app.storage().save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, last);
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
    let final_id = app.storage().save_snapshot(&final_snapshot).unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(&app, last);
    }

    app.cancel_token().cancel();

    app.retrigger_event();

    let state = app.load_or_fresh();
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

// B2 fail-loud (orchestrator-level, ticket 04 seam): world fetch failure
// during retry-event must surface as `GenerationStatus::Error` via
// `ActionPipeline::finalize_phase_error` consuming `PhaseError::FetchFailed(msg)`.

#[test]
fn test_retry_event_continuation_returns_ok_on_world_fetch_failure() {
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
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let app = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .game_service(Arc::new(service))
        .build_service();

    setup_event_flow(&app);

    app.retry_last_response();

    let state = app.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on world fetch failure, got {other:?}"),
    };
    assert!(
        msg.contains("simulated get_world failure"),
        "expected the injected storage error to flow through PhaseError::FetchFailed -> finalize_phase_error, got: {msg}"
    );
}

// B2 fail-loud (orchestrator-level, ticket 04 seam): persona fetch failure,
// same seam as the world-fetch test above.

#[test]
fn test_retry_event_continuation_returns_ok_on_persona_fetch_failure() {
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
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let app = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .game_service(Arc::new(service))
        .build_service();

    setup_event_flow(&app);

    app.retry_last_response();

    let state = app.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on persona fetch failure, got {other:?}"),
    };
    assert!(
        msg.contains("simulated get_persona failure"),
        "expected the injected storage error to flow through PhaseError::FetchFailed -> finalize_phase_error, got: {msg}"
    );
}

// Required-read migration: missing game row must surface as canonical
// `EngineError::GameNotFound`. Full chain (ticket 04):
// `require_game(999)` -> `Err(GameNotFound(999))` -> `PhaseError::FetchFailed(e.to_string())`
// -> `finalize_phase_error` -> `GenerationStatus::Error(msg)`.

#[test]
fn retry_records_canonical_game_not_found_when_game_missing() {
    let storage = {
        let base = Storage::new_in_memory();
        let data = TestDataBuilder::default_test().build();
        // Seed world + persona + npcs so the IIFE bundle-load could proceed past
        // `require_game` if it ever got that far. The missing-game path is what
        // we want to lock in here.
        data.seed_into(&base);
        // Override `seed_into`'s `set_game_id(1)` — there is no game row at id
        // 999, so `require_game(999)` will return `Err(GameNotFound(999))`.
        base.set_game_id(999);
        base
    };
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let app = TestAppBuilder::with_data(TestDataBuilder::default_test().build())
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .game_service(Arc::new(service))
        .build_service();

    setup_event_flow(&app);

    app.retry_last_response();

    let state = app.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error, got {other:?}"),
    };
    assert!(
        msg.contains("Game not found: 999"),
        "expected canonical 'Game not found: 999' in error message, got: {msg}"
    );
}

// Cancellation seam coverage: drive the `Err(PhaseError::Cancelled)` arm
// of `retry_last_response` by flipping `app.set_game_id` mid-pipeline
// (the α-check in `phase_trigger_continuation_llm_call` returns Cancelled
// when the game id changes during the LLM call's sleep window).

#[test]
fn test_retry_last_response_cancelled_at_phase_boundary() {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let mock_backend = Arc::new(MockBackend::default().with_trigger_delay(200));
    let narrator_recorder = make_test_recorder(Arc::clone(&mock_backend) as Arc<_>);
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let app = TestAppBuilder::default_test()
        .game_service(Arc::new(service))
        .build_service();

    setup_event_flow(&app);

    let initial_game_id = app.current_game_id();
    let app_for_thread = Arc::clone(&app);
    let flipper = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        app_for_thread.set_game_id(initial_game_id.wrapping_add(1));
    });

    app.retry_last_response();

    flipper
        .join()
        .expect("game-id flipper thread should complete");

    let state = app.load_or_fresh();
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

// Same game-id-flip trick for `retrigger_event`: drives the
// `Err(PhaseError::Cancelled)` arm in its bottom match block, which the
// existing `test_retrigger_event_cancels_cleanly` only covered by
// accident (phase_finalize reset status to Idle after a successful run).

#[test]
fn test_retrigger_event_cancelled_at_phase_boundary() {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let mock_backend = Arc::new(MockBackend::default().with_trigger_delay(200));
    let narrator_recorder = make_test_recorder(Arc::clone(&mock_backend) as Arc<_>);
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let app = TestAppBuilder::default_test()
        .game_service(Arc::new(service))
        .build_service();

    setup_event_flow(&app);

    let initial_game_id = app.current_game_id();
    let app_for_thread = Arc::clone(&app);
    let flipper = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        app_for_thread.set_game_id(initial_game_id.wrapping_add(1));
    });

    app.retrigger_event();

    flipper
        .join()
        .expect("game-id flipper thread should complete");

    let state = app.load_or_fresh();
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancelled retrigger must reset status to Idle via the \
         orchestrator's Cancelled match arm, got {:?}",
        state.narrative.input_buffer.status
    );
}

// `retrigger_event`'s `Err(e) => finalize_phase_error` arm: drives
// `retry_event_continuation` to return `Err(PhaseError::FetchFailed(msg))`
// by injecting a `get_world` storage failure and confirming the orchestrator
// surfaces it as `GenerationStatus::Error` with the underlying message.

#[test]
fn test_retrigger_event_emits_error_on_world_fetch_failure() {
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
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let app = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .game_service(Arc::new(service))
        .build_service();

    setup_event_flow(&app);

    app.retrigger_event();

    let state = app.load_or_fresh();
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

// `retry_event_continuation`'s `last_input_text()` None arm: when the
// loaded state has no Input messages, fall back to an empty string so
// downstream stages still see a valid value.

#[test]
fn test_retry_event_continuation_handles_state_without_input_message() {
    let app = TestAppBuilder::default_test().build_service();

    let mut state = app.load_or_fresh();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::standard());
    state.add_message(
        "Narration without prior input".to_string(),
        None,
        MessageType::Narration,
    );

    let _ = crate::application::action_pipeline::retry::retry_event_continuation(&app, state);
}

// Snapshot referenced by the anchor message's swipe no longer exists in
// storage — `load_snapshot_by_id` returns `Ok(None)`. The retry
// orchestrator must record the missing snapshot as a typed error rather
// than panicking on the unwrap.

#[test]
fn test_retry_records_missing_snapshot_id() {
    const MISSING_ID: u64 = 99_999;

    let app = TestAppBuilder::default_test().build_service();

    let mut state = app.load_or_fresh();
    let player_name = "Player".to_string();
    state.add_message(
        "test input".to_string(),
        Some(player_name),
        MessageType::Input,
    );
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(MISSING_ID));
        insert_message_with_swipe(&app, last);
    }

    app.retry_last_response();

    let state = app.load_or_fresh();
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error on missing snapshot, got {other:?}"),
    };
    assert!(
        msg.contains(&format!("no snapshot found for id {MISSING_ID}")),
        "expected typed 'no snapshot found for id {MISSING_ID}' error, got: {msg}"
    );
}
