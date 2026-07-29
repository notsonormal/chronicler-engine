use std::sync::Arc;
use std::sync::Barrier;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tokio::time::sleep;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::application::agents::registry::AgentRegistry;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::Agent;
use crate::application::application_service::DefaultApplicationService;
use crate::application::errors::ApplicationError;
use crate::application::game_service::GameService;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::agent::{AgentContext, AgentResult, BackendSelector, ExecutionPhase};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;
use crate::test_support::fixtures::{TestMap, TestWorld};
use crate::test_support::make_test_app;
use crate::test_support::make_test_recorder;
use crate::test_support::TestAppBuilder;
use crate::test_support::TestDataBuilder;

fn minimal_state() -> GameState {
    GameState::new("start")
}

fn minimal_app() -> Arc<DefaultApplicationService> {
    make_test_app(minimal_state())
        .map(|wired| Arc::clone(&wired.application_service))
        .expect("minimal_app: make_test_app should succeed")
}

fn minimal_app_no_game() -> Arc<DefaultApplicationService> {
    let state = minimal_state();
    let world_arc = Arc::new(TestWorld::minimal());
    let map_arc = Arc::new(TestMap::single_room("start"));
    let storage = Arc::new(Storage::new_in_memory());
    storage.seed_world(&world_arc, &map_arc).unwrap();
    let _ = storage.save_snapshot(&GameStateSnapshot::from_game_state(&state));
    let mock: Arc<dyn LlmProvider> = Arc::new(MockBackend::default());
    let backend = GameService::with_backends(make_test_recorder(mock), AgentRegistry::default());
    crate::test_support::build_test_service(
        storage,
        Arc::new(Storage::new_in_memory()),
        Arc::new(backend),
    )
    .expect("build_test_service: build_app_graph_for_tests should succeed")
}

#[test]
fn test_get_generating_status_returns_current_state() {
    let app = minimal_app();
    let (status, _phase) = app.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}

#[test]
fn test_get_current_game_name_unknown_when_no_game() {
    let app = minimal_app_no_game();
    let name = app.get_current_game_name().unwrap();
    assert_eq!(name, "Unknown"); // No default game anymore
}

#[test]
fn test_list_latest_llm_messages_empty() {
    let app = minimal_app_no_game();
    let messages = app.list_latest_llm_messages(10).unwrap();
    assert!(messages.is_empty());
}

#[test]
fn test_get_story_log_entries_empty() {
    let app = minimal_app_no_game();
    let (entries, has_trigger) = app.get_story_log_entries().unwrap();
    assert!(entries.is_empty());
    assert!(!has_trigger);
}

#[test]
fn test_get_current_room_view_succeeds_with_valid_state() {
    let app = minimal_app();
    let result = app.get_current_room_view();
    assert!(result.is_ok());
    let (room_name, _image_path) = result.unwrap();
    assert_eq!(room_name, "Room start");
}

#[test]
fn test_get_current_room_view_returns_typed_error_when_game_missing() {
    let app = minimal_app_no_game();
    let err = app.get_current_room_view().unwrap_err();
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
    let err = app.get_npc_headshots(true).unwrap_err();
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
    let headshots = app.get_npc_headshots(true).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_npc_headshots_all_empty() {
    let app = minimal_app();
    let headshots = app.get_npc_headshots(false).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_debug_state_populates_fields() {
    let app = minimal_app_no_game();
    let debug = app.get_debug_state().unwrap();
    assert_eq!(debug.narration_history_length, 0);
    assert!(debug.dynamic_rooms.is_empty());
    assert_eq!(debug.dynamic_room_count, 0);
    assert!(debug.last_error.is_none());
}

#[test]
fn test_active_quantifier_prompt_does_not_panic() {
    let app = minimal_app();
    let prompt = app.active_quantifier_prompt();
    let _ = prompt.len();
}

#[test]
fn test_reset_generating_status_sets_idle() {
    let app = minimal_app_no_game();
    let result = app.reset_generating_status();
    assert!(result.is_ok());
    let (status, _) = app.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}

fn cached_flag(app: &DefaultApplicationService) -> bool {
    app.is_generating_now()
}

fn persisted_flag(app: &DefaultApplicationService) -> bool {
    app.storage()
        .load_latest_snapshot()
        .ok()
        .flatten()
        .map(|_snap| {
            app.persistence_gate
                .load_or_fresh()
                .narrative
                .input_buffer
                .status
                .is_generating()
        })
        .unwrap_or(false)
}

fn invariant_holds(app: &DefaultApplicationService) -> bool {
    cached_flag(app) == persisted_flag(app)
}

async fn wait_until_idle(app: &DefaultApplicationService, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let cached = cached_flag(app);
        let persisted = persisted_flag(app);

        if !cached && persisted {
            panic!(
                "invariant violation during wait_until_idle: cached=false persisted=true. \
                 expected (cached=true, persisted=Idle) as the only allowed transient."
            );
        }

        if cached && !persisted {
            sleep(Duration::from_millis(50)).await;
            continue;
        }

        if !cached && !persisted {
            assert!(invariant_holds(app), "both idle but invariant violated");
            return true;
        }

        sleep(Duration::from_millis(50)).await;
    }
    false
}

#[test]
fn test_is_generating_invariant_helper_detects_divergence() {
    let app = TestAppBuilder::default_test().build_service();

    assert!(
        invariant_holds(&app),
        "Invariant should hold initially: cached={} persisted={}",
        cached_flag(&app),
        persisted_flag(&app)
    );

    app.generation_gate
        .is_generating()
        .store(true, Ordering::SeqCst);

    assert!(
        !invariant_holds(&app),
        "Invariant helper must detect divergence: cached=true persisted=false"
    );
    assert!(cached_flag(&app), "AtomicBool forced to true");
    assert!(
        !persisted_flag(&app),
        "Persisted status should still report Idle"
    );
}

#[tokio::test(flavor = "current_thread")]
#[should_panic(expected = "invariant violation during wait_until_idle")]
async fn test_wait_until_idle_fails_fast_on_cached_false_persisted_generating() {
    let app = std::sync::Arc::new(TestAppBuilder::default_test().build_service());

    let mut gs = app.persistence_gate.load_or_fresh();
    gs.narrative.input_buffer.status = GenerationStatus::Generating;
    let snapshot_id = app
        .persistence_gate
        .save_state(&gs)
        .expect("save_state should persist Generating");
    app.generation_gate
        .is_generating()
        .store(false, Ordering::SeqCst);
    let _ = snapshot_id;

    let _ = wait_until_idle(&app, Duration::from_secs(2)).await;
}

#[tokio::test]
async fn test_projection_invariant_under_interleaved_release() {
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use crate::adapters::driven::llm::providers::MockBackend;
    use crate::application::agents::registry::AgentRegistry;
    use crate::application::errors::ProcessActionResult;
    use crate::application::game_service::GameService;
    use crate::test_support::make_test_recorder;

    let mock_backend_raw = std::sync::Arc::new(
        MockBackend::default()
            .with_delay(300)
            .with_narrations(vec!["GEN_A_OUTPUT".to_string(), "GEN_B_OUTPUT".to_string()]),
    );
    let game_service = {
        let recorder = make_test_recorder(mock_backend_raw.clone());
        std::sync::Arc::new(GameService::with_backends(
            recorder,
            AgentRegistry::default(),
        ))
    };
    let app = TestAppBuilder::default_test()
        .game_service(game_service)
        .build_service();

    let game_a_id = app.current_game_id();

    let result_a = app
        .process_action("look".to_string())
        .expect("process_action A should not error");
    assert!(
        matches!(result_a, ProcessActionResult::Started),
        "gen A claim should return Started, got {result_a:?}"
    );
    assert!(
        app.is_generating_now(),
        "projection must be true after A claim"
    );

    let narration_started =
        wait_for_condition(Duration::from_secs(5), Duration::from_millis(25), || {
            mock_backend_raw.narration_started.load(Ordering::SeqCst)
        })
        .await;
    assert!(
        narration_started,
        "gen A's narration call should start within timeout"
    );

    let game_b_id = app
        .create_game("test", "test_player")
        .expect("create_game(B) should succeed");
    assert_ne!(game_b_id, game_a_id, "reset must produce distinct game id");

    let result_b = app
        .process_action("go north".to_string())
        .expect("process_action B should not error");
    assert!(
        matches!(result_b, ProcessActionResult::Started),
        "gen B claim should return Started, got {result_b:?}"
    );

    let projection_held =
        wait_for_condition(Duration::from_secs(5), Duration::from_millis(25), || {
            app.is_generating_now()
        })
        .await;
    assert!(
        projection_held,
        "TOCTOU regression: projection must stay true after B claims \
         while A is still in flight. Pre-fix this could be false because \
         A's release_owned_slot stored false on the projection OUTSIDE the \
         registry write lock, racing B's claim and clobbering B's store(true)."
    );

    let b_completed =
        wait_for_condition(Duration::from_secs(10), Duration::from_millis(50), || {
            !app.is_generating_now()
        })
        .await;
    assert!(b_completed, "gen B's pipeline must complete within timeout");
    assert!(
        !app.is_generating_now(),
        "projection must be false after B completes"
    );

    app.cancel_token().cancel();
}

async fn wait_for_condition(
    timeout: Duration,
    poll: Duration,
    mut cond: impl FnMut() -> bool,
) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if cond() {
            return true;
        }
        tokio::time::sleep(poll).await;
    }
    false
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
) -> crate::application::game_service::GameService {
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let registry = AgentRegistry::with_agent(Box::new(agent));
    crate::application::game_service::GameService::with_backends(narrator_recorder, registry)
}

fn make_test_service_with_agent(
    narrator_recorder: Arc<LlmCallRecorder>,
    agent: Box<dyn crate::application::agents::Agent>,
) -> crate::application::game_service::GameService {
    let registry = AgentRegistry::with_agent(agent);
    crate::application::game_service::GameService::with_backends(narrator_recorder, registry)
}

#[test]
fn test_execute_action_completes_and_persists_state() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service))
        .build_service();
    app.execute_action("look".to_string());
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
        .game_service(Arc::new(service))
        .build_service();
    app.execute_action("look".to_string());
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
        .game_service(Arc::new(service))
        .build_service();
    app.execute_action("look".to_string());
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
        .game_service(Arc::new(service))
        .build_service();
    app.cancel_token().cancel();
    app.execute_action("look".to_string());
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
        .game_service(Arc::new(service))
        .build_service();
    app.execute_action("examine room".to_string());
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
    let service = Arc::new(make_test_service_with_agent(narrator_recorder, sync_agent));
    let app: Arc<DefaultApplicationService> = TestAppBuilder::default_test()
        .game_service(Arc::clone(&service))
        .build_service();
    let app_for_thread = Arc::clone(&app);

    let handle = thread::spawn(move || {
        app_for_thread.execute_action("look".to_string());
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
    let service = Arc::new(make_test_service_with_agent(narrator_recorder, sync_agent));
    let app: Arc<DefaultApplicationService> = TestAppBuilder::default_test()
        .game_service(Arc::clone(&service))
        .build_service();
    let app_for_thread = Arc::clone(&app);

    let handle = thread::spawn(move || {
        app_for_thread.execute_action("test".to_string());
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
