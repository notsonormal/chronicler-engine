use std::sync::Arc;

use crate::engine::game_service::context::GameServiceContext;
use crate::engine::game_service::retry::{
    retry_event_continuation, retry_last_response_impl, retry_main_narration,
};

#[allow(unused_imports)]
// retry_main_narration is kept for explicit test coverage even though
// most tests exercise it indirectly via retry_last_response_impl.
use crate::engine::game_service::service::DefaultGameService;
use crate::error::{EngineError, internal_error};
use crate::model::checkpoint::Checkpoint;
use crate::model::state::{GameState, GenerationStatus, LogType};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::narrative::llm::MockBackend;
use crate::narrative::llm::backend::LlmCallResult;
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::fixtures::{TestMap, TestNpc, TestPlayer, TestWorld};
use crate::test_support::in_memory_storage::InMemorySnapshotStorage;
use crate::test_support::make_test_context_with_sqlite;

/// Snapshot storage that can be configured to fail specific operations.
struct FailingSnapshotStorage {
    fallback: Arc<dyn SnapshotStorage>,
    fail_load_latest: std::sync::atomic::AtomicBool,
    fail_load_by_turn: std::sync::atomic::AtomicBool,
}

impl FailingSnapshotStorage {
    fn new(fallback: Arc<dyn SnapshotStorage>) -> Self {
        Self {
            fallback,
            fail_load_latest: std::sync::atomic::AtomicBool::new(false),
            fail_load_by_turn: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl SnapshotStorage for FailingSnapshotStorage {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<(), EngineError> {
        self.fallback.save(snapshot)
    }

    fn load_latest(&self, turn_id: Option<&str>) -> Result<Option<GameStateSnapshot>, EngineError> {
        if self
            .fail_load_latest
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(EngineError::Internal(internal_error(
                "simulated load_latest failure",
            )));
        }
        self.fallback.load_latest(turn_id)
    }

    fn load_by_turn(
        &self,
        turn_id: &str,
        swipe_index: u32,
    ) -> Result<Option<GameStateSnapshot>, EngineError> {
        if self
            .fail_load_by_turn
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(EngineError::Internal(internal_error(
                "simulated load_by_turn failure",
            )));
        }
        self.fallback.load_by_turn(turn_id, swipe_index)
    }

    fn delete_turn_snapshots(&self, turn_id: &str) -> Result<(), EngineError> {
        self.fallback.delete_turn_snapshots(turn_id)
    }

    fn commit(&self, snapshot_id: &str) -> Result<(), EngineError> {
        self.fallback.commit(snapshot_id)
    }

    fn reset(&self) -> Result<(), EngineError> {
        self.fallback.reset()
    }

    fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), EngineError> {
        self.fallback.save_checkpoint(checkpoint)
    }

    fn load_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>, EngineError> {
        self.fallback.load_checkpoint(id)
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, EngineError> {
        self.fallback.list_checkpoints()
    }

    fn delete_checkpoint(&self, id: &str) -> Result<(), EngineError> {
        self.fallback.delete_checkpoint(id)
    }
}

fn make_test_state() -> GameState {
    let world = Arc::new(TestWorld::minimal());
    let map = Arc::new(TestMap::single_room("start"));
    let player = Arc::new(TestPlayer::standard());
    let npcs = vec![TestNpc::named("npc1", "Test NPC")];
    GameState::new(world, map, player, npcs, "start".to_string())
}

fn make_empty_context(state: GameState) -> GameServiceContext {
    let storage = Arc::new(InMemorySnapshotStorage::new()) as Arc<dyn SnapshotStorage>;
    GameServiceContext {
        snapshot_storage: storage,
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        action_lock: Arc::new(std::sync::Mutex::new(())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}

fn make_service() -> DefaultGameService {
    DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(None)),
        Arc::new(crate::narrative::agents::quantifier::MockQuantifierBackend::default()),
    )
}

fn add_input_and_save(ctx: &GameServiceContext, text: &str) {
    let mut state = ctx.load_state();
    let player_name = state.player.sheet.name.clone();
    state.add_log(text.to_string(), Some(player_name), LogType::Input);
    let turn_id = state
        .narrative
        .messages
        .last()
        .map(|m| m.turn_id.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state, turn_id, 0);
    let _ = ctx.snapshot_storage.save(&snapshot);
}

fn add_narration_and_save(ctx: &GameServiceContext, text: &str, turn_id: &str) {
    let mut state = ctx.load_state();
    state.add_log(text.to_string(), None, LogType::Narration);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        turn_id.to_string(),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);
}

fn save_pre_main(ctx: &GameServiceContext, turn_id: &str) {
    let state = ctx.load_state();
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        format!("pre-main:{turn_id}"),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);
}

fn save_pre_event(ctx: &GameServiceContext, turn_id: &str) {
    let state = ctx.load_state();
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        format!("pre-event:{turn_id}"),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);
}

// ─── retry_last_response_impl ────────────────────────────────────────────────

#[test]
fn test_retry_no_snapshot() {
    let state = make_test_state();
    let ctx = make_empty_context(state);
    let service = make_service();
    // No snapshots saved → should log error and return cleanly
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_no_input() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();
    // Snapshot exists but contains no input messages
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_event_with_no_pre_event_fallback_to_main() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    add_input_and_save(&ctx, "test input");
    let turn_id = ctx.load_state().narrative.current_turn_id.clone();

    // Save a narration with event header (marks it as event continuation)
    let mut state = ctx.load_state();
    state.add_log("Event narration".to_string(), None, LogType::Narration);
    state.narrative.messages.last_mut().unwrap().event_header = Some("Event".to_string());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        turn_id.clone(),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);

    // No pre-event snapshot → falls back to main narration retry
    // But no pre-main snapshot either → should error gracefully
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_event_with_no_pre_event_and_no_input() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    // Create a state with event header but no input text
    let mut state = ctx.load_state();
    state.add_log("Event only".to_string(), None, LogType::Narration);
    state.narrative.messages.last_mut().unwrap().event_header = Some("Event".to_string());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        "turn-1".to_string(),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);

    // No pre-event, no input → logs error and returns
    retry_last_response_impl(&service, ctx);
}

// ─── retry_event_continuation ────────────────────────────────────────────────

#[test]
fn test_retry_event_storage_error_on_pre_event() {
    let state = make_test_state();
    let base_ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&base_ctx, "test input");
    let turn_id = base_ctx.load_state().narrative.current_turn_id.clone();

    let failing = Arc::new(FailingSnapshotStorage::new(Arc::clone(
        &base_ctx.snapshot_storage,
    )));
    failing
        .fail_load_by_turn
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let ctx = GameServiceContext {
        snapshot_storage: failing,
        ..base_ctx.clone()
    };

    let service = make_service();
    let latest = ctx.load_state();

    retry_event_continuation(&service, &ctx, &turn_id, 0, &latest);

    let state = base_ctx.load_state();
    assert!(
        matches!(
            state.narrative.generation.status,
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

    add_input_and_save(&ctx, "test input");
    let turn_id = ctx.load_state().narrative.current_turn_id.clone();
    save_pre_event(&ctx, &turn_id);

    // Pre-event snapshot exists but has no last_trigger
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_event_trigger_narration_fails() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let llm = Arc::new(MockBackend::with_failing_trigger_narration());
    let service = DefaultGameService::with_mock_quantifier(
        llm,
        Arc::new(crate::narrative::agents::quantifier::MockQuantifierBackend::default()),
    );

    add_input_and_save(&ctx, "test input");
    let turn_id = ctx.load_state().narrative.current_turn_id.clone();
    save_pre_event(&ctx, &turn_id);

    // Set up trigger context in pre-event snapshot
    let mut state = ctx.load_state();
    state.narrative.last_trigger = Some(crate::model::state::StoredTriggerContext {
        npc_id: "npc1".to_string(),
        trigger_idx: 0,
        trigger_name: "Test".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Test prompt".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        format!("pre-event:{turn_id}"),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);

    retry_last_response_impl(&service, ctx.clone());

    // Should have saved an error state
    let state = ctx.load_state();
    assert!(
        matches!(
            state.narrative.generation.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status when trigger narration fails"
    );
}

#[test]
fn test_retry_event_empty_continuation_text() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    // MockBackend doesn't support empty trigger responses directly;
    // this test exercises the setup path (covered by other tests for the error branch).
    let llm = Arc::new(MockBackend::new(None));
    let service = DefaultGameService::with_mock_quantifier(
        llm,
        Arc::new(crate::narrative::agents::quantifier::MockQuantifierBackend::default()),
    );

    add_input_and_save(&ctx, "test input");
    let turn_id = ctx.load_state().narrative.current_turn_id.clone();
    save_pre_event(&ctx, &turn_id);

    let mut state = ctx.load_state();
    state.narrative.last_trigger = Some(crate::model::state::StoredTriggerContext {
        npc_id: "npc1".to_string(),
        trigger_idx: 0,
        trigger_name: "Test".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Test prompt".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        format!("pre-event:{turn_id}"),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);

    retry_last_response_impl(&service, ctx);
}

// ─── retry_main_narration ────────────────────────────────────────────────────

#[test]
fn test_retry_main_no_pre_main_snapshot() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    add_input_and_save(&ctx, "test input");
    let turn_id = ctx.load_state().narrative.current_turn_id.clone();

    // Save narration snapshot but NOT pre-main snapshot
    add_narration_and_save(&ctx, "Narration text", &turn_id);

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state();
    assert!(
        matches!(
            state.narrative.generation.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status when pre-main snapshot is missing"
    );
}

#[test]
fn test_retry_event_continuation_happy_path() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = make_service();

    // Set up the full snapshot chain for an event retry
    add_input_and_save(&ctx, "test input");
    let turn_id = ctx.load_state().narrative.current_turn_id.clone();
    save_pre_main(&ctx, &turn_id);

    let mut pre_event_state = ctx.load_state();
    pre_event_state.narrative.last_trigger = Some(crate::model::state::StoredTriggerContext {
        npc_id: "npc1".to_string(),
        trigger_idx: 0,
        trigger_name: "Test".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Test prompt".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &pre_event_state,
        format!("pre-event:{turn_id}"),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);

    // Final state with event narration
    let mut final_state = pre_event_state;
    final_state.add_log("Event narration".to_string(), None, LogType::Narration);
    final_state
        .narrative
        .messages
        .last_mut()
        .unwrap()
        .event_header = Some("Event".to_string());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &final_state,
        turn_id.clone(),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state();
    assert!(
        matches!(state.narrative.generation.status, GenerationStatus::Idle),
        "Should finish with Idle status, got {:?}",
        state.narrative.generation.status
    );
}

#[test]
fn test_retry_main_narration_happy_path() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "test input");
    let turn_id = ctx.load_state().narrative.current_turn_id.clone();
    add_narration_and_save(&ctx, "Narration text", &turn_id);
    save_pre_main(&ctx, &turn_id);

    let service = make_service();
    retry_main_narration(&service, &ctx, &turn_id, 0, "test input".to_string());
}

#[test]
fn test_retry_main_storage_error_on_pre_main() {
    let state = make_test_state();
    let base_ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&base_ctx, "test input");
    let turn_id = base_ctx.load_state().narrative.current_turn_id.clone();
    add_narration_and_save(&base_ctx, "Narration text", &turn_id);
    save_pre_main(&base_ctx, &turn_id);

    let failing = Arc::new(FailingSnapshotStorage::new(Arc::clone(
        &base_ctx.snapshot_storage,
    )));
    failing
        .fail_load_by_turn
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let ctx = GameServiceContext {
        snapshot_storage: failing,
        ..base_ctx.clone()
    };

    let service = make_service();
    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state();
    assert!(
        matches!(
            state.narrative.generation.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status on storage failure"
    );
}

#[test]
fn test_save_retry_error() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    // Directly test save_retry_error by calling it
    super::retry::save_retry_error(&ctx, "turn-1", 1, "test error");

    let state = ctx.load_state();
    assert!(
        matches!(
            state.narrative.generation.status,
            GenerationStatus::Error(ref msg) if msg == "test error"
        ),
        "Should save error status with exact message"
    );
}

/// Custom backend that returns empty text for trigger continuation.
struct EmptyTriggerBackend;

impl crate::narrative::llm::LlmBackend for EmptyTriggerBackend {
    fn generate_dialogue(
        &self,
        agent_name: &str,
        context: &crate::narrative::prompt::PromptContext,
        _npc: &crate::model::character::NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(LlmCallResult {
            text: format!("[MockGenerated] {}", context.user_message),
            system_prompt: String::new(),
            user_prompt: String::new(),
            raw_request_json: String::new(),
            raw_response_json: String::new(),
            backend_name: "EmptyTrigger".to_string(),
            model_name: "mock".to_string(),
            agent_name: agent_name.to_string(),
        })
    }

    fn narrate_action(
        &self,
        agent_name: &str,
        context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(LlmCallResult {
            text: format!("[MockNarration] {}", context.user_message),
            system_prompt: String::new(),
            user_prompt: String::new(),
            raw_request_json: String::new(),
            raw_response_json: String::new(),
            backend_name: "EmptyTrigger".to_string(),
            model_name: "mock".to_string(),
            agent_name: agent_name.to_string(),
        })
    }

    fn narrate_arrival(
        &self,
        agent_name: &str,
        context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(LlmCallResult {
            text: format!("[MockArrival] {}", context.room.name),
            system_prompt: String::new(),
            user_prompt: String::new(),
            raw_request_json: String::new(),
            raw_response_json: String::new(),
            backend_name: "EmptyTrigger".to_string(),
            model_name: "mock".to_string(),
            agent_name: agent_name.to_string(),
        })
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

    fn narrate_action_from_prompt(
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
        Arc::new(crate::narrative::agents::quantifier::MockQuantifierBackend::default()),
    );

    // Set up the snapshot chain: input → pre-main → pre-event → final (with event_header)
    add_input_and_save(&ctx, "test input");
    let turn_id = ctx.load_state().narrative.current_turn_id.clone();
    save_pre_main(&ctx, &turn_id);

    // Create pre-event state with trigger context
    let mut pre_event_state = ctx.load_state();
    pre_event_state.narrative.last_trigger = Some(crate::model::state::StoredTriggerContext {
        npc_id: "npc1".to_string(),
        trigger_idx: 0,
        trigger_name: "Test".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Test prompt".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &pre_event_state,
        format!("pre-event:{turn_id}"),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);

    // Create final state with event narration and event header
    let mut final_state = pre_event_state;
    final_state.add_log("Event narration".to_string(), None, LogType::Narration);
    final_state
        .narrative
        .messages
        .last_mut()
        .unwrap()
        .event_header = Some("Event".to_string());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &final_state,
        turn_id.clone(),
        0,
    );
    let _ = ctx.snapshot_storage.save(&snapshot);

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state();
    assert!(
        matches!(state.narrative.generation.status, GenerationStatus::Error(ref msg) if msg.contains("empty response")),
        "Should set error status when continuation text is empty, got {:?}",
        state.narrative.generation.status
    );
}
