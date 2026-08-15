//! Unit tests for the action entry path (process_action / execute_action).

use std::sync::Arc;
use std::sync::Barrier;

use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::registry::AgentRegistry;
use crate::application::agents::Agent;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::agent::{AgentContext, AgentResult, BackendSelector, ExecutionPhase};
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::test_support::{
    make_test_pipeline_app as make_test_app,
    make_test_pipeline_app_with_storage as make_test_app_with_storage,
    make_test_pipeline_with_backends, make_test_pipeline_with_mock_quantifier, make_test_recorder,
    TestAppBuilder, TestDataBuilder,
};

#[test]
fn test_execute_action_clears_last_trigger() {
    use crate::test_support::TestStoredTriggerContext;

    let app = make_test_app();

    let mut state = app.message_service.load_or_fresh();
    state.narrative.last_trigger = Some(TestStoredTriggerContext::for_npc(
        "npc_1",
        "Old",
        "The old trigger fires.",
    ));
    app.message_service
        .save_state(&state)
        .expect("save_state should succeed");

    app.pipeline.execute_action("look".to_string());

    let final_state = app.message_service.load_or_fresh();
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared by execute_action"
    );
}

#[test]
fn test_streaming_narration_saved_before_quantifier_complete() {
    use std::thread;
    use std::time::Duration;

    let quantifier_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(
            MockBackend::default()
                .with_prompt_responses(vec![r#"{"npcs_in_room": []}"#.to_string()])
                .with_delay(500),
        );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let service = crate::test_support::make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let (app, _storage) = TestAppBuilder::with_data(TestDataBuilder::default_test().build())
        .pipeline(service)
        .build_service_with_storage();

    let app_clone = app.clone();
    let message_service = Arc::clone(&app.message_service);
    let handle = thread::spawn(move || {
        app_clone.pipeline.execute_action("look around".to_string());
    });

    let start = std::time::Instant::now();
    let mut narration_found = false;
    while start.elapsed() < Duration::from_millis(400) {
        if message_service
            .load_messages()
            .map(|msgs| {
                msgs.iter()
                    .any(|m| m.message_type == MessageType::Narration)
            })
            .unwrap_or(false)
        {
            narration_found = true;
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        narration_found,
        "Narration should be saved before quantifier completes (quantifier takes 500ms)"
    );

    handle.join().expect("Action thread should complete");

    let final_state = app.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Should complete after quantifier finishes"
    );
    let final_narration_count = message_service
        .load_messages()
        .unwrap()
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        final_narration_count, 1,
        "Should have exactly 1 narration (no duplicates), found {final_narration_count}"
    );
}

#[test]
fn test_execute_action_completes_and_persists_state() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.message_service.load_or_fresh();
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
fn test_execute_action_handles_narration_error() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.message_service.load_or_fresh();
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
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.shutdown_token.cancel();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.message_service.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancellation should reset status to Idle"
    );
}

#[test]
fn test_execute_action_preserves_existing_input_log() {
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::default_test()
        .log("examine room", Some("Player"), MessageType::Input)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("examine room".to_string());
    let final_state = app.message_service.load_or_fresh();
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

#[test]
fn test_phase_transitions_to_quantifying_during_post_generation() {
    use std::thread;

    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let base_agent =
        QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider.clone());
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let sync_agent = Box::new(SyncQuantifierAgent {
        inner: base_agent,
        entered: entered.clone(),
        release: release.clone(),
    });
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        AgentRegistry::with_agent(sync_agent),
    );
    let app = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service();
    let app_for_thread = app.clone();

    let handle = thread::spawn(move || {
        app_for_thread.pipeline.execute_action("look".to_string());
    });

    entered.wait();
    let mid_state = app.message_service.load_or_fresh();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying during post-generation, not stuck on Narrating"
    );

    release.wait();
    handle.join().expect("Action thread should complete");
    let final_state = app.message_service.load_or_fresh();
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
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let base_agent =
        QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider.clone());
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let sync_agent = Box::new(SyncQuantifierAgent {
        inner: base_agent,
        entered: entered.clone(),
        release: release.clone(),
    });
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        AgentRegistry::with_agent(sync_agent),
    );
    let app = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service();
    let app_for_thread = app.clone();

    let handle = thread::spawn(move || {
        app_for_thread.pipeline.execute_action("test".to_string());
    });

    entered.wait();
    let messages = app.message_service.load_messages().unwrap();
    let narration_count = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 1,
        "Narration should be saved before quantifier completes"
    );
    let mid_state = app.message_service.load_or_fresh();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying while quantifier runs"
    );

    release.wait();
    handle.join().expect("Action thread should complete");
}

#[tokio::test]
async fn test_process_action_cancels_prior_generation_on_game_reset() {
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::{Duration, Instant};

    use crate::application::errors::ProcessActionResult;

    // Narration is delayed so the game-id can be flipped while gen A is mid-flight.
    let mock_backend = Arc::new(
        MockBackend::default()
            .with_delay(300)
            .with_narrations(vec!["GEN_A_OUTPUT".to_string(), "GEN_B_OUTPUT".to_string()]),
    );
    let narrator_recorder = make_test_recorder(Arc::clone(&mock_backend) as Arc<_>);
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        AgentRegistry::default(),
    );
    let (app, _storage) = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service_with_storage();

    let game1 = app.game_catalogue.current_game_id();

    let result_a = app
        .pipeline
        .process_action(&app.generation_gate, "look".to_string())
        .expect("gen A claim should not error");
    assert!(
        matches!(result_a, ProcessActionResult::Started),
        "gen A claim should return Started, got {result_a:?}"
    );

    let backend = Arc::clone(&mock_backend);
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(5) {
        if backend.narration_started.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        backend.narration_started.load(Ordering::SeqCst),
        "gen A narration should start within timeout"
    );

    // Reset to a new game mid-flight: gen A's next phase-boundary check reads game2 and cancels.
    let game2 = app
        .game_catalogue
        .create_game("test", "test_player")
        .expect("create_game should succeed");
    assert_ne!(game2, game1, "reset must produce a distinct game id");

    let gate = &app.generation_gate;
    let released = Instant::now();
    while released.elapsed() < Duration::from_secs(5) {
        if !gate.is_busy(game1) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !gate.is_busy(game1),
        "gen A slot must release after cancellation on game reset"
    );

    let state_after_a = app.message_service.load_or_fresh();
    let a_leaked = state_after_a
        .narrative
        .history
        .iter()
        .any(|e| e.text().contains("GEN_A_OUTPUT"));
    assert!(
        !a_leaked,
        "game 2 state must not contain cancelled gen A narration"
    );

    let result_b = app
        .pipeline
        .process_action(&app.generation_gate, "go north".to_string())
        .expect("gen B claim should not error");
    assert!(
        matches!(result_b, ProcessActionResult::Started),
        "gen B claim must succeed after gen A cancels and releases the slot, got {result_b:?}"
    );

    let completed = Instant::now();
    while completed.elapsed() < Duration::from_secs(5) {
        if !gate.is_busy(game2) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !gate.is_busy(game2),
        "gen B must complete and release its slot"
    );

    let state_after_b = app.message_service.load_or_fresh();
    let b_present = state_after_b
        .narrative
        .history
        .iter()
        .any(|e| e.text().contains("GEN_B_OUTPUT"));
    assert!(b_present, "game 2 state must contain gen B narration");

    app.shutdown_token.cancel();
}

#[test]
fn test_process_action_heals_stale_status_before_validation_error() {
    let (app, storage) = make_test_app_with_storage();

    // Create a game whose persona_key does not resolve; this is the validation
    // that will fail. The default world/persona are already seeded, but the new
    // game points at a non-existent persona.
    let bad_game_id = storage
        .create_game(
            "Test World",
            "test",
            "nonexistent",
            "Nonexistent",
            "Bad Game",
        )
        .expect("create_game should succeed");
    storage.set_game_id(bad_game_id);

    // Persist a stale Generating snapshot for the new game.
    let mut state = app.message_service.load_or_fresh();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    storage
        .save_snapshot(&GameStateSnapshot::from_game_state(&state))
        .expect("save stale snapshot should succeed");

    let result = app
        .pipeline
        .process_action(&app.generation_gate, "look".to_string());

    assert!(
        result.is_err(),
        "process_action should fail when persona is missing, got {result:?}"
    );

    let (status, _) = app
        .game_view_query
        .get_generating_status()
        .expect("get_generating_status should succeed");
    assert_eq!(
        status,
        GenerationStatus::Idle,
        "stale Generating status should be healed to Idle before validation error returns"
    );
}
