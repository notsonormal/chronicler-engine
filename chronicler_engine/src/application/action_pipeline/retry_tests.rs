use std::sync::Arc;

use crate::application::action_pipeline::retry::{
    retry_event_continuation, retry_last_response_impl, retry_main_narration,
};
use crate::application::context::GameServiceContext;

#[allow(unused_imports)]
use crate::application::game_service::DefaultGameService;
use crate::error::EngineError;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, MessageType};
use crate::narrative::llm::MockBackend;
use crate::narrative::llm::backend::LlmCallResult;
use crate::storage::{Operation, Storage, TestOverride};
use crate::test_support::fixtures::{TestMap, TestNpc, TestPlayer, TestWorld};
use crate::test_support::make_test_context_with_sqlite;

fn make_test_state() -> GameState {
    let world = Arc::new(TestWorld::minimal());
    let map = Arc::new(TestMap::single_room("start"));
    let player = Arc::new(TestPlayer::standard());
    let npcs = vec![TestNpc::named("npc1", "Test NPC")];
    GameState::new(world, map, player, npcs, "start".to_string())
}

fn make_service() -> DefaultGameService {
    DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(None)),
        Arc::new(crate::narrative::llm::MockBackend::default()),
    )
}

fn insert_message_with_swipe(ctx: &GameServiceContext, msg: &crate::model::message::Message) {
    let id = ctx.storage.insert_message(msg).unwrap();
    if let Some(swipe) = msg.swipes.first() {
        let mut swipe = swipe.clone();
        swipe.text = msg.text().to_string();
        swipe.snapshot_id = msg.snapshot_id();
        swipe.location_header = msg.location_header().map(|s| s.to_string());
        swipe.event_header = msg.event_header().map(|s| s.to_string());
        let _ = ctx.storage.insert_swipe(id, &swipe, 0);
    }
}

fn add_input_and_save(ctx: &GameServiceContext, text: &str) -> u64 {
    let mut state = ctx.load_state_for_test();
    let player_name = state.player.sheet.name.clone();
    state.add_message(text.to_string(), Some(player_name), MessageType::Input);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let id = ctx.storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(id));
        insert_message_with_swipe(ctx, last);
    }
    id
}

fn add_narration_and_save(ctx: &GameServiceContext, text: &str) -> u64 {
    let mut state = ctx.load_state_for_test();
    state.add_message(text.to_string(), None, MessageType::Narration);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let id = ctx.storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(id));
        insert_message_with_swipe(ctx, last);
    }
    id
}

fn save_pre_main(ctx: &GameServiceContext) -> u64 {
    let state = ctx.load_state_for_test();
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    ctx.storage.save_snapshot(&snapshot).unwrap()
}

fn save_pre_event(ctx: &GameServiceContext) -> u64 {
    let state = ctx.load_state_for_test();
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    ctx.storage.save_snapshot(&snapshot).unwrap()
}


#[test]
fn test_retry_no_snapshot() {
    let state = make_test_state();
    let ctx = crate::test_support::make_test_context_without_snapshot(state);
    let service = make_service();
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_load_messages_error() {
    let state = make_test_state();
    let base_ctx = make_test_context_with_sqlite(state).unwrap();

    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        Operation::LoadMessageRows,
        TestOverride::internal("simulated load_message_rows failure"),
    );

    let ctx = GameServiceContext {
        storage: failing,
        ..base_ctx.clone()
    };

    let service = make_service();
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_no_input() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_event_with_no_pre_event_fallback_to_main() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let _input_id = add_input_and_save(&ctx, "test input");

    let mut state = ctx.load_state_for_test();
    state.add_message("Event narration".to_string(), None, MessageType::Narration);
    state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&snapshot);

    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_event_with_no_pre_event_and_no_input() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let mut state = ctx.load_state_for_test();
    state.add_message("Event only".to_string(), None, MessageType::Narration);
    state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&snapshot);

    retry_last_response_impl(&service, ctx);
}


#[test]
fn test_retry_event_storage_error_on_pre_event() {
    let state = make_test_state();
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    let storage = Arc::new(storage);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = storage.save_snapshot(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = storage.insert_message(&msg) {
            if let Some(swipe) = msg.swipes.first() {
                let _ = storage.insert_swipe(id, swipe, 0);
            }
        }
    }
    let base_ctx = GameServiceContext {
        storage: Arc::clone(&storage),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(Storage::new_in_memory()),
    };

    let _input_id = add_input_and_save(&base_ctx, "test input");
    let _pre_event_id = save_pre_event(&base_ctx);

    handle.set(
        Operation::LoadSnapshotById,
        TestOverride::internal("simulated load_by_id failure"),
    );

    let service = make_service();
    let latest = base_ctx.load_state_for_test();

    let _ = retry_event_continuation(&service, &base_ctx, latest);

    handle.clear(Operation::LoadSnapshotById);

    let state = base_ctx.load_state_for_test();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status on storage failure"
    );
}

#[test]
fn test_retry_event_missing_trigger_context() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_event_id = save_pre_event(&ctx);

    let mut state = ctx.load_state_for_test();
    state.add_message("Event narration".to_string(), None, MessageType::Narration);
    state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&snapshot);

    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_event_continuation_cancels_before_llm() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_main_id = save_pre_main(&ctx);

    let mut pre_event_state = ctx.load_state_for_test();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&pre_event_state);
    let pre_event_id = ctx.storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&ctx, last);
    }

    ctx.cancel_token.cancel();

    let _ = retry_event_continuation(&service, &ctx, pre_event_state);

    let state = ctx.load_state_for_test();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Idle),
        "Cancelled retry should reset status to Idle, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_event_trigger_narration_fails() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let llm = Arc::new(MockBackend::with_failing_trigger_narration());
    let service = DefaultGameService::with_mock_quantifier(
        llm,
        Arc::new(crate::narrative::llm::MockBackend::default()),
    );

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_event_id = save_pre_event(&ctx);

    let mut state = ctx.load_state_for_test();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::standard());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _pre_event_with_trigger_id = ctx.storage.save_snapshot(&snapshot).unwrap();

    let mut final_state = state;
    final_state.add_message("Event narration".to_string(), None, MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.storage.save_snapshot(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        insert_message_with_swipe(&ctx, last);
    }

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state_for_test();
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
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let llm = Arc::new(MockBackend::new(None));
    let service = DefaultGameService::with_mock_quantifier(
        llm,
        Arc::new(crate::narrative::llm::MockBackend::default()),
    );

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_event_id = save_pre_event(&ctx);

    let mut state = ctx.load_state_for_test();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::standard());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _pre_event_with_trigger_id = ctx.storage.save_snapshot(&snapshot).unwrap();

    let mut final_state = state;
    final_state.add_message("Event narration".to_string(), None, MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.storage.save_snapshot(&final_snapshot);

    retry_last_response_impl(&service, ctx);
}


#[test]
fn test_retry_main_no_pre_main_snapshot() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let mut state = ctx.load_state_for_test();
    let player_name = state.player.sheet.name.clone();
    state.add_message(
        "test input".to_string(),
        Some(player_name),
        MessageType::Input,
    );
    if let Some(last) = state.narrative.history.last_mut() {
        insert_message_with_swipe(&ctx, last);
    }

    let mut state = ctx.load_state_for_test();
    state.add_message("Narration text".to_string(), None, MessageType::Narration);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&snapshot);
    if let Some(last) = state.narrative.history.last_mut() {
        insert_message_with_swipe(&ctx, last);
    }

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state_for_test();
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
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = make_service();

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_main_id = save_pre_main(&ctx);

    let mut pre_event_state = ctx.load_state_for_test();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&pre_event_state);
    let pre_event_id = ctx.storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&ctx, last);
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
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.storage.save_snapshot(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_snapshot.db_id.unwrap_or(0)));
        insert_message_with_swipe(&ctx, last);
    }

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state_for_test();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Idle),
        "Should finish with Idle status, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_main_narration_happy_path() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_main_id = save_pre_main(&ctx);
    let _final_id = add_narration_and_save(&ctx, "Narration text");

    let service = make_service();
    let state = ctx.load_state_for_test();
    let _ = retry_main_narration(&service, &ctx, state, "test input".to_string());
}

#[test]
fn test_retry_main_storage_error_on_pre_main() {
    let state = make_test_state();
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    let storage = Arc::new(storage);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = storage.save_snapshot(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = storage.insert_message(&msg) {
            if let Some(swipe) = msg.swipes.first() {
                let _ = storage.insert_swipe(id, swipe, 0);
            }
        }
    }
    let base_ctx = GameServiceContext {
        storage: Arc::clone(&storage),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(Storage::new_in_memory()),
    };

    let _input_id = add_input_and_save(&base_ctx, "test input");
    let _pre_main_id = save_pre_main(&base_ctx);
    let _final_id = add_narration_and_save(&base_ctx, "Narration text");

    handle.set(
        Operation::LoadSnapshotById,
        TestOverride::internal("simulated load_by_id failure"),
    );

    let service = make_service();
    retry_last_response_impl(&service, base_ctx.clone());

    handle.clear(Operation::LoadSnapshotById);

    let state = base_ctx.load_state_for_test();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status on storage failure"
    );
}

#[test]
fn test_save_retry_error() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    super::retry::save_retry_error(&ctx, "test error");

    let state = ctx.load_state_for_test();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg == "test error"
        ),
        "Should save error status with exact message"
    );
}

#[test]
fn test_save_retry_error_persist_fails() {
    let state = make_test_state();
    let base_ctx = make_test_context_with_sqlite(state).unwrap();

    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated save failure"),
    );

    let ctx = GameServiceContext {
        storage: failing,
        ..base_ctx.clone()
    };

    // Should not panic when save_state fails inside save_retry_error
    super::retry::save_retry_error(&ctx, "persist failure");
}

struct EmptyTriggerBackend;

impl crate::narrative::llm::LlmBackend for EmptyTriggerBackend {
    fn model(&self) -> &str {
        "mock"
    }

    fn narrate_continuation(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(LlmCallResult {
            text: format!("[Trigger: {trigger_prompt}]"),
            system_prompt: String::new(),
            user_prompt: String::new(),
            raw_request_json: String::new(),
            raw_response_json: String::new(),
            backend_name: "EmptyTrigger".to_string(),
            model_name: "mock".to_string(),
            agent_name: agent_name.to_string(),
        })
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
            system_prompt: String::new(),
            user_prompt: String::new(),
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
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let llm = Arc::new(EmptyTriggerBackend);
    let service = DefaultGameService::with_mock_quantifier(
        llm,
        Arc::new(crate::narrative::llm::MockBackend::default()),
    );

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_main_id = save_pre_main(&ctx);

    let mut pre_event_state = ctx.load_state_for_test();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&pre_event_state);
    let pre_event_id = ctx.storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&ctx, last);
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
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.storage.save_snapshot(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        insert_message_with_swipe(&ctx, last);
    }
    retry_last_response_impl(&service, ctx.clone());
    let state = ctx.load_state_for_test();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Error(ref msg) if msg.contains("empty response")),
        "Should set error status when continuation text is empty, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_appends_swipe_to_same_message() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_main_id = save_pre_main(&ctx);
    let _narration_id = add_narration_and_save(&ctx, "Narration text");

    // Add an extra swipe to the narration
    let msgs = ctx.load_messages().unwrap();
    let narration_msg = msgs
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .unwrap();
    let original_id = narration_msg.id;
    let extra_swipe = crate::model::message::Swipe {
        text: "Alt narration".to_string(),
        snapshot_id: Some(_pre_main_id),
        location_header: None,
        event_header: None,
    };
    ctx.storage
        .insert_swipe(narration_msg.id, &extra_swipe, 1)
        .unwrap();

    retry_last_response_impl(&service, ctx.clone());

    // After retry, the SAME message should have gained a new swipe
    let msgs = ctx.load_messages().unwrap();
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
fn test_retrigger_event_impl_cancels_cleanly() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_main_id = save_pre_main(&ctx);

    // Set up pre-event snapshot with trigger context
    let mut pre_event_state = ctx.load_state_for_test();
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), None, MessageType::Narration);
    let snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&pre_event_state);
    let pre_event_id = ctx.storage.save_snapshot(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&ctx, last);
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
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.storage.save_snapshot(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        insert_message_with_swipe(&ctx, last);
    }

    ctx.cancel_token.cancel();

    // Directly call retrigger_event_impl to hit the Cancelled branch
    crate::application::action_pipeline::retrigger_event_impl(&service, &ctx);

    let state = ctx.load_state_for_test();
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
