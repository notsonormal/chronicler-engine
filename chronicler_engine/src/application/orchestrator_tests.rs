//! Unit tests for action-pipeline orchestration and collaborator behaviour.
use std::sync::Arc;
use std::sync::Barrier;

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::application::agents::registry::AgentRegistry;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::Agent;
use crate::application::errors::ApplicationError;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::LlmProvider;
use crate::adapters::driving::http::AppState;
use crate::domain::model::agent::{AgentContext, AgentResult, BackendSelector, ExecutionPhase};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;
use crate::test_support::fixtures::{TestMap, TestWorld};
use crate::test_support::make_test_recorder;
use crate::test_support::TestAppBuilder;
use crate::test_support::TestDataBuilder;

fn minimal_state() -> GameState {
    GameState::new("start")
}

fn minimal_app() -> AppState {
    AppState::from_wired(
        crate::test_support::make_test_app(minimal_state()).expect("minimal_app should build"),
    )
}

fn minimal_app_no_game() -> AppState {
    let state = minimal_state();
    let world_arc = Arc::new(TestWorld::minimal());
    let map_arc = Arc::new(TestMap::single_room("start"));
    let storage = Arc::new(Storage::new_in_memory());
    storage.seed_world(&world_arc, &map_arc).unwrap();
    let _ = storage.save_snapshot(&GameStateSnapshot::from_game_state(&state));
    let mock: Arc<dyn LlmProvider> = Arc::new(MockBackend::default());
    let narrator_recorder = make_test_recorder(Arc::clone(&mock));
    let registry = AgentRegistry::default();
    let persistence_gate = crate::test_support::build_test_persistence_gate(Arc::clone(&storage));
    let settings = Arc::new(std::sync::RwLock::new(
        crate::domain::model::settings::AppSettings::default(),
    ));
    let pipeline = crate::application::pipeline::pipeline::ActionPipeline::with_backends(
        CancellationToken::new(),
        narrator_recorder,
        registry,
        persistence_gate,
        settings,
    );
    let wired = crate::test_support::build_test_wired_app(
        Arc::clone(&storage),
        Arc::new(Storage::new_in_memory()),
        pipeline,
    )
    .expect("build_test_wired_app: build_app_graph_for_tests should succeed");
    AppState::from_wired(wired)
}

#[test]
fn test_get_generating_status_returns_current_state() {
    let app = minimal_app();
    let (status, _phase) = app.game_view_query.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}

#[test]
fn test_get_current_game_name_unknown_when_no_game() {
    let app = minimal_app_no_game();
    let name = app.game_view_query.get_current_game_name().unwrap();
    assert_eq!(name, "Unknown");
}

#[test]
fn test_list_latest_llm_messages_empty() {
    let app = minimal_app_no_game();
    let messages = app.game_view_query.list_latest_llm_messages(10).unwrap();
    assert!(messages.is_empty());
}

#[test]
fn test_get_story_log_entries_empty() {
    let app = minimal_app_no_game();
    let (entries, has_trigger) = app.game_view_query.get_story_log_entries().unwrap();
    assert!(entries.is_empty());
    assert!(!has_trigger);
}

#[test]
fn test_get_current_room_view_succeeds_with_valid_state() {
    let app = minimal_app();
    let result = app.game_view_query.get_current_room_view();
    assert!(result.is_ok());
    let (room_name, _image_path) = result.unwrap();
    assert_eq!(room_name, "Room start");
}

#[test]
fn test_get_current_room_view_returns_typed_error_when_game_missing() {
    let app = minimal_app_no_game();
    let err = app.game_view_query.get_current_room_view().unwrap_err();
    assert!(
        matches!(
            &err,
            ApplicationError::Engine(EngineError::GameNotFound(id)) if *id == 0
        ),
        "expected GameNotFound(0), got: {err:?}"
    );
}

#[test]
fn test_get_npc_headshots_returns_typed_error_when_game_missing() {
    let app = minimal_app_no_game();
    let err = app.game_view_query.get_npc_headshots(true).unwrap_err();
    assert!(
        matches!(
            &err,
            ApplicationError::Engine(EngineError::GameNotFound(id)) if *id == 0
        ),
        "expected GameNotFound(0), got: {err:?}"
    );
}

#[test]
fn test_get_npc_headshots_scene_only_empty() {
    let app = minimal_app();
    let headshots = app.game_view_query.get_npc_headshots(true).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_npc_headshots_all_empty() {
    let app = minimal_app();
    let headshots = app.game_view_query.get_npc_headshots(false).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_debug_state_populates_fields() {
    let app = minimal_app_no_game();
    let debug = app.game_view_query.get_debug_state().unwrap();
    assert_eq!(debug.narration_history_length, 0);
    assert!(debug.dynamic_rooms.is_empty());
    assert_eq!(debug.dynamic_room_count, 0);
    assert!(debug.last_error.is_none());
}

#[test]
fn test_active_quantifier_prompt_does_not_panic() {
    let app = minimal_app();
    let prompt = app.game_view_query.active_quantifier_prompt();
    let _ = prompt.len();
}

#[test]
fn test_reset_generating_status_sets_idle() {
    let app = minimal_app_no_game();
    let game_id = app.game_catalogue.current_game_id();
    let _ = app
        .generation_gate
        .release_generation_slot_for_game(game_id);
    let result = app.pipeline.reset_persisted_status();
    assert!(result.is_ok());
    let (status, _) = app.game_view_query.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}

struct SyncQuantifierAgent {
    inner: QuantifierAgent,
    entered: Arc<Barrier>,
    release: Arc<Barrier>,
}

impl std::fmt::Debug for SyncQuantifierAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncQuantifierAgent")
            .finish_non_exhaustive()
    }
}

impl Agent for SyncQuantifierAgent {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn phase(&self) -> ExecutionPhase {
        self.inner.phase()
    }
    fn backend_selector(&self) -> BackendSelector {
        self.inner.backend_selector()
    }
    fn execute(&self, ctx: &AgentContext) -> crate::error::Result<AgentResult> {
        self.entered.wait();
        self.release.wait();
        self.inner.execute(ctx)
    }
}

fn make_test_service(
    narrator_recorder: Arc<LlmCallRecorder>,
    quantifier_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
) -> crate::application::pipeline::pipeline::ActionPipeline {
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let registry = AgentRegistry::with_agent(Box::new(agent));
    let storage = Arc::new(Storage::new_in_memory());
    let persistence_gate = crate::test_support::build_test_persistence_gate(Arc::clone(&storage));
    let settings = Arc::new(std::sync::RwLock::new(
        crate::domain::model::settings::AppSettings::default(),
    ));
    crate::application::pipeline::pipeline::ActionPipeline::with_backends(
        CancellationToken::new(),
        narrator_recorder,
        registry,
        persistence_gate,
        settings,
    )
}

#[test]
fn test_boot_heal_resets_stale_generating_status() {
    let world_arc = Arc::new(TestWorld::minimal());
    let map_arc = Arc::new(TestMap::single_room("start"));
    let storage = Arc::new(Storage::new_in_memory());
    storage.seed_world(&world_arc, &map_arc).unwrap();

    let mut state = minimal_state();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    let _ = storage.save_snapshot(&GameStateSnapshot::from_game_state(&state));

    let mock: Arc<dyn LlmProvider> = Arc::new(MockBackend::default());
    let narrator_recorder = make_test_recorder(Arc::clone(&mock));
    let agent_registry = AgentRegistry::default();
    let persistence_gate = crate::test_support::build_test_persistence_gate(Arc::clone(&storage));
    let settings = Arc::new(std::sync::RwLock::new(
        crate::domain::model::settings::AppSettings::default(),
    ));
    let pipeline = crate::application::pipeline::pipeline::ActionPipeline::with_backends(
        CancellationToken::new(),
        narrator_recorder,
        agent_registry,
        Arc::clone(&persistence_gate),
        Arc::clone(&settings),
    );
    let wired = crate::test_support::build_test_wired_app(
        storage,
        Arc::new(Storage::new_in_memory()),
        pipeline,
    )
    .expect("build_test_wired_app should succeed");
    let app = AppState::from_wired(wired);

    let (status, phase) = app.game_view_query.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
    assert_eq!(phase, GenerationPhase::default());
}

fn make_test_service_with_agent(
    narrator_recorder: Arc<LlmCallRecorder>,
    agent: Box<dyn crate::application::agents::Agent>,
) -> crate::application::pipeline::pipeline::ActionPipeline {
    let registry = AgentRegistry::with_agent(agent);
    let storage = Arc::new(Storage::new_in_memory());
    let persistence_gate = crate::test_support::build_test_persistence_gate(Arc::clone(&storage));
    let settings = Arc::new(std::sync::RwLock::new(
        crate::domain::model::settings::AppSettings::default(),
    ));
    crate::application::pipeline::pipeline::ActionPipeline::with_backends(
        CancellationToken::new(),
        narrator_recorder,
        registry,
        persistence_gate,
        settings,
    )
}

#[test]
fn test_execute_action_completes_and_persists_state() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.persistence_gate.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle
    );
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default()
    );
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(has_narration, "execute_action should persist narration");
}

#[test]
fn test_execute_action_clears_last_trigger() {
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = TestAppBuilder::default_test()
        .last_trigger(crate::test_support::TestStoredTriggerContext::named(
            "Old Trigger",
            "npc1",
        ))
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.persistence_gate.load_or_fresh();
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared before pipeline runs"
    );
}

#[test]
fn test_execute_action_handles_narration_error() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.persistence_gate.load_or_fresh();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "State should reflect error status after failed narration"
    );
}

#[test]
fn test_execute_action_handles_cancellation() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.shutdown_token.cancel();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.persistence_gate.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancellation should reset status to Idle"
    );
}

#[test]
fn test_execute_action_preserves_existing_input_log() {
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = TestAppBuilder::default_test()
        .log("examine room", Some("Player"), MessageType::Input)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("examine room".to_string());
    let final_state = app.persistence_gate.load_or_fresh();
    let entries: Vec<_> = final_state.narrative.history().into_iter().collect();
    let input_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Input);
    let narration_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Narration);
    assert!(input_idx.is_some(), "Existing input should be preserved");
    assert!(narration_idx.is_some(), "Narration should be added");
    assert!(
        input_idx.unwrap() < narration_idx.unwrap(),
        "Input should appear before narration"
    );
}

#[test]
fn test_phase_transitions_to_quantifying_during_post_generation() {
    use std::thread;

    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let base_agent =
        QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider.clone());
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let sync_agent = Box::new(SyncQuantifierAgent {
        inner: base_agent,
        entered: entered.clone(),
        release: release.clone(),
    });
    let service = make_test_service_with_agent(narrator_recorder, sync_agent);
    let app = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service();
    let app_for_thread = app.clone();

    let handle = thread::spawn(move || {
        app_for_thread.pipeline.execute_action("look".to_string());
    });

    entered.wait();
    let mid_state = app.persistence_gate.load_or_fresh();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying during post-generation, not stuck on Narrating"
    );

    release.wait();
    handle.join().expect("Action thread should complete");
    let final_state = app.persistence_gate.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should reset to default after completion"
    );
}

#[test]
fn test_narration_saved_before_quantifying_phase() {
    use std::thread;

    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let base_agent =
        QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider.clone());
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let sync_agent = Box::new(SyncQuantifierAgent {
        inner: base_agent,
        entered: entered.clone(),
        release: release.clone(),
    });
    let service = make_test_service_with_agent(narrator_recorder, sync_agent);
    let app = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service();
    let app_for_thread = app.clone();

    let handle = thread::spawn(move || {
        app_for_thread.pipeline.execute_action("test".to_string());
    });

    entered.wait();
    let messages = app.persistence_gate.load_messages().unwrap();
    let narration_count = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 1,
        "Narration should be saved before quantifier completes"
    );
    let mid_state = app.persistence_gate.load_or_fresh();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying while quantifier runs"
    );

    release.wait();
    handle.join().expect("Action thread should complete");
}
