use std::sync::Arc;

use crate::application::action_pipeline::retry::{
    retry_event_continuation, retry_last_response_impl, retry_main_narration,
};
use crate::application::context::GameServiceContext;

#[allow(unused_imports)]
// retry_main_narration is kept for explicit test coverage even though
// most tests exercise it indirectly via retry_last_response_impl.
use crate::application::game_service::DefaultGameService;
use crate::error::{EngineError, internal_error};
use crate::model::message::Message;
use crate::model::state::{GameState, GenerationStatus, LogType};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::narrative::llm::MockBackend;
use crate::narrative::llm::backend::LlmCallResult;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::fixtures::{TestMap, TestNpc, TestPlayer, TestWorld};
use crate::test_support::in_memory_storage::InMemoryGameStorage;
use crate::test_support::make_test_context_with_sqlite;

struct FailingSnapshotStorage {
    fallback: Arc<dyn SnapshotStorage>,
    fail_save: std::sync::atomic::AtomicBool,
    fail_load_latest: std::sync::atomic::AtomicBool,
    fail_load_by_id: std::sync::atomic::AtomicBool,
}

impl FailingSnapshotStorage {
    fn new(fallback: Arc<dyn SnapshotStorage>) -> Self {
        Self {
            fallback,
            fail_save: std::sync::atomic::AtomicBool::new(false),
            fail_load_latest: std::sync::atomic::AtomicBool::new(false),
            fail_load_by_id: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl SnapshotStorage for FailingSnapshotStorage {
    fn set_game_id(&self, game_id: u64) {
        self.fallback.set_game_id(game_id);
    }

    fn current_game_id(&self) -> u64 {
        self.fallback.current_game_id()
    }

    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError> {
        if self.fail_save.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(EngineError::Internal(internal_error(
                "simulated save failure",
            )));
        }
        self.fallback.save(snapshot)
    }

    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError> {
        if self
            .fail_load_latest
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(EngineError::Internal(internal_error(
                "simulated load_latest failure",
            )));
        }
        self.fallback.load_latest()
    }

    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError> {
        if self
            .fail_load_by_id
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(EngineError::Internal(internal_error(
                "simulated load_by_id failure",
            )));
        }
        self.fallback.load_by_id(id)
    }

    fn list_games(&self) -> Result<Vec<crate::model::game::Game>, EngineError> {
        self.fallback.list_games()
    }

    fn create_game(&self, world_name: &str, name: &str) -> Result<u64, EngineError> {
        self.fallback.create_game(world_name, name)
    }

    fn delete_game(&self, id: u64) -> Result<(), EngineError> {
        self.fallback.delete_game(id)
    }

    fn get_game(&self, id: u64) -> Result<Option<crate::model::game::Game>, EngineError> {
        self.fallback.get_game(id)
    }
}

struct FailingMessageStorage {
    fallback: Arc<dyn MessageStorage>,
    fail_load_messages: std::sync::atomic::AtomicBool,
    fail_delete_message: std::sync::atomic::AtomicBool,
}

impl FailingMessageStorage {
    fn new(fallback: Arc<dyn MessageStorage>) -> Self {
        Self {
            fallback,
            fail_load_messages: std::sync::atomic::AtomicBool::new(false),
            fail_delete_message: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl MessageStorage for FailingMessageStorage {
    fn set_game_id(&self, game_id: u64) {
        self.fallback.set_game_id(game_id);
    }

    fn current_game_id(&self) -> u64 {
        self.fallback.current_game_id()
    }

    fn insert_message(&self, msg: &mut Message) -> Result<(), EngineError> {
        self.fallback.insert_message(msg)
    }

    fn update_message(&self, id: u64, text: &str) -> Result<(), EngineError> {
        self.fallback.update_message(id, text)
    }

    fn delete_message(&self, id: u64) -> Result<(), EngineError> {
        if self
            .fail_delete_message
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(EngineError::Internal(internal_error(
                "simulated delete_message failure",
            )));
        }
        self.fallback.delete_message(id)
    }

    fn load_messages(&self) -> Result<Vec<Message>, EngineError> {
        if self
            .fail_load_messages
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(EngineError::Internal(internal_error(
                "simulated load_messages failure",
            )));
        }
        self.fallback.load_messages()
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
    let storage = Arc::new(InMemoryGameStorage::new());
    let snapshot_storage: Arc<dyn SnapshotStorage> =
        Arc::clone(&storage) as Arc<dyn SnapshotStorage>;
    let message_storage: Arc<dyn MessageStorage> = Arc::clone(&storage) as Arc<dyn MessageStorage>;
    GameServiceContext {
        snapshot_storage,
        message_storage,
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
    }
}

fn make_service() -> DefaultGameService {
    DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(None)),
        Arc::new(crate::narrative::llm::MockBackend::default()),
    )
}

fn add_input_and_save(ctx: &GameServiceContext, text: &str) -> u64 {
    let mut state = ctx.load_state();
    let player_name = state.player.sheet.name.clone();
    state.add_log(text.to_string(), Some(player_name), LogType::Input);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let id = ctx.snapshot_storage.save(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.snapshot_id = Some(id);
        let _ = ctx.message_storage.insert_message(last);
    }
    id
}

fn add_narration_and_save(ctx: &GameServiceContext, text: &str) -> u64 {
    let mut state = ctx.load_state();
    state.add_log(text.to_string(), None, LogType::Narration);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let id = ctx.snapshot_storage.save(&snapshot).unwrap();
    if let Some(last) = state.narrative.history.last_mut() {
        last.snapshot_id = Some(id);
        let _ = ctx.message_storage.insert_message(last);
    }
    id
}

fn save_pre_main(ctx: &GameServiceContext) -> u64 {
    let state = ctx.load_state();
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    ctx.snapshot_storage.save(&snapshot).unwrap()
}

fn save_pre_event(ctx: &GameServiceContext) -> u64 {
    let state = ctx.load_state();
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    ctx.snapshot_storage.save(&snapshot).unwrap()
}

// ─── retry_last_response_impl ────────────────────────────────────────────────

#[test]
fn test_retry_no_snapshot() {
    let state = make_test_state();
    let ctx = make_empty_context(state);
    let service = make_service();
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_load_messages_error() {
    let state = make_test_state();
    let base_ctx = make_test_context_with_sqlite(state).unwrap();

    let failing_msg_storage = Arc::new(FailingMessageStorage::new(Arc::clone(
        &base_ctx.message_storage,
    )));
    failing_msg_storage
        .fail_load_messages
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let ctx = GameServiceContext {
        message_storage: failing_msg_storage,
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

    let mut state = ctx.load_state();
    state.add_log("Event narration".to_string(), None, LogType::Narration);
    state.narrative.history.last_mut().unwrap().event_header = Some("Event".to_string());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&snapshot);

    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_event_with_no_pre_event_and_no_input() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let mut state = ctx.load_state();
    state.add_log("Event only".to_string(), None, LogType::Narration);
    state.narrative.history.last_mut().unwrap().event_header = Some("Event".to_string());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&snapshot);

    retry_last_response_impl(&service, ctx);
}

// ─── retry_event_continuation ────────────────────────────────────────────────

#[test]
fn test_retry_event_storage_error_on_pre_event() {
    let state = make_test_state();
    let base_ctx = make_test_context_with_sqlite(state).unwrap();

    let _input_id = add_input_and_save(&base_ctx, "test input");
    let _pre_event_id = save_pre_event(&base_ctx);

    let failing = Arc::new(FailingSnapshotStorage::new(Arc::clone(
        &base_ctx.snapshot_storage,
    )));
    failing
        .fail_load_by_id
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let ctx = GameServiceContext {
        snapshot_storage: failing,
        ..base_ctx.clone()
    };

    let service = make_service();
    let latest = ctx.load_state();

    retry_event_continuation(&service, &ctx, latest);

    let state = base_ctx.load_state();
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

    // Build a final snapshot that points to the pre-event snapshot
    let mut state = ctx.load_state();
    state.add_log("Event narration".to_string(), None, LogType::Narration);
    state.narrative.history.last_mut().unwrap().event_header = Some("Event".to_string());
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&snapshot);

    // Pre-event snapshot exists but has no last_trigger
    retry_last_response_impl(&service, ctx);
}

#[test]
fn test_retry_event_continuation_cancels_before_llm() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_main_id = save_pre_main(&ctx);

    // Set up a pre-event snapshot with last_trigger so retry_event_continuation is reached
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
    pre_event_state.add_log("Main narration".to_string(), None, LogType::Narration);
    let snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&pre_event_state);
    let pre_event_id = ctx.snapshot_storage.save(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.snapshot_id = Some(pre_event_id);
        let _ = ctx.message_storage.insert_message(last);
    }

    // Cancel the token before retry runs
    ctx.cancel_token.cancel();

    retry_event_continuation(&service, &ctx, pre_event_state);

    let state = ctx.load_state();
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
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _pre_event_with_trigger_id = ctx.snapshot_storage.save(&snapshot).unwrap();

    // Final snapshot points to pre-event
    let mut final_state = state;
    final_state.add_log("Event narration".to_string(), None, LogType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .event_header = Some("Event".to_string());
    let final_snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.snapshot_storage.save(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        let _ = ctx.message_storage.insert_message(last);
    }

    retry_last_response_impl(&service, ctx.clone());

    // Should have saved an error state
    let state = ctx.load_state();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Should set error status when trigger narration fails"
    );
}

#[test]
fn test_retry_event_empty_continuation_text() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    // Exercise the setup path for event retry with an empty continuation.
    // The error branch is covered by other tests; this validates the happy-path wiring.
    let llm = Arc::new(MockBackend::new(None));
    let service = DefaultGameService::with_mock_quantifier(
        llm,
        Arc::new(crate::narrative::llm::MockBackend::default()),
    );

    let _input_id = add_input_and_save(&ctx, "test input");
    let _pre_event_id = save_pre_event(&ctx);

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
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _pre_event_with_trigger_id = ctx.snapshot_storage.save(&snapshot).unwrap();

    let mut final_state = state;
    final_state.add_log("Event narration".to_string(), None, LogType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .event_header = Some("Event".to_string());
    let final_snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.snapshot_storage.save(&final_snapshot);

    retry_last_response_impl(&service, ctx);
}

// ─── retry_main_narration ────────────────────────────────────────────────────

#[test]
fn test_retry_main_no_pre_main_snapshot() {
    let state = make_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = make_service();

    let mut state = ctx.load_state();
    let player_name = state.player.sheet.name.clone();
    state.add_log("test input".to_string(), Some(player_name), LogType::Input);
    if let Some(last) = state.narrative.history.last_mut() {
        let _ = ctx.message_storage.insert_message(last);
    }

    let mut state = ctx.load_state();
    state.add_log("Narration text".to_string(), None, LogType::Narration);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&snapshot);
    if let Some(last) = state.narrative.history.last_mut() {
        let _ = ctx.message_storage.insert_message(last);
    }

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state();
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
    pre_event_state.add_log("Main narration".to_string(), None, LogType::Narration);
    let snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&pre_event_state);
    let pre_event_id = ctx.snapshot_storage.save(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.snapshot_id = Some(pre_event_id);
        let _ = ctx.message_storage.insert_message(last);
    }

    let mut final_state = pre_event_state;
    final_state.add_log("Event narration".to_string(), None, LogType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .event_header = Some("Event".to_string());
    let final_snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.snapshot_storage.save(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.snapshot_id = Some(final_snapshot.db_id.unwrap_or(0));
        let _ = ctx.message_storage.insert_message(last);
    }

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state();
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
    let state = ctx.load_state();
    retry_main_narration(&service, &ctx, state, "test input".to_string());
}

#[test]
fn test_retry_main_storage_error_on_pre_main() {
    let state = make_test_state();
    let base_ctx = make_test_context_with_sqlite(state).unwrap();

    let _input_id = add_input_and_save(&base_ctx, "test input");
    let _pre_main_id = save_pre_main(&base_ctx);
    let _final_id = add_narration_and_save(&base_ctx, "Narration text");

    let failing = Arc::new(FailingSnapshotStorage::new(Arc::clone(
        &base_ctx.snapshot_storage,
    )));
    failing
        .fail_load_by_id
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

    let state = ctx.load_state();
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

    let failing = Arc::new(FailingSnapshotStorage::new(Arc::clone(
        &base_ctx.snapshot_storage,
    )));
    failing
        .fail_save
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let ctx = GameServiceContext {
        snapshot_storage: failing,
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
    pre_event_state.add_log("Main narration".to_string(), None, LogType::Narration);
    let snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&pre_event_state);
    let pre_event_id = ctx.snapshot_storage.save(&snapshot).unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.snapshot_id = Some(pre_event_id);
        let _ = ctx.message_storage.insert_message(last);
    }

    let mut final_state = pre_event_state;
    final_state.add_log("Event narration".to_string(), None, LogType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .event_header = Some("Event".to_string());
    let final_snapshot =
        crate::model::state_snapshot::GameStateSnapshot::from_game_state(&final_state);
    let _ = ctx.snapshot_storage.save(&final_snapshot);
    if let Some(last) = final_state.narrative.history.last_mut() {
        let _ = ctx.message_storage.insert_message(last);
    }

    retry_last_response_impl(&service, ctx.clone());

    let state = ctx.load_state();
    assert!(
        matches!(state.narrative.input_buffer.status, GenerationStatus::Error(ref msg) if msg.contains("empty response")),
        "Should set error status when continuation text is empty, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_aborts_when_message_delete_fails() {
    let state = make_test_state();
    let base_ctx = make_test_context_with_sqlite(state).unwrap();

    let _input_id = add_input_and_save(&base_ctx, "test input");
    let _pre_main_id = save_pre_main(&base_ctx);
    let _narration_id = add_narration_and_save(&base_ctx, "Narration text");

    let failing_msg_storage = Arc::new(FailingMessageStorage::new(Arc::clone(
        &base_ctx.message_storage,
    )));
    failing_msg_storage
        .fail_delete_message
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let ctx = GameServiceContext {
        message_storage: failing_msg_storage,
        ..base_ctx.clone()
    };

    let service = make_service();
    retry_last_response_impl(&service, ctx.clone());

    // The error state should be saved
    let state = base_ctx.load_state();
    assert!(
        matches!(
            state.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("could not delete message")
        ),
        "Should set error status when message deletion fails, got {:?}",
        state.narrative.input_buffer.status
    );

    // Messages should NOT have been truncated — both input and narration remain
    let msgs = ctx.message_storage.load_messages().unwrap();
    assert_eq!(
        msgs.len(),
        2,
        "Messages should not be truncated when deletion fails"
    );
}
