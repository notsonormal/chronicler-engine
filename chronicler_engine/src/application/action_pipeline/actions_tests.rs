use std::sync::{Arc, Barrier};

use crate::application::action_pipeline::execute_action_impl;
use crate::application::application_service::DefaultApplicationService;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::agents::registry::AgentRegistry;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::Agent;
use crate::test_support::fixtures::{TestGameState, TestNpc};
use crate::test_support::make_test_app_with_game_service;
use crate::test_support::make_test_recorder;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::domain::model::agent::{AgentContext, AgentResult, BackendSelector, ExecutionPhase};

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

fn make_test_state() -> GameState {
    TestGameState::with_npc("start", TestNpc::named("npc1", "Test NPC"))
}

#[test]
fn test_execute_action_impl_completes_and_persists_state() {
    let state = make_test_state();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = make_test_app_with_game_service(state, |_| Arc::new(service)).unwrap();
    execute_action_impl(&app, "look".to_string());
    let final_state = app.load_or_fresh();
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
    assert!(
        has_narration,
        "execute_action_impl should persist narration"
    );
}

#[test]
fn test_execute_action_impl_clears_last_trigger() {
    let mut state = make_test_state();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::named(
        "Old Trigger",
        "npc1",
    ));
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = make_test_app_with_game_service(state, |_| Arc::new(service)).unwrap();
    execute_action_impl(&app, "look".to_string());
    let final_state = app.load_or_fresh();
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared before pipeline runs"
    );
}

#[test]
fn test_execute_action_impl_handles_narration_error() {
    let state = make_test_state();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = make_test_app_with_game_service(state, |_| Arc::new(service)).unwrap();
    execute_action_impl(&app, "look".to_string());
    let final_state = app.load_or_fresh();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "State should reflect error status after failed narration"
    );
}

#[test]
fn test_execute_action_impl_handles_cancellation() {
    let state = make_test_state();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = make_test_app_with_game_service(state, |_| Arc::new(service)).unwrap();
    app.cancel_token().cancel();
    execute_action_impl(&app, "look".to_string());
    let final_state = app.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancellation should reset status to Idle"
    );
}

#[test]
fn test_execute_action_impl_preserves_existing_input_log() {
    let mut state = make_test_state();
    state.add_message(
        "examine room".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default())
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let service = make_test_service(narrator_recorder, quantifier_provider);
    let app = make_test_app_with_game_service(state, |_| Arc::new(service)).unwrap();
    execute_action_impl(&app, "examine room".to_string());
    let final_state = app.load_or_fresh();
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

    let state = make_test_state();
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
    let app: Arc<DefaultApplicationService> =
        make_test_app_with_game_service(state, |_| Arc::clone(&service)).unwrap();
    let app_for_thread = Arc::clone(&app);

    let handle = thread::spawn(move || {
        execute_action_impl(&app_for_thread, "look".to_string());
    });

    entered.wait();
    let mid_state = app.load_or_fresh();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying during post-generation, not stuck on Narrating"
    );

    release.wait();
    handle.join().expect("Action thread should complete");
    let final_state = app.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should reset to default after completion"
    );
}

#[test]
fn test_narration_saved_before_quantifying_phase() {
    use std::thread;

    let state = make_test_state();
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
    let app: Arc<DefaultApplicationService> =
        make_test_app_with_game_service(state, |_| Arc::clone(&service)).unwrap();
    let app_for_thread = Arc::clone(&app);

    let handle = thread::spawn(move || {
        execute_action_impl(&app_for_thread, "test".to_string());
    });

    entered.wait();
    let messages = app.load_messages().unwrap();
    let narration_count = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 1,
        "Narration should be saved before quantifier completes"
    );
    let mid_state = app.load_or_fresh();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying while quantifier runs"
    );

    release.wait();
    handle.join().expect("Action thread should complete");
}
